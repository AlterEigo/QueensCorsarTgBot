use crate::prelude::*;
use slog::Logger;
use telegram_bot_api::types::{Message, Update};

pub struct DefaultUpdateHandler {
    logger: Logger
}

impl DefaultUpdateHandler {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger
        }
    }
}

impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, _msg: telegram_bot_api::types::Message) -> UResult {
        info!(self.logger, "Received a message object!");
        Ok(())
    }
}

