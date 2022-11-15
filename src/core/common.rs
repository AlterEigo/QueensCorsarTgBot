use slog::Logger;

use crate::prelude::*;

use std::io;
use std::io::Write;
use std::net::{self, TcpListener};
use std::os::unix::net as uxnet;
use std::sync::Arc;
use std::thread::{JoinHandle, ScopedJoinHandle};
use std::{
    io::{BufRead, BufReader, BufWriter, Read},
    sync::atomic::{AtomicBool, Ordering},
};

use http::{Request, Version};
use rustls::{ServerConfig, ServerConnection};
use telegram_bot_api::types::{Message, Update};

pub trait LoggingEntity {
    fn logger(&self) -> Logger;
}

pub trait StreamHandler<T>
where
    T: io::Read + io::Write,
    Self: Send + Sync,
{
    fn handle_stream(&self, stream: T) -> UResult;
}

pub type StreamHandlerArc<'a, ListenerT> = Arc<dyn StreamHandler<<ListenerT as ListenerAdapter<'a>>::StreamT>>;
pub type StreamHandlerRef<'a, ListenerT> = &'a dyn StreamHandler<<ListenerT as ListenerAdapter<'a>>::StreamT>;

pub trait ListenerAdapter<'a>: Send + Sync {
    type StreamT: io::Read + io::Write + Send + Sync;
    type SockAddrT;
    type IncomingT: Iterator<Item = io::Result<Self::StreamT>>;

    fn accept(&'a self) -> UResult<(Self::StreamT, Self::SockAddrT)>;

    fn incoming(&'a self) -> Self::IncomingT;
}

impl<'a> ListenerAdapter<'a> for net::TcpListener {
    type StreamT = net::TcpStream;
    type SockAddrT = net::SocketAddr;
    type IncomingT = net::Incoming<'a>;

    fn accept(&self) -> UResult<(Self::StreamT, Self::SockAddrT)> {
        match self.accept() {
            Ok((stream, addr)) => Ok((stream, addr)),
            Err(why) => Err(why.into()),
        }
    }

    fn incoming(&'a self) -> Self::IncomingT {
        self.incoming()
    }
}

impl<'a> ListenerAdapter<'a> for uxnet::UnixListener {
    type StreamT = uxnet::UnixStream;
    type SockAddrT = uxnet::SocketAddr;
    type IncomingT = uxnet::Incoming<'a>;

    fn accept(&self) -> UResult<(Self::StreamT, Self::SockAddrT)> {
        match self.accept() {
            Ok((stream, addr)) => Ok((stream, addr)),
            Err(why) => Err(why.into()),
        }
    }

    fn incoming(&'a self) -> Self::IncomingT {
        self.incoming()
    }
}

pub trait StreamListenerExt<'a, ListenerT>
where
    for<'x> ListenerT: ListenerAdapter<'x>,
    Self: Send + Sync
{
    fn request_stop(&'a mut self);

    fn is_stopped(&'a self) -> bool;

    fn listen(&'a self) -> UResult;
}

pub struct StreamListener<'a, ListenerT>
where
    for<'x> ListenerT: ListenerAdapter<'x>,
{
    logger: Logger,
    listener: ListenerT,
    stream_handler: Option<StreamHandlerRef<'a, ListenerT>>,
    stop_requested: AtomicBool,
}

pub struct StreamListenerBuilder<'a, T>
where
    for<'x> T: ListenerAdapter<'x>,
{
    listener: Option<T>,
    logger: Option<Logger>,
    handler: Option<StreamHandlerRef<'a, T>>,
}

impl<'a, T> Default for StreamListenerBuilder<'a, T>
where
    for<'x> T: ListenerAdapter<'x>,
{
    fn default() -> Self {
        StreamListenerBuilder {
            listener: None,
            logger: None,
            handler: None,
        }
    }
}

impl<'a, T> StreamListenerBuilder<'a, T>
where
    for<'x> T: ListenerAdapter<'x>,
{
    pub fn listener(self, new_listener: T) -> Self {
        Self {
            listener: Some(new_listener),
            ..self
        }
    }

    pub fn logger(self, new_logger: Logger) -> Self {
        Self {
            logger: Some(new_logger),
            ..self
        }
    }

    pub fn stream_handler(
        self,
        handler: StreamHandlerRef<'a, T>
    ) -> Self {
        Self {
            handler: Some(handler),
            ..self
        }
    }

    pub fn build(self) -> StreamListener<'a, T> {
        StreamListener {
            logger: self
                .logger
                .expect("Did not provide a logger for StreamListenerBuilder"),
            listener: self
                .listener
                .expect("Did not provide a listener type for StreamListenerBuilder"),
            stop_requested: AtomicBool::new(false),
            stream_handler: self.handler,
        }
    }
}

impl<'a, ListenerT> StreamListener<'a, ListenerT>
where
    for<'x> ListenerT: ListenerAdapter<'x>,
{
    pub fn new() -> StreamListenerBuilder<'a, ListenerT> {
        StreamListenerBuilder::<ListenerT>::default()
    }
}

impl<'a, ListenerT> StreamListenerExt<'a, ListenerT> for StreamListener<'a, ListenerT>
where
    for<'x> ListenerT: ListenerAdapter<'x>,
{
    fn request_stop(&mut self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }

    /// Engage the the loop for processing new connections in the
    /// current thread
    fn listen(&'a self) -> UResult {
        // A scope for each new spawned thread. All threads
        // spawned into a scope are guaranteed to be destroyed
        // before the function returns
        std::thread::scope(|scope| -> UResult {
            // New container for all worker threads
            let mut workers: Vec<ScopedJoinHandle<UResult>> = Vec::new();

            // Iterating through the connection queue and
            // spawning a new handler thread for each new
            // connection
            for stream in self.listener.incoming() {
                debug!(self.logger, "Handling incoming request");

                // Handling connection errors before processing
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

                // If a connection handler is available, we spawn
                // a new thread and delegating the connection processing
                // to this external stream handler
                if let Some(ref handler) = self.stream_handler {
                    let logger = self.logger.clone();
                    let handler = handler.clone();
                    let worker = scope.spawn(move || {
                        if let Err(why) = handler.handle_stream(stream) {
                            error!(logger, "TCP stream handling error"; "error" => format!("{:#?}", why));
                        }
                        Ok(())
                    });
                    workers.push(worker);
                }

                // Checking if the client requested server stop
                if self.is_stopped() {
                    break;
                }
            }

            // Joining all threads manually and handling
            // errors before qutting the scope
            for w in workers {
                if let Err(why) = w.join() {
                    error!(self.logger, "Error while joining the worker thread"; "reason" => format!("{:#?}", why));
                }
            }
            Ok(())
        })
    }
}
