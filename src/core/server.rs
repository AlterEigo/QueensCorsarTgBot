use slog::Logger;

use crate::prelude::*;

use std::net::{TcpListener, TcpStream};

struct UpdateServer<'a> {
    listener: StreamListener<'a, TcpListener>,
    dispatcher: UpdateDispatcher,
}

#[derive(Default)]
struct DefaultUpdateHandler;

impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, _msg: telegram_bot_api::types::Message) -> UResult {
        Ok(())
    }
}

impl<'a> UpdateServer<'a> {
    fn new(addr: &str, logger: Logger) -> UResult<Self> {
        Ok(Self {
            listener: StreamListener::new()
                .listener(TcpListener::bind(addr)?)
                .logger(logger)
                .build(),
            dispatcher: UpdateDispatcher::new(DefaultUpdateHandler::default()),
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
