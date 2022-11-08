mod core;
mod logger;
mod prelude;
mod utility;

use std::net::TcpListener;

use crate::prelude::*;
use tokio::main;

use telegram_bot_api::{bot, methods::{self, SetWebhook}, types};

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
        crit!(
            logger,
            "Could not instantiate the bot with the provided token"
        );
        return Err("BotApi instantiation error".into());
    }
    let bot = bot.unwrap();
    {
        let mut webhook = SetWebhook::new("https://45.67.230.27/".into());
        webhook.ip_address = Some("45.67.230.27".into());
        webhook.allowed_updates = Some(vec!["message".into()]);
        webhook.certificate = Some(load_input_file("tgbot.crt")?);
        if let Err(err) = bot.set_webhook(webhook).await {
            error!(logger, "Unable to set up the webhook"; "reason" => err.to_string());
            return Err("Webhook set up error".into());
        }
    }

    let server_thread = {
        let tls_config = create_server_config();
        if let Err(why) = tls_config {
            crit!(logger, "Could not instantiate a valid TLS config"; "reason" => why.to_string());
            return Err(why.into());
        }
        let tls_config = tls_config.unwrap();
        info!(logger, "TLS config successfully initialized");

        let server = TcpListener::bind("127.0.0.1:8080")?;
        info!(logger, "Starting server at port 8080");

        UpdateProvider::new()
            .logger(logger.clone())
            .listener(server)
            .tls_config(tls_config)
            .build()
            .listen()
    };

    if let Err(err) = server_thread.await {
        crit!(logger, "Critical error while running the server"; "reason" => err.to_string());
    }

    Ok(())
}
