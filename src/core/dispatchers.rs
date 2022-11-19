use slog::Logger;
use telegram_bot_api::types::Update;

use crate::prelude::*;
use std::sync::Arc;

/// Default implementation of an update dispatcher
pub struct DefaultUpdateDispatcher {
    handler: Arc<dyn UpdateHandler>,
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
