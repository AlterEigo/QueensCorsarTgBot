use rustls::{Certificate, PrivateKey, ServerConfig};

use crate::prelude::UResult;

pub fn load_x509_credentials() -> UResult<(Vec<Certificate>, PrivateKey)> {
    todo!()
}

pub fn create_server_config() -> UResult<ServerConfig> {
    let (certs, pkey) = load_x509_credentials()?;
    Ok(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, pkey)?)
}
