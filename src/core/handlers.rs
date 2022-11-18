use crate::prelude::*;
use rustls::{ServerConfig, ServerConnection};
use slog::Logger;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use telegram_bot_api::types::Update;

/// Default implementation of an update handler
pub struct DefaultUpdateHandler {
    logger: Logger,
}

/// Default implementation of a qcproto command handler
pub struct DefaultCommandHandler {
    logger: Logger,
}

impl DefaultUpdateHandler {
    /// Instantiate a new default handler
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }
}

impl DefaultCommandHandler {
    /// Instantiate a new default command handler
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }
}

impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, _msg: telegram_bot_api::types::Message) -> UResult {
        info!(self.logger, "Received a message object!");
        Ok(())
    }
}

impl CommandHandler for DefaultCommandHandler {
    fn forward_message(&self, _msg: Command) -> UResult {
        todo!()
    }
}

/// Default implementation of an update handler
pub struct DefaultStreamHandler {
    dispatcher: Arc<dyn Dispatcher<Update>>,
    tls_config: ServerConfig,
    logger: Logger,
}

/// Default implementation of a unix stream handler
///
/// The default implementation expects that the data
/// being transmitted over the stream is done according
/// to the qcproto protocol
pub struct DefaultUnixStreamHandler {
    dispatcher: Arc<dyn Dispatcher<Command>>,
    logger: Logger,
}

/// Builder type allowing to configure and instantiate
/// a default update handler
#[derive(Default)]
pub struct DefaultStreamHandlerBuilder {
    dispatcher: Option<Arc<dyn Dispatcher<Update>>>,
    tls_config: Option<ServerConfig>,
    logger: Option<Logger>,
}

impl DefaultStreamHandlerBuilder {
    /// Set the data dispatcher
    pub fn dispatcher(self, dispatcher: Arc<dyn Dispatcher<Update>>) -> Self {
        Self {
            dispatcher: Some(dispatcher),
            ..self
        }
    }

    /// Set the configuration for SSL/TLS encryption
    pub fn tls_config(self, tls_config: ServerConfig) -> Self {
        Self {
            tls_config: Some(tls_config),
            ..self
        }
    }

    /// Set the integrated logger
    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    /// Finalize the instantiation of a default stream
    /// handler
    pub fn build(self) -> DefaultStreamHandler {
        assert!(
            self.logger.is_some(),
            "Did not provide a logger for the default stream handler"
        );
        assert!(
            self.dispatcher.is_some(),
            "Did not provide an update dispatcher for the default stream handler"
        );
        assert!(
            self.tls_config.is_some(),
            "Did not provide a tls config for the default stream handler"
        );

        DefaultStreamHandler {
            dispatcher: self.dispatcher.unwrap(),
            tls_config: self.tls_config.unwrap(),
            logger: self.logger.unwrap(),
        }
    }
}

impl DefaultStreamHandler {
    /// Instantiate a new default stream handler
    pub fn new() -> DefaultStreamHandlerBuilder {
        Default::default()
    }
}

impl DefaultUnixStreamHandler {
    /// Instantiate a new default unix stream handler
    pub fn new(
        dispatcher: Arc<dyn Dispatcher<Command>>,
        logger: Logger,
    ) -> DefaultUnixStreamHandler {
        DefaultUnixStreamHandler { dispatcher, logger }
    }
}

impl StreamHandler<UnixStream> for DefaultUnixStreamHandler {
    fn handle_stream(&self, mut stream: UnixStream) -> UResult {
        let response = serde_json::to_string(&TransmissionResult::Received)?;
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer);
        let command = serde_json::from_str::<Command>(&buffer);
        if let Err(_) = command {
            write!(
                stream,
                "{}",
                serde_json::to_string(&TransmissionResult::BadSyntax)?
            );
        }
        let command = command.unwrap();
        write!(stream, "{}", response);
        self.dispatcher.dispatch(command)
    }
}

impl StreamHandler<TcpStream> for DefaultStreamHandler {
    fn handle_stream(&self, mut stream: TcpStream) -> UResult {
        let response = http::Response::builder()
            .version(http::Version::HTTP_11)
            .status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"result":"ok"}"#)
            .unwrap();
        let mut conn = ServerConnection::new(Arc::new(self.tls_config.clone()))?;
        let mut stream = rustls::Stream::new(&mut conn, &mut stream);
        let request = read_http_request(&mut stream)?;
        let update = serde_json::from_str::<Update>(request.body())?;
        write_http_response(&mut stream, response)?;
        self.dispatcher.dispatch(update)?;
        Ok(())
    }
}
