use std::{sync::atomic::{AtomicBool, Ordering}, net::{TcpListener, TcpStream}, io::Read};
use std::io;
use std::io::Write;

use slog::Logger;

use crate::prelude::*;

pub trait UpdateHandler {

}

#[derive(Debug)]
pub struct UpdateProvider {
    tcp_handle: TcpListener,
    stop_requested: AtomicBool,
    logger: Logger
}

impl UpdateProvider {
    pub fn new() -> UpdateProviderBuilder {
        UpdateProviderBuilder::default()
    }

    fn handle_stream(&self, mut stream: TcpStream) -> UResult {
        let response = http::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(r"{'result':'ok'}")
            .unwrap();
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        write!(stream, "{}", response.body())?;
        Ok(())
    }

    pub async fn listen(&self) -> UResult {
        for stream in self.tcp_handle.incoming() {
            if let Err(err) = stream {
                match err.kind() {
                    io::ErrorKind::WouldBlock => continue,
                    _ => {
                        error!(self.logger, "TCP stream error"; "reason" => err.to_string());
                        return Err(err.into());
                    }
                };
            }
            let stream = stream.unwrap();

            if let Err(why) = Self::handle_stream(self, stream) {
                error!(self.logger, "Error while handling a tcp stream"; "reason" => why.to_string());
                continue;
            };
            if self.is_stopped() {
                break;
            }
        }
        Ok(())
    }

    pub fn request_stop(&mut self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }
}

#[derive(Default,Debug)]
pub struct UpdateProviderBuilder {
    tcp_listener: Option<TcpListener>,
    logger: Option<Logger>
}

impl UpdateProviderBuilder {
    pub fn listener(self, tcp_listener: TcpListener) -> Self {
        Self {
            tcp_listener: Some(tcp_listener),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn build(self) -> UpdateProvider {
        UpdateProvider {
            tcp_handle: self.tcp_listener.unwrap(),
            stop_requested: false.into(),
            logger: self.logger.unwrap()
        }
    }
}
