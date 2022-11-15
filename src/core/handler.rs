use crate::prelude::*;
use telegram_bot_api::types::{Message, Update};

pub trait UpdateHandler: Send + Sync {
    fn message(&self, _msg: Message) -> UResult;
}

pub trait CommandHandler: Send + Sync {
    fn forward_message(&self, _msg: Command) -> UResult;
}
