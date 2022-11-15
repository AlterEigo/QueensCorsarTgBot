use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

struct UpdateServer<'a> {
    update_handler: Arc<dyn UpdateHandler>,
    update_dispatcher: Arc<dyn Dispatcher<Update>>,
    stream_handler: Arc<dyn StreamHandler<TcpStream>>,
    stream_listener: Arc<StreamListener<'a, TcpListener>>,
}

#[derive(Default)]
struct DefaultUpdateHandler;
impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, _msg: telegram_bot_api::types::Message) -> UResult {
        Ok(())
    }
}

struct DefaultStreamHandler {
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

impl<'a> UpdateServer<'a> {
    fn new(addr: &str, logger: Logger) -> UResult<Self> {
        let update_handler = Arc::new(DefaultUpdateHandler::default());
        let update_dispatcher = Arc::new(DefaultUpdateDispatcher::new(update_handler));
        let stream_handler = Arc::new(DefaultStreamHandler::new(update_dispatcher));
        let stream_listener = Arc::new(
            StreamListener::new()
                .listener(TcpListener::bind(addr)?)
                .logger(logger)
                .stream_handler(stream_handler)
                .build(),
        );
        Ok(Self {
            update_handler,
            update_dispatcher,
            stream_handler,
            stream_listener,
        })
    }
}

impl<'a> StreamHandler<TcpStream> for UpdateServer<'a> {
    fn handle_stream(&self, stream: TcpStream) -> UResult {
        todo!()
    }
}

impl<'a> StreamListenerExt<'a, TcpListener> for UpdateServer<'a> {
    fn listen(&'a self) -> UResult {
        self.listener.listen()
    }

    fn request_stop(&'a mut self) {
        self.listener.request_stop()
    }

    fn is_stopped(&'a self) -> bool {
        self.listener.is_stopped()
    }
}
