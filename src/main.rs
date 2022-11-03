mod logger;
mod core;
mod prelude;

use tokio::main;
use crate::prelude::*;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> UResult {
    let logger = configure_term_root();

    info!(logger, "Starting QueenCorsar telegram bot";
        "upstream" => "https://github.com/AlterEigo/QueensCorsarTgBot",
        "email" => "iaroslav.sorokin@gmail.com",
        "author" => "Iaroslav Sorokin",
        "version" => CRATE_VERSION,
    );

    let token = std::env::var("QUEENSCORSAR_TG_TOKEN");
    if token.is_err() {
        crit!(logger, "Could not fetch API token from the environment");
        return Err(token.err().unwrap().into());
    }
    let token = token.unwrap();
    debug!(logger, "API token fetched");

    Ok(())
}
