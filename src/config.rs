use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

use crate::prelude::*;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub server_ip: String,
    pub server_port: u16,
    pub private_key_path: String,
    pub certificate_path: String,
    pub token_var: String,
    pub sock_addr: PathBuf
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
            sock_addr = '/tmp/qcorsar.sock'
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
