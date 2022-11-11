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
        config: config::APP_CONFIG.clone(),
    };

    application::bootstrap(requirements).await
}
