use crate::prelude::*;
use rustls::{ServerConfig, ServerConnection};
use slog::Logger;
use telegram_bot_api::bot::BotApi;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use telegram_bot_api::types::Update;

#[derive(Debug)]
pub struct AppCommandHandler {
    logger: Logger,
    tgbot: Arc<BotApi>
}

#[derive(Debug,Default)]
pub struct AppCommandHandlerBuilder {
    logger: Option<Logger>,
    tgbot: Option<Arc<BotApi>>
}

impl AppCommandHandler {
    pub fn new() -> AppCommandHandlerBuilder {
        Default::default()
    }
}

impl AppCommandHandlerBuilder {
    pub fn bot(self, tgbot: Arc<BotApi>) -> Self {
        Self {
            tgbot: Some(tgbot),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn build(self) -> AppCommandHandler {
        assert!(self.logger.is_some(), "Did not provide a logger for the app command handler");
        assert!(self.tgbot.is_some(), "Did not provide the telegram bot handle for the app command handler");
        AppCommandHandler {
            logger: self.logger.unwrap(),
            tgbot: self.tgbot.unwrap()
        }
    }
}

/// Default implementation of an update handler
#[derive(Debug)]
pub struct DefaultUpdateHandler {
    discord_sender: Option<Arc<CommandSender>>,
    logger: Logger,
}
impl DefaultUpdateHandler {
    /// Instantiate a new default handler
    pub fn new() -> DefaultUpdateHandlerBuilder {
        Default::default()
    }
}

#[derive(Default, Debug)]
pub struct DefaultUpdateHandlerBuilder {
    discord_sender: Option<Arc<CommandSender>>,
    logger: Option<Logger>,
}

impl DefaultUpdateHandlerBuilder {
    pub fn discord_sender(self, sender: Arc<CommandSender>) -> Self {
        Self {
            discord_sender: Some(sender),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn build(self) -> DefaultUpdateHandler {
        assert!(
            self.logger.is_some(),
            "Did not provide a logger for the default update handler"
        );

        DefaultUpdateHandler {
            discord_sender: self.discord_sender,
            logger: self.logger.unwrap(),
        }
    }
}

impl UpdateHandler for DefaultUpdateHandler {
    fn message(&self, msg: telegram_bot_api::types::Message) -> UResult {
        info!(self.logger, "Received a message object!");
        let author = if msg.from.is_none() {
            "Unknown".to_owned()
        } else {
            format_user_name(&msg.from.as_ref().unwrap())
        };
        let cmd = Command {
            kind: CommandKind::ForwardMessage {
                from: ActorInfos { server: format!("{}", msg.chat.id), name: author },
                to: ActorInfos { server: "1032941443058241546".to_string(), name: Default::default() },
                content: msg.text.unwrap_or(Default::default())
            },
            sender_bot_family: BotFamily::Telegram,
            protocol_version: qcproto::types::PROTOCOL_VERSION
        };

        if let Some(ref sender) = self.discord_sender {
            sender.send(cmd)?;
        }
        Ok(())
    }
}

/// Default implementation of an update handler
pub struct DefaultStreamHandler {
    dispatcher: Arc<dyn Dispatcher<Update>>,
    tls_config: ServerConfig,
    logger: Logger,
}

/// Builder type allowing to configure and instantiate
/// a default update handler
#[derive(Default)]
pub struct DefaultStreamHandlerBuilder {
    dispatcher: Option<Arc<dyn Dispatcher<Update>>>,
    tls_config: Option<ServerConfig>,
    logger: Option<Logger>,
}

impl DefaultStreamHandlerBuilder {
    /// Set the data dispatcher
    pub fn dispatcher(self, dispatcher: Arc<dyn Dispatcher<Update>>) -> Self {
        Self {
            dispatcher: Some(dispatcher),
            ..self
        }
    }

    /// Set the configuration for SSL/TLS encryption
    pub fn tls_config(self, tls_config: ServerConfig) -> Self {
        Self {
            tls_config: Some(tls_config),
            ..self
        }
    }

    /// Set the integrated logger
    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    /// Finalize the instantiation of a default stream
    /// handler
    pub fn build(self) -> DefaultStreamHandler {
        assert!(
            self.logger.is_some(),
            "Did not provide a logger for the default stream handler"
        );
        assert!(
            self.dispatcher.is_some(),
            "Did not provide an update dispatcher for the default stream handler"
        );
        assert!(
            self.tls_config.is_some(),
            "Did not provide a tls config for the default stream handler"
        );

        DefaultStreamHandler {
            dispatcher: self.dispatcher.unwrap(),
            tls_config: self.tls_config.unwrap(),
            logger: self.logger.unwrap(),
        }
    }
}

impl DefaultStreamHandler {
    /// Instantiate a new default stream handler
    pub fn new() -> DefaultStreamHandlerBuilder {
        Default::default()
    }
}

impl StreamHandler<TcpStream> for DefaultStreamHandler {
    fn handle_stream(&self, mut stream: TcpStream) -> UResult {
        let response = http::Response::builder()
            .version(http::Version::HTTP_11)
            .status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"result":"ok"}"#)
            .unwrap();
        let mut conn = ServerConnection::new(Arc::new(self.tls_config.clone()))?;
        let mut stream = rustls::Stream::new(&mut conn, &mut stream);
        let request = read_http_request(&mut stream)?;
        let update = serde_json::from_str::<Update>(request.body())?;
        write_http_response(&mut stream, response)?;
        self.dispatcher.dispatch(update)?;
        Ok(())
    }
}
