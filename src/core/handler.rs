use crate::prelude::*;
use telegram_bot_api::types::{Update,Message};

pub trait Handler<T>: Send + Sync {
    fn dispatch(&self, data: T) -> UResult;
}

pub trait UpdateHandler: Handler<Update> {
    fn spec1(&self);
}

pub trait CommandHandler: Handler<Command> {
    fn spec2(&self);
}
