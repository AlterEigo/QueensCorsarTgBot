use std::fmt::Display;
use std::io;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{Scope, ScopedJoinHandle};
use std::time::Duration;
use std::{io::Read, os::unix::net::UnixStream};

use serde::{Deserialize, Serialize};
use slog::Logger;

use crate::prelude::*;

const PROTOCOL_VERSION: u16 = 100;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ActorInfos {
    server: String,
    sender: String
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum CommandKind {
    ForwardMessage {
        from: ActorInfos,
        to: ActorInfos,
        content: String
    },
    GetOnlineUsers
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum BotFamily {
    Discord,
    Telegram,
    WhatsApp
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Command {
    kind: CommandKind,
    sender_bot_family: BotFamily,
    protocol_version: u16
}

#[derive(Serialize,Deserialize,Debug)]
pub enum TransmissionResult {
    Received,
    BadSyntax,
    MismatchedVersions,
}

impl Display for TransmissionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TransmissionResult::Received => write!(f, "Received")?,
            TransmissionResult::BadSyntax => write!(f, "Bad syntax")?,
            TransmissionResult::MismatchedVersions => write!(f, "Mismatched protocol versions")?,
        }
        Ok(())
    }
}

pub struct CommandContext;
