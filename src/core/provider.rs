use std::net::TcpStream;
use std::os::unix::net::UnixListener;
use std::{net::TcpListener, os::unix::net::UnixStream};

use std::io;
use crate::prelude::*;
use rustls::{Stream, ServerConnection, StreamOwned, ConfigSide};
use telegram_bot_api::types::Update;

pub trait Provider<T, S>
    where Self: Send + Sync,
          S: io::Read + io::Write
{
    type ListenerType;

    fn handle_stream(&self, _stream: S) -> UResult {
        Ok(())
    }
}

pub struct UpdateProvider;
pub struct CommandProvider;

impl Provider<Update, TcpStream> for UpdateProvider {
    type ListenerType = TcpListener;

    fn handle_stream(&self, _stream: TcpStream) -> UResult {
        todo!()
    }
}

impl Provider<Command, UnixStream> for CommandProvider {
    type ListenerType = UnixListener;

    fn handle_stream(&self, _stream: UnixStream) -> UResult {
        todo!()
    }
}
