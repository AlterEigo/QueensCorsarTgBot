use std::net::TcpStream;
use std::os::unix::net::UnixListener;
use std::{net::TcpListener, os::unix::net::UnixStream};

use std::io;
use crate::prelude::*;
use rustls::{Stream, ServerConnection, StreamOwned, ConfigSide};
use telegram_bot_api::types::Update;

pub trait Provider<T>: Send + Sync
{
    type ListenerType;
}

pub struct UpdateProvider;
pub struct CommandProvider;

impl Provider<Update> for UpdateProvider {
    type ListenerType = TcpListener;
}

impl Provider<Command> for CommandProvider {
    type ListenerType = UnixListener;
}
