use crate::prelude::*;

pub trait Dispatcher<T> {
    fn dispatch(&self, data: T) -> UResult;
}
