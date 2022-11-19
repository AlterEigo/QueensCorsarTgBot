use serde::Serialize;
use slog::Logger;

use crate::prelude::*;

use telegram_bot_api::types::Message;

/// An interface for handling dispatched telegram
/// updates
pub trait UpdateHandler: Send + Sync {
    /// Process a message received by the telegram bot
    fn message(&self, _msg: Message) -> UResult;
}
