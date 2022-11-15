use crate::prelude::*;
use telegram_bot_api::types::{Message, Update};

pub trait UpdateHandler {
    fn message(&self, _msg: Message) -> UResult;
}

pub trait CommandHandler {
    fn forward_message(&self, _msg: Command) -> UResult;
}
