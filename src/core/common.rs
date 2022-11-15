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
{
    fn handle_stream(&self, stream: T) -> UResult;
}

pub trait ListenerAdapter<'a>: Send + Sync {
    type StreamT: io::Read + io::Write + Send + Sync;
    type SockAddrT;
    type IncomingT: 'a + Iterator<Item = io::Result<Self::StreamT>>;

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

pub trait StreamListenerExt<ListenerT>
where
    for<'a> ListenerT: 'a + ListenerAdapter<'a>,
{
    fn request_stop(&mut self);

    fn is_stopped(&self) -> bool;

    fn listen(&self) -> UResult;
}

pub struct StreamListener<ListenerT>
where
    for<'a> ListenerT: 'a + ListenerAdapter<'a>,
{
    logger: Logger,
    listener: ListenerT,
    stop_requested: AtomicBool,
}

#[derive(Debug)]
pub struct StreamListenerBuilder<T>
where
    for<'a> T: 'a + ListenerAdapter<'a>,
{
    listener: Option<T>,
    logger: Option<Logger>,
}

impl<T> Default for StreamListenerBuilder<T>
where
    for<'a> T: 'a + ListenerAdapter<'a>,
{
    fn default() -> Self {
        StreamListenerBuilder {
            listener: None,
            logger: None,
        }
    }
}

impl<T> StreamListenerBuilder<T>
where
    for<'a> T: 'a + ListenerAdapter<'a>,
{
    fn listener(self, new_listener: T) -> Self {
        Self {
            listener: Some(new_listener),
            ..self
        }
    }

    fn logger(self, new_logger: Logger) -> Self {
        Self {
            logger: Some(new_logger),
            ..self
        }
    }

    fn build(self) -> StreamListener<T> {
        StreamListener::<T> {
            logger: self.logger.expect("Did not provide a logger for StreamListenerBuilder"),
            listener: self.listener.expect("Did not provide a listener type for StreamListenerBuilder"),
            stop_requested: AtomicBool::new(false),
        }
    }
}

impl<ListenerT, StreamT> StreamHandler<StreamT> for StreamListener<ListenerT>
where
    for<'a> ListenerT: 'a + ListenerAdapter<'a>,
    StreamT: io::Read + io::Write,
{
    fn handle_stream(&self, stream: StreamT) -> UResult {
        todo!()
    }
}

impl<ListenerT> StreamListener<ListenerT>
where
    for<'a> ListenerT: 'a + ListenerAdapter<'a>,
{
    fn new() -> StreamListenerBuilder<ListenerT> {
        StreamListenerBuilder::<ListenerT>::default()
    }
}

impl<ListenerT> StreamListenerExt<ListenerT> for StreamListener<ListenerT>
where
    for<'a> ListenerT: 'a + ListenerAdapter<'a>,
{
    fn request_stop(&mut self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }

    fn listen(&self) -> UResult {
        std::thread::scope(|scope| -> UResult {
            let mut workers: Vec<ScopedJoinHandle<UResult>> = Vec::new();
            for stream in self.listener.incoming() {
                debug!(self.logger, "Handling incoming request");
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
                let worker = scope.spawn(|| {
                    if let Err(why) = self.handle_stream(stream) {
                        error!(self.logger, "TCP stream handling error"; "error" => format!("{:#?}", why));
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
}
