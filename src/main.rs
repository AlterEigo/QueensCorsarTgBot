mod logger;
mod core;
mod prelude;
mod utility;

use std::net::TcpListener;

use tokio::main;
use crate::prelude::*;

use telegram_bot_api::{bot,methods,types};

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

    let bot = bot::BotApi::new(token, None).await;
    if bot.is_err() {
        crit!(logger, "Could not instantiate the bot with the provided token");
        return Err("BotApi instantiation error".into());
    }

    // Temporary
    let tls_config = create_server_config()?;
    let server = TcpListener::bind("127.0.0.1:8080")?;
    let server = UpdateProvider::new()
        .logger(logger.clone())
        .listener(server)
        .tls_config(tls_config)
        .build();

    if let Err(err) = server.listen().await {
        crit!(logger, "Critical error while running the server"; "reason" => err.to_string());
    }

    Ok(())
}
