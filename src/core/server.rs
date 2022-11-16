use rustls::{ServerConfig,ServerConnection};
use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;

use serde::{Serialize,Deserialize};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;

pub struct UpdateServer {
    stream_listener: Arc<dyn StreamListenerExt<TcpListener>>,
}

#[derive(Default)]
pub struct UpdateServerBuilder {
    stream_handler: Option<Arc<dyn StreamHandler<TcpStream>>>,
    stream_listener: Option<Arc<dyn StreamListenerExt<TcpListener>>>,
    logger: Option<Logger>,
    bind_addr: Option<String>,
}

impl UpdateServerBuilder {
    pub fn server_addr(self, addr: &str) -> Self {
        assert!(
            self.stream_listener.is_none(),
            "You must either provide an address OR a listener"
        );
        Self {
            bind_addr: Some(String::from(addr)),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn stream_handler(self, handler: Arc<dyn StreamHandler<TcpStream>>) -> Self {
        assert!(
            self.stream_listener.is_none(),
            "Custom stream handler won't be used if a custom listener is provided"
        );
        Self {
            stream_handler: Some(handler),
            ..self
        }
    }

    pub fn stream_listener<ListenerT>(
        self,
        listener: Arc<dyn StreamListenerExt<TcpListener>>,
    ) -> Self {
        assert!(
            self.bind_addr.is_none(),
            "You must either provide an address OR a listener"
        );
        assert!(
            self.stream_handler.is_none(),
            "A custom stream handler won't be used if a custom listener is provided"
        );
        Self {
            stream_listener: Some(listener),
            ..self
        }
    }

    pub fn build(self) -> UResult<UpdateServer> {
        let flags = (
            self.logger.is_some(),
            self.bind_addr.is_some(),
            self.stream_listener.is_some(),
            self.stream_handler.is_some()
        );
        match flags {
            (false, _, _, _) => panic!("Did not provide a logger for the update server"),
            (_, true, true, _) | (_, false, false, _) => panic!("You have to provide either an address to bind to, or a configured listener"),
            (_, _, true, true) => panic!("A custom stream handler won't be used if you also provide a listener"),
            _ => ()
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

#[derive(Default)]
pub struct DefaultUpdateHandler;
impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, _msg: telegram_bot_api::types::Message) -> UResult {
        Ok(())
    }
}

pub struct DefaultStreamHandler {
    dispatcher: Arc<dyn Dispatcher<Update>>,
    tls_config: ServerConfig,
}

impl DefaultStreamHandler {
    pub fn new(dispatcher: Arc<dyn Dispatcher<Update>>, tls_config: ServerConfig) -> Self {
        Self {
            dispatcher,
            tls_config,
        }
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

impl UpdateServer {
    pub fn new() -> UpdateServerBuilder {
        Default::default()
    }
}

impl StreamHandler<TcpStream> for UpdateServer {
    fn handle_stream(&self, stream: TcpStream) -> UResult {
        todo!()
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
