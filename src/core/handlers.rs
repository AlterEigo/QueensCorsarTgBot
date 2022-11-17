use crate::prelude::*;
use rustls::{ServerConfig, ServerConnection};
use slog::Logger;
use std::net::TcpStream;
use std::sync::Arc;
use telegram_bot_api::types::Update;

pub struct DefaultUpdateHandler {
    logger: Logger,
}

impl DefaultUpdateHandler {
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

pub struct DefaultStreamHandler {
    dispatcher: Arc<dyn Dispatcher<Update>>,
    tls_config: ServerConfig,
    logger: Logger,
}

#[derive(Default)]
pub struct DefaultStreamHandlerBuilder {
    dispatcher: Option<Arc<dyn Dispatcher<Update>>>,
    tls_config: Option<ServerConfig>,
    logger: Option<Logger>,
}

impl DefaultStreamHandlerBuilder {
    pub fn dispatcher(self, dispatcher: Arc<dyn Dispatcher<Update>>) -> Self {
        Self {
            dispatcher: Some(dispatcher),
            ..self
        }
    }

    pub fn tls_config(self, tls_config: ServerConfig) -> Self {
        Self {
            tls_config: Some(tls_config),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

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
    pub fn new() -> DefaultStreamHandlerBuilder {
        Default::default()
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
