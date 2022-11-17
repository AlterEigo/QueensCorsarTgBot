use crate::config;
use crate::prelude::*;
use std::net::TcpListener;
use std::sync::Arc;
use telegram_bot_api::bot;

#[derive(Clone)]
pub struct BootstrapRequirements {
    pub logger: slog::Logger,
    pub config: config::Config,
    // + cli args
    // + environment
}

fn introduce_self(ctx: &BootstrapRequirements) {
    info!(ctx.logger, "Starting QueenCorsar telegram bot";
        "upstream" => "https://github.com/AlterEigo/QueensCorsarTgBot",
        "email" => "iaroslav.sorokin@gmail.com",
        "author" => "Iaroslav Sorokin",
        "version" => config::PACKAGE_VERSION,
    );
}

fn extract_token(ctx: &BootstrapRequirements) -> UResult<String> {
    let token = std::env::var(&ctx.config.token_var);
    if token.is_err() {
        crit!(
            ctx.logger,
            "Could not fetch the API token from the environment"
        );
        Err(token.err().unwrap().into())
    } else {
        Ok(token.unwrap())
    }
}

async fn instantiate_tgbot(ctx: &BootstrapRequirements) -> UResult<bot::BotApi> {
    let token = extract_token(&ctx)?;
    debug!(ctx.logger, "API token fetched");

    let bot = bot::BotApi::new(token.into(), None).await;
    match bot {
        Ok(v) => {
            info!(ctx.logger, "Telegram Bot instantiated");
            Ok(v)
        }
        Err(why) => {
            crit!(
                ctx.logger,
                "Could not instantiate the bot with the provided token";
                "env token" => &ctx.config.token_var,
                "reason" => format!("{:#?}", why)
            );
            Err("BotApi instantiation error".into())
        }
    }
}

// async fn instantiate_update_listener(ctx: &BootstrapRequirements) -> UResult<UpdateProvider> {
// let tls_config = create_server_config(&ctx.config);
// if let Err(why) = tls_config {
// crit!(ctx.logger, "Could not instantiate a valid TLS config"; "reason" => why.to_string());
// return Err(why.into());
// }
// let tls_config = tls_config.unwrap();
// info!(ctx.logger, "TLS config successfully initialized");

// let server = TcpListener::bind(format!(
// "{}:{}",
// &ctx.config.server_ip, &ctx.config.server_port
// ))?;
// info!(
// ctx.logger,
// "Starting server at {}:{}", &ctx.config.server_ip, &ctx.config.server_port
// );

// let up = UpdateProvider::new()
// .logger(ctx.logger.clone())
// .listener(server)
// .tls_config(tls_config)
// .build()?;
// info!(ctx.logger, "Server thread engaged");
// Ok(up)
// todo!()
// }

async fn show_webhook_infos(ctx: &BootstrapRequirements, bot: &bot::BotApi) -> UResult {
    let infos = bot.get_webhook_info().await;
    match infos {
        Ok(infos) => {
            info!(ctx.logger, "Webhook status"; "infos" => format!("{:#?}", infos));
            Ok(())
        }
        Err(_) => {
            crit!(ctx.logger, "Unable to get webhook infos!");
            Err("Webhook infos request error".into())
        }
    }
}

pub async fn bootstrap(ctx: BootstrapRequirements) -> UResult {
    introduce_self(&ctx);

    let srv_addr = format!("{}:{}", ctx.config.server_ip, ctx.config.server_port);
    let bot_fut = instantiate_tgbot(&ctx);
    // let listener_fut = instantiate_update_listener(&ctx);

    let tls_config = create_server_config(&ctx.config)?;

    let update_handler = Arc::new(DefaultUpdateHandler::new(ctx.logger.clone()));
    let update_dispatcher = Arc::new(DefaultUpdateDispatcher::new(update_handler, ctx.logger.clone()));
    let stream_handler = Arc::new(
        DefaultStreamHandler::new()
            .logger(ctx.logger.clone())
            .dispatcher(update_dispatcher.clone())
            .tls_config(tls_config.clone())
            .build()
    );
    // let stream_listener = Arc::new(
        // StreamListener::<TcpListener>::new()
            // .logger(ctx.logger.clone())
            // .listener(TcpListener::bind(srv_addr)?)
            // .stream_handler(stream_handler)
            // .build(),
    // );
    let update_server = UpdateServer::new()
        .logger(ctx.logger.clone())
        .server_addr(&srv_addr)
        .stream_handler(stream_handler)
        .build()?;

    {
        let bot = bot_fut.await?;
        show_webhook_infos(&ctx, &bot).await?;
    }

    {
        if let Err(err) = update_server.listen() {
            crit!(ctx.logger, "Critical error while running the server"; "reason" => err.to_string());
        }
    }

    // Ok(())

    todo!()
}
