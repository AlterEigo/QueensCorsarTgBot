use std::io;
use std::io::Write;
use std::sync::Arc;
use std::thread::{JoinHandle, ScopedJoinHandle};
use std::{
    io::{BufRead, BufReader, BufWriter, Read},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
};

use http::{Request, Version};
use rustls::{ServerConfig, ServerConnection};
use slog::Logger;
use telegram_bot_api::types::{Message, Update};

use crate::prelude::*;

pub mod application;
pub struct UpdateContext;

mod common;
mod dispatcher;
mod handler;
mod sender;
mod server;

pub use common::*;
pub use dispatcher::*;
pub use handler::*;
pub use sender::*;
pub use server::*;

// fn dispatch_request(&self, request: Request<String>) -> UResult {
// let update = serde_json::from_str::<Update>(request.body())?;
// debug!(self.logger, "Received an update"; "update" => format!("{:#?}", update));
// let dummy_context = UpdateContext {};
// if let Some(message) = update.message {
// self.handler.message(&dummy_context, message);
// }
// Ok(())
// }
