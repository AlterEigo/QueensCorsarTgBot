use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;
use std::sync::Arc;

pub trait Dispatcher<T>: Send + Sync {
    fn dispatch(&self, data: T) -> UResult;
}

pub struct DefaultUpdateDispatcher {
    handler: Arc<dyn UpdateHandler>,
    logger: Logger,
}

pub struct DefaultCommandDispatcher {
    handler: Arc<dyn CommandHandler>,
    logger: Logger,
}

impl DefaultUpdateDispatcher {
    pub fn new(handler: Arc<dyn UpdateHandler>, logger: Logger) -> Self {
        Self {
            handler: handler.clone(),
            logger,
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
