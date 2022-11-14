use crate::prelude::*;

pub trait StreamHandler<T> {
    fn handle_stream(&self, stream: T) -> UResult;
}

pub trait StreamListener<T> {
    fn listen(&self) -> UResult {
        todo!()
    }
}
