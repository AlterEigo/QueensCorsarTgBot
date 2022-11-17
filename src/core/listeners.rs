use crate::prelude::*;

use slog::Logger;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::ScopedJoinHandle;

/// Default implementation of a stream listener for
/// any type which implements the ListenerAdapter
/// trait
pub struct StreamListener<ListenerT>
where
    ListenerT: ListenerAdapter,
{
    logger: Logger,
    listener: ListenerT,
    stream_handler: Option<StreamHandlerArc<ListenerT>>,
    stop_requested: AtomicBool,
}

/// A builder type for instantiating the default
/// stream listener
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
    /// Set the listener type
    pub fn listener(self, new_listener: T) -> Self {
        Self {
            listener: Some(new_listener),
            ..self
        }
    }

    /// Set the listener's logger
    pub fn logger(self, new_logger: Logger) -> Self {
        Self {
            logger: Some(new_logger),
            ..self
        }
    }

    /// Set an entity which will handle all established connections
    pub fn stream_handler(self, handler: StreamHandlerArc<T>) -> Self {
        Self {
            handler: Some(handler),
            ..self
        }
    }

    /// Finalize the instantiation of a stream listener
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
    /// Instantiate a new default stream listener
    pub fn new() -> StreamListenerBuilder<ListenerT> {
        StreamListenerBuilder::<ListenerT>::default()
    }

    /// Change the stream handler on the fly
    pub fn set_handler<'b>(&'b mut self, handler: StreamHandlerArc<ListenerT>) -> &'b mut Self {
        self.stream_handler = Some(handler);
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
            loop {
                let result = self.listener.accept();
                // let (stream, _) = self.listener.accept()?;
                debug!(self.logger, "Handling incoming request");

                // Handling connection errors before processing
                if let Err(err) = result {
                    match err.kind() {
                        io::ErrorKind::WouldBlock => continue,
                        _ => {
                            error!(self.logger, "TCP stream error"; "reason" => err.to_string());
                            return Err(err.into());
                        }
                    };
                }
                let (stream, _) = result.unwrap();

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
