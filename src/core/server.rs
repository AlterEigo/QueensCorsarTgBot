use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;

use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;

pub struct UpdateServer {
    stream_listener: Arc<dyn StreamListenerExt<TcpListener>>,
}

#[derive(Default)]
pub struct UpdateServerBuilder {
    update_handler: Option<Arc<dyn UpdateHandler>>,
    update_dispatcher: Option<Arc<dyn Dispatcher<Update>>>,
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

    pub fn update_handler(self, handler: Arc<dyn UpdateHandler>) -> Self {
        Self {
            update_handler: Some(handler),
            ..self
        }
    }

    pub fn update_dispatcher(self, dispatcher: Arc<dyn Dispatcher<Update>>) -> Self {
        assert!(
            self.update_handler.is_none(),
            "A custom update handler won't be used if a custom update dispatcher is provided"
        );
        Self {
            update_dispatcher: Some(dispatcher),
            ..self
        }
    }

    pub fn stream_handler(self, handler: Arc<dyn StreamHandler<TcpStream>>) -> Self {
        assert!(
            self.update_dispatcher.is_none(),
            "A custom update dispatcher won't be used if a custom stream handler is provided"
        );
        assert!(
            self.update_handler.is_none(),
            "A custom update handler won't be used if a custom stream handler is provided"
        );
        Self {
            stream_handler: Some(handler),
            ..self
        }
    }

    pub fn stream_listener<ListenerT>(self, listener: Arc<dyn StreamListenerExt<TcpListener>>) -> Self
    {
        assert!(
            self.bind_addr.is_none(),
            "You must either provide an address OR a listener"
        );
        assert!(
            self.stream_handler.is_none(),
            "A custom stream handler won't be used if a custom listener is provided"
        );
        assert!(
            self.update_dispatcher.is_none(),
            "A custom data dispatcher won't be used if a custom listener is provided"
        );
        assert!(
            self.update_handler.is_none(),
            "A custom update handler won't be used if a custom listener is provided"
        );
        Self {
            stream_listener: Some(listener),
            ..self
        }
    }

    pub fn build(self) -> UResult<UpdateServer> {
        assert!(
            self.logger.is_some(),
            "Did not provide a logger for the update server"
        );
        assert!(
            self.bind_addr.is_some() || self.stream_listener.is_some(),
            "Did not provide a listener or an interface to listen to"
        );

        let update_handler = self
            .update_handler
            .unwrap_or(Arc::new(DefaultUpdateHandler::default()));
        let update_dispatcher = self
            .update_dispatcher
            .unwrap_or(Arc::new(DefaultUpdateDispatcher::new(update_handler)));
        let stream_handler = self
            .stream_handler
            .unwrap_or(Arc::new(DefaultStreamHandler::new(update_dispatcher)));
        let stream_listener = self.stream_listener.unwrap_or(Arc::new(
            StreamListener::<TcpListener>::new()
                .listener(TcpListener::bind(self.bind_addr.unwrap())?)
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
}

impl DefaultStreamHandler {
    pub fn new(dispatcher: Arc<dyn Dispatcher<Update>>) -> Self {
        Self { dispatcher }
    }
}

impl StreamHandler<TcpStream> for DefaultStreamHandler {
    fn handle_stream(&self, stream: TcpStream) -> UResult {
        todo!()
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
