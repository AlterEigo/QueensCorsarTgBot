mod config;
mod core;
mod logger;
mod prelude;
mod qcproto;
mod utility;

use crate::prelude::*;

#[tokio::main]
async fn main() -> UResult {
    let requirements = application::BootstrapRequirements {
        logger: logger::configure_compact_root()?,
        config: config::read_or_create("tgbot.toml")?,
    };

    application::bootstrap(requirements).await
}
