use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;
use std::sync::Arc;

/// An interface for dispatching any received
/// data and possibly delegating it to another
/// specific data handler
pub trait Dispatcher<T>: Send + Sync {
    fn dispatch(&self, data: T) -> UResult;
}

/// Default implementation of an update dispatcher
pub struct DefaultUpdateDispatcher {
    handler: Arc<dyn UpdateHandler>,
    logger: Logger,
}

/// Default implementation of an interprocess
/// command dispatcher
pub struct DefaultCommandDispatcher {
    handler: Arc<dyn CommandHandler>,
    logger: Logger,
}

impl DefaultUpdateDispatcher {
    /// Instantiate a new default Telegram update dispatcher
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
    fn dispatch(&self, data: Command) -> UResult {
        info!(self.logger, "Incoming command: {:#?}", data);
        Ok(())
    }
}
