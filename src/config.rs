use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::io::{BufRead, BufReader};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub server_ip: String,
    pub server_port: u16,
    pub private_key_path: String,
    pub certificate_path: String,
    pub token_var: String,
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
            "#,
        )
        .unwrap()
    }
}

pub const PACKAGE_VERSION: &'static str = std::env!("CARGO_PKG_VERSION");

lazy_static! {
    pub static ref APP_CONFIG: Config = {
        const FILENAME: &'static str = "bot_config.toml";
        let file = match std::fs::File::open(FILENAME) {
            Ok(opened) => opened,
            Err(why) => {
                if why.kind() == std::io::ErrorKind::NotFound {
                    let default_config = Config::default();
                    let default_config = toml::to_string(&default_config).unwrap();
                    let mut file = std::fs::File::create(FILENAME).unwrap();
                    write!(file, "{}", default_config).unwrap();
                    file
                } else {
                    panic!("{:#?}", why);
                }
            }
        };
        let mut reader = BufReader::new(file);
        let contents = String::from_utf8(reader.fill_buf().unwrap().to_vec()).unwrap();
        toml::from_str::<Config>(&contents).unwrap()
    };
}
