use std::fmt::Display;
use std::io;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{Scope, ScopedJoinHandle};
use std::time::Duration;
use std::{io::Read, os::unix::net::UnixStream};

use serde::{Deserialize, Serialize};
use slog::Logger;

use crate::prelude::*;

const PROTOCOL_VERSION: u16 = 100;

#[derive(Serialize, Deserialize)]
pub struct Command;

#[derive(Serialize, Deserialize, Debug)]
pub enum TransmissionResult {
    Received,
    BadSyntax,
    MismatchedVersions,
}

impl Display for TransmissionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TransmissionResult::Received => write!(f, "Received")?,
            TransmissionResult::BadSyntax => write!(f, "Bad syntax")?,
            TransmissionResult::MismatchedVersions => write!(f, "Mismatched protocol versions")?,
        }
        Ok(())
    }
}

pub struct CommandProvider {
    tunnel: UnixListener,
    stop_requested: AtomicBool,
    logger: Logger,
    handler: Box<dyn CommandHandler>,
}

impl CommandProvider {
    pub fn new() -> CommandProviderBuilder {
        CommandProviderBuilder::default()
    }

    fn dispatch_command(&self, cmd: Command) -> UResult {
        todo!()
    }

    fn handle_stream(&self, mut stream: UnixStream) -> UResult {
        let mut command = String::new();
        stream.read_to_string(&mut command)?;
        let command = serde_json::from_str::<Command>(&command)?;
        let mut dummy_context = CommandContext {};
        self.handler.dispatch_command(&mut dummy_context, command)
    }

    pub async fn listen(self) -> UResult {
        std::thread::scope(|scope| -> UResult {
            let mut workers: Vec<ScopedJoinHandle<UResult>> = Vec::new();
            for stream in self.tunnel.incoming() {
                debug!(self.logger, "Handling incoming request");
                if let Err(err) = stream {
                    match err.kind() {
                        io::ErrorKind::WouldBlock => continue,
                        _ => {
                            error!(self.logger, "Unix socket stream error"; "reason" => err.to_string());
                            return Err(err.into());
                        }
                    };
                }
                let stream = stream.unwrap();
                let worker = scope.spawn(|| {
                    if let Err(why) = self.handle_stream(stream) {
                        error!(self.logger, "Unix socket stream handling error"; "error" => format!("{:#?}", why));
                    }
                    Ok(())
                });
                workers.push(worker);

                if self.is_stopped() {
                    break;
                }
            }
            for w in workers {
                if let Err(why) = w.join() {
                    error!(self.logger, "Error while joining the worker thread"; "reason" => format!("{:#?}", why));
                }
            }
            Ok(())
        })
    }

    pub fn request_stop(&mut self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
pub struct CommandProviderBuilder {
    unix_listener: Option<UnixListener>,
    logger: Option<Logger>,
    command_handler: Option<Box<dyn CommandHandler>>,
}

impl CommandProviderBuilder {
    pub fn listener(self, listener: UnixListener) -> Self {
        Self {
            unix_listener: Some(listener),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn command_handler(self, handler: Box<dyn CommandHandler>) -> Self {
        Self {
            command_handler: Some(handler),
            ..self
        }
    }

    pub fn build(self) -> UResult<CommandProvider> {
        let provider = CommandProvider {
            tunnel: self
                .unix_listener
                .ok_or("Unix socket listener not provided".to_owned())?,
            stop_requested: false.into(),
            logger: self.logger.ok_or("Logger not provided".to_owned())?,
            handler: self
                .command_handler
                .ok_or("Command handler not provided".to_owned())?,
        };
        Ok(provider)
    }
}

pub struct CommandSender {
    tunnel: UnixStream,
}

impl CommandSender {
    fn new(socket_path: &str) -> UResult<Self> {
        let sender = CommandSender {
            tunnel: UnixStream::connect(socket_path)?,
        };
        sender.tunnel.set_read_timeout(Some(Duration::new(3, 0)))?;
        sender.tunnel.set_write_timeout(Some(Duration::new(3, 0)))?;
        Ok(sender)
    }

    fn send_command(&mut self, cmd: &Command) -> UResult {
        let serialized = serde_json::to_string(cmd)?;

        write!(self.tunnel, "{}", serialized)?;
        let mut response = String::new();
        self.tunnel.read_to_string(&mut response)?;
        let response = serde_json::from_str::<TransmissionResult>(&response)?;
        match response {
            TransmissionResult::Received => Ok(()),
            TransmissionResult::BadSyntax => Err("Malformed data syntax reported".into()),
            TransmissionResult::MismatchedVersions => {
                Err("Mismatched protocol versions reported".into())
            }
        }
    }
}

pub struct CommandContext;

pub trait CommandHandler: Send + Sync {
    fn dispatch_command(&self, ctx: &mut CommandContext, cmd: Command) -> UResult;
}
