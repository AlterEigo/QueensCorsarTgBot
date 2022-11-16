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

pub type StreamHandlerArc<ListenerT> =
    Arc<dyn StreamHandler<<ListenerT as ListenerAdapter>::StreamT>>;
pub type StreamHandlerRef<'a, ListenerT> =
    &'a dyn StreamHandler<<ListenerT as ListenerAdapter>::StreamT>;

pub trait ListenerAdapter: Send + Sync {
    type StreamT: io::Read + io::Write + Send + Sync;
    type SockAddrT;

    fn accept(&self) -> UResult<(Self::StreamT, Self::SockAddrT)>;
}

impl ListenerAdapter for net::TcpListener {
    type StreamT = net::TcpStream;
    type SockAddrT = net::SocketAddr;

    fn accept(&self) -> UResult<(Self::StreamT, Self::SockAddrT)> {
        match self.accept() {
            Ok((stream, addr)) => Ok((stream, addr)),
            Err(why) => Err(why.into()),
        }
    }
}

impl ListenerAdapter for uxnet::UnixListener {
    type StreamT = uxnet::UnixStream;
    type SockAddrT = uxnet::SocketAddr;

    fn accept(&self) -> UResult<(Self::StreamT, Self::SockAddrT)> {
        match self.accept() {
            Ok((stream, addr)) => Ok((stream, addr)),
            Err(why) => Err(why.into()),
        }
    }
}

pub trait StreamListenerExt<ListenerT>
where
    ListenerT: ListenerAdapter,
    Self: Send + Sync,
{
    fn request_stop(&self);

    fn is_stopped(&self) -> bool;

    fn listen(&self) -> UResult;
}

pub struct StreamListener<ListenerT>
where
    ListenerT: ListenerAdapter,
{
    logger: Logger,
    listener: ListenerT,
    stream_handler: Option<StreamHandlerArc<ListenerT>>,
    stop_requested: AtomicBool,
}

pub struct StreamListenerBuilder<T>
where
    T: ListenerAdapter,
{
    listener: Option<T>,
    logger: Option<Logger>,
    handler: Option<StreamHandlerArc<T>>,
}

impl<T> Default for StreamListenerBuilder<T>
where
    T: ListenerAdapter,
{
    fn default() -> Self {
        StreamListenerBuilder {
            listener: None,
            logger: None,
            handler: None,
        }
    }
}

impl<T> StreamListenerBuilder<T>
where
    T: ListenerAdapter,
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

    pub fn stream_handler(self, handler: StreamHandlerArc<T>) -> Self {
        Self {
            handler: Some(handler),
            ..self
        }
    }

    pub fn build(self) -> StreamListener<T> {
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

impl<ListenerT> StreamListener<ListenerT>
where
    ListenerT: ListenerAdapter,
{
    pub fn new() -> StreamListenerBuilder<ListenerT> {
        StreamListenerBuilder::<ListenerT>::default()
    }

    pub fn set_handler<'b>(&'b mut self, handler: StreamHandlerArc<ListenerT>) -> &'b mut Self {
        std::mem::replace(&mut self.stream_handler, Some(handler));
        self
    }
}

impl<ListenerT> StreamListenerExt<ListenerT> for StreamListener<ListenerT>
where
    ListenerT: ListenerAdapter,
{
    fn request_stop(&self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }

    /// Engage the the loop for processing new connections in the
    /// current thread
    fn listen(&self) -> UResult {
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
