use std::{sync::atomic::{AtomicBool, Ordering}, net::{TcpListener, TcpStream}};
use std::io;

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

    fn handle_stream(&self, stream: TcpStream) -> UResult {
        todo!()
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

}
