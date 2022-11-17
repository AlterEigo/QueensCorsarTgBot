use crate::config;
use crate::prelude::*;
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::thread;
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

fn bootstrap_update_server(ctx: &BootstrapRequirements) -> UResult {
    let srv_addr = format!("{}:{}", ctx.config.server_ip, ctx.config.server_port);
    let tls_config = create_server_config(&ctx.config)?;

    let update_handler = Arc::new(DefaultUpdateHandler::new(ctx.logger.clone()));
    let update_dispatcher = Arc::new(DefaultUpdateDispatcher::new(
        update_handler,
        ctx.logger.clone(),
    ));
    let stream_handler = Arc::new(
        DefaultStreamHandler::new()
            .logger(ctx.logger.clone())
            .dispatcher(update_dispatcher.clone())
            .tls_config(tls_config.clone())
            .build(),
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
    update_server.listen()
}

fn bootstrap_command_server(ctx: &BootstrapRequirements) -> UResult {
    let srv_addr = format!("/tmp/qcorsar_sock");

    let command_handler = Arc::new(DefaultCommandHandler::new(ctx.logger.clone()));
    let command_dispatcher = Arc::new(DefaultCommandDispatcher::new(
        command_handler,
        ctx.logger.clone(),
    ));
    let stream_handler = Arc::new(
        DefaultUnixStreamHandler::new(command_dispatcher, ctx.logger.clone())
    );
    // let stream_listener = Arc::new(
        // StreamListener::<UnixListener>::new()
        // .logger(ctx.logger.clone())
        // .listener(UnixListener::bind(&srv_addr)?)
        // .stream_handler(stream_handler)
        // .build(),
    // );
    let update_server = CommandServer::new()
        .logger(ctx.logger.clone())
        .server_addr(&srv_addr)
        .stream_handler(stream_handler)
        .build()?;
    update_server.listen()
}

pub async fn bootstrap(ctx: BootstrapRequirements) -> UResult {
    introduce_self(&ctx);

    let bot_fut = instantiate_tgbot(&ctx);

    {
        let bot = bot_fut.await?;
        show_webhook_infos(&ctx, &bot).await?;
    }

    thread::scope(|scope| {

        scope.spawn(|| {
            if let Err(why) = bootstrap_update_server(&ctx) {
                todo!()
            }
        });

        scope.spawn(|| {
            if let Err(why) = bootstrap_command_server(&ctx) {
                todo!()
            }
        });

    });

    Ok(())
}
