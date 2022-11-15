use telegram_bot_api::types::{Message, Update};

use crate::prelude::*;

pub trait Dispatcher<T> {
    fn dispatch(&self, data: T) -> UResult;
}

pub struct UpdateDispatcher {
    handler: Box<dyn UpdateHandler>,
}

pub struct CommandDispatcher {
    handler: Box<dyn CommandHandler>,
}

impl UpdateDispatcher {
    fn new<T>(handler: T) -> Self
        where for<'a> T: 'a + UpdateHandler
    {
        Self {
            handler: Box::new(handler)
        }
    }
}

impl Dispatcher<Update> for UpdateDispatcher {
    fn dispatch(&self, _data: Update) -> UResult {
        todo!()
    }
}

impl Dispatcher<Command> for CommandDispatcher {
    fn dispatch(&self, _data: Command) -> UResult {
        todo!()
    }
}
