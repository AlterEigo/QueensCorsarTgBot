use std::path::Path;

const SERVER_IP: &str = "";
const SERVER_PORT: u16 = 8443;
const SERVER_KEY_PATH: &str = "tgbot.key";
const SERVER_CERT_PATH: &str = "tgbot.cert";
const PACKAGE_VERSION: &str = std::env!("CARGO_PKG_VERSION");
const TOKEN_VARNAME: &str = "QUEENSCORSAR_TG_TOKEN";
