use telegram_bot_api::types::{Update, Message};

use crate::prelude::*;

pub trait Dispatcher<T> {
    fn dispatch(&self, data: T) -> UResult;
}

pub struct UpdateDispatcher;
pub struct CommandDispatcher;

impl UpdateDispatcher {
}

impl Dispatcher<Update> for UpdateDispatcher {
    fn dispatch(&self, _data: Update) -> UResult {
        todo!()
    }
}
