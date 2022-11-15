use telegram_bot_api::types::{Message, Update};

use crate::prelude::*;
use std::sync::Arc;

pub trait Dispatcher<T>: Send + Sync {
    fn dispatch(&self, data: T) -> UResult;
}

pub struct DefaultUpdateDispatcher {
    handler: Arc<dyn UpdateHandler>,
}

pub struct DefaultCommandDispatcher {
    handler: Arc<dyn CommandHandler>,
}

impl DefaultUpdateDispatcher {
    pub fn new<T>(handler: Arc<T>) -> Self
    where
        T: UpdateHandler,
    {
        Self {
            handler: handler.clone(),
        }
    }
}

impl Dispatcher<Update> for DefaultUpdateDispatcher {
    fn dispatch(&self, data: Update) -> UResult {
        if let Some(msg) = data.message {
            self.handler.message(msg)?;
        }
        Ok(())
    }
}

impl Dispatcher<Command> for DefaultCommandDispatcher {
    fn dispatch(&self, _data: Command) -> UResult {
        todo!()
    }
}
