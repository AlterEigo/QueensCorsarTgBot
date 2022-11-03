use crate::prelude::*;
use chrono;
use slog::{o, Drain, Logger};

fn get_datetime_str() -> String {
    chrono::offset::Local::now()
        .format("%d-%m-%Y_%H-%M")
        .to_string()
}

/// Инициализатор логгера с компактным отображением
pub fn configure_compact_root() -> UResult<Logger> {
    let file = {
        let filename = format!("{}.txt", get_datetime_str());
        let file_path = std::path::Path::new(&filename);
        std::fs::File::create(file_path)?
    };
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::CompactFormat::new(decorator)
        .use_local_timestamp()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Ok(slog::Logger::root(drain, o!()))
}

/// Инициализатор логгера с полным отображением
pub fn configure_full_root() -> UResult<Logger> {
    let file = {
        let filename = format!("{}.txt", get_datetime_str());
        let file_path = std::path::Path::new(&filename);
        std::fs::File::create(file_path)?
    };
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator)
        .use_original_order()
        .use_local_timestamp()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Ok(slog::Logger::root(drain, o!()))
}
