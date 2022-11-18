use slog::Logger;

use crate::prelude::*;

use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;

/// Struct containing all the data to run the server
/// managing Telegram update webhook requests
pub struct UpdateServer {
    stream_listener: Arc<dyn StreamListenerExt<TcpListener>>,
}

/// Builder type for the construction of an update
/// server
#[derive(Default)]
pub struct UpdateServerBuilder {
    stream_handler: Option<Arc<dyn StreamHandler<TcpStream>>>,
    stream_listener: Option<Arc<dyn StreamListenerExt<TcpListener>>>,
    logger: Option<Logger>,
    bind_addr: Option<String>,
}

impl UpdateServerBuilder {
    /// Set the address for server in the format accepted by
    /// the standard `std::net::TcpListener` type ('IP_ADDR:PORT')
    pub fn server_addr(self, addr: &str) -> Self {
        Self {
            bind_addr: Some(String::from(addr)),
            ..self
        }
    }

    /// Set the logger for the server and all of the initialized
    /// default modules
    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    /// Set a handler for all established TCP connections
    pub fn stream_handler(self, handler: Arc<dyn StreamHandler<TcpStream>>) -> Self {
        Self {
            stream_handler: Some(handler),
            ..self
        }
    }

    /// Set a custom TCP listener
    pub fn stream_listener<ListenerT>(
        self,
        listener: Arc<dyn StreamListenerExt<TcpListener>>,
    ) -> Self {
        Self {
            stream_listener: Some(listener),
            ..self
        }
    }

    /// Finalize the update server construction.
    ///
    /// Construction of the server fails in the following scenarios:
    /// - A logger is not provided
    /// - Both server address and custom stream listener are provided, or neither of them
    /// - A custom stream handler is provided with the custom stream listener
    /// - If an error occurs while initializing one of the subcomponents
    pub fn build(self) -> UResult<UpdateServer> {
        let flags = (
            self.logger.is_some(),
            self.bind_addr.is_some(),
            self.stream_listener.is_some(),
            self.stream_handler.is_some(),
        );
        match flags {
            (false, _, _, _) => panic!("Did not provide a logger for the update server"),
            (_, true, true, _) | (_, false, false, _) => {
                panic!("You have to provide either an address to bind to, or a configured listener")
            }
            (_, _, true, true) => {
                panic!("A custom stream handler won't be used if you also provide a listener")
            }
            _ => (),
        };

        let stream_handler = self.stream_handler.unwrap();
        let stream_listener = self.stream_listener.unwrap_or(Arc::new(
            StreamListener::<TcpListener>::new()
                .listener(TcpListener::bind(self.bind_addr.unwrap())?)
                .stream_handler(stream_handler)
                .logger(self.logger.unwrap())
                .build(),
        ));
        let srv = UpdateServer { stream_listener };
        Ok(srv)
    }
}

impl UpdateServer {
    /// Instantiate a new update server
    pub fn new() -> UpdateServerBuilder {
        Default::default()
    }
}

impl StreamListenerExt<TcpListener> for UpdateServer {
    fn listen(&self) -> UResult {
        self.stream_listener.listen()
    }

    fn request_stop(&self) {
        self.stream_listener.request_stop()
    }

    fn is_stopped(&self) -> bool {
        self.stream_listener.is_stopped()
    }
}

/// Application's local command server implementation
/// allowing to communicate with other bots using qcproto
/// running on the local machine via unix sockets
pub struct CommandServer {
    stream_listener: Arc<dyn StreamListenerExt<UnixListener>>,
}

#[derive(Default)]
pub struct CommandServerBuilder {
    stream_handler: Option<Arc<dyn StreamHandler<UnixStream>>>,
    stream_listener: Option<Arc<dyn StreamListenerExt<UnixListener>>>,
    logger: Option<Logger>,
    bind_addr: Option<String>,
}

impl CommandServer {
    /// Instantiate a new command server
    pub fn new() -> CommandServerBuilder {
        Default::default()
    }
}

impl CommandServerBuilder {
    /// Set the address for server in the format accepted by
    /// the standard `std::os::unix::UnixListener` type ('UNIX_FILEPATH')
    pub fn server_addr(self, addr: &str) -> Self {
        Self {
            bind_addr: Some(String::from(addr)),
            ..self
        }
    }

    /// Set the logger for the server and all of the initialized
    /// default modules
    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    /// Set a handler for all established connections via unix sockets
    pub fn stream_handler(self, handler: Arc<dyn StreamHandler<UnixStream>>) -> Self {
        Self {
            stream_handler: Some(handler),
            ..self
        }
    }

    /// Set a custom unix socket listener
    pub fn stream_listener<ListenerT>(
        self,
        listener: Arc<dyn StreamListenerExt<UnixListener>>,
    ) -> Self {
        Self {
            stream_listener: Some(listener),
            ..self
        }
    }

    /// Finalize the update server construction.
    ///
    /// Construction of the server fails in the following scenarios:
    /// - A logger is not provided
    /// - Both server address and custom stream listener are provided, or neither of them
    /// - A custom stream handler is provided with the custom stream listener
    /// - If an error occurs while initializing one of the subcomponents
    pub fn build(self) -> UResult<CommandServer> {
        let flags = (
            self.logger.is_some(),
            self.bind_addr.is_some(),
            self.stream_listener.is_some(),
            self.stream_handler.is_some(),
        );
        match flags {
            (false, _, _, _) => panic!("Did not provide a logger for the command server"),
            (_, true, true, _) | (_, false, false, _) => {
                panic!("You have to provide either an address to bind to, or a configured listener")
            }
            (_, _, true, true) => {
                panic!("A custom stream handler won't be used if you also provide a listener")
            }
            _ => (),
        };

        let stream_handler = self.stream_handler.unwrap();
        let stream_listener = self.stream_listener.unwrap_or(Arc::new(
            StreamListener::<UnixListener>::new()
                .listener(UnixListener::bind(self.bind_addr.unwrap())?)
                .stream_handler(stream_handler)
                .logger(self.logger.unwrap())
                .build(),
        ));
        let srv = CommandServer { stream_listener };
        Ok(srv)
    }
}

impl StreamListenerExt<UnixListener> for CommandServer {
    fn listen(&self) -> UResult {
        self.stream_listener.listen()
    }

    fn request_stop(&self) {
        self.stream_listener.request_stop()
    }

    fn is_stopped(&self) -> bool {
        self.stream_listener.is_stopped()
    }
}
