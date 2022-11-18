#![allow(unused)]

mod config;
mod core;
mod logger;
mod prelude;
mod utility;

use crate::prelude::*;

#[tokio::main]
async fn main() -> UResult {
    let requirements = application::BootstrapRequirements {
        logger: logger::configure_term_root(),
        config: config::read_or_create("bot_config.toml")?,
    };

    application::bootstrap(requirements).await
}
