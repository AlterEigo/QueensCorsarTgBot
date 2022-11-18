use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::prelude::*;

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct ServersSection {
    discord: Option<PathBuf>,
    whatsapp: Option<PathBuf>
}

/// Main application config structure
///
/// Available settings:
/// - `server_ip` and `server_port`: Interface and port used to deploy a server
///   listening for incoming telegram updates
/// - `private_key_path`: Location of the private key used for SSL encryption
/// - `certificate_path`: Server's certificate used to authenticate our server
/// - `token_var`: Name of the environment variable used to retrieve telegram
///   api token
/// - `sock_addr`: Path to the socket used to receive data from other bots
///   via qcproto protocol
/// - 'integrations': Known sockets of other bots able to communicate via
///   qcproto protocol
#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Config {
    pub server_ip: String,
    pub server_port: u16,
    pub private_key_path: String,
    pub certificate_path: String,
    pub token_var: String,
    pub sock_addr: PathBuf,
    pub integrations: Option<ServersSection>
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str::<Config>(
            r#"
            server_ip = '127.0.0.1'
            server_port = 8443
            private_key_path = 'private.key'
            certificate_path = 'server.crt'
            token_var = 'QUEENSCORSAR_TG_TOKEN'
            sock_addr = '/tmp/qcorsar.tg.sock'
            "#,
        )
        .unwrap()
    }
}

pub const PACKAGE_VERSION: &'static str = std::env!("CARGO_PKG_VERSION");

pub fn create<T>(cfg_path: &str) -> UResult<T>
where
    T: Default + Serialize,
{
    let default_config = T::default();
    let serialized = toml::to_string(&default_config)?;
    let mut file = std::fs::File::create(cfg_path)?;
    write!(file, "{}", serialized)?;
    Ok(default_config)
}

pub fn read<T>(cfg_path: &str) -> UResult<T>
where
    T: DeserializeOwned,
{
    let contents = std::fs::read_to_string(&cfg_path)?;
    let config = toml::from_str::<T>(&contents)?;
    Ok(config)
}

pub fn read_or_create<T>(cfg_path: &str) -> UResult<T>
where
    T: Default + DeserializeOwned + Serialize,
{
    match read::<T>(cfg_path) {
        Ok(v) => Ok(v),
        Err(_) => create(cfg_path),
    }
}
