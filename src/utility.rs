use std::io::BufReader;
use std::iter;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{Item, read_one};

use crate::prelude::UResult;

pub fn load_x509_credentials() -> UResult<(Vec<Certificate>, PrivateKey)> {
    let cert_file = std::fs::File::open(std::path::Path::new("tgbot.crt"))?;
    let key_file = std::fs::File::open(std::path::Path::new("tgbot.key"))?;
    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let mut certs = Vec::new();
    for item in iter::from_fn(|| read_one(&mut cert_reader).transpose()) {
        match item.unwrap() {
            Item::X509Certificate(cert) => certs.push(Certificate(cert)),
            _ => return Err("Expected X509 certificates, found something else".into())
        }
    }
    let key = {
        let item = read_one(&mut key_reader)?.unwrap();
        match item {
            Item::RSAKey(key) => PrivateKey(key),
            _ => return Err("Expected an RSA private key, found something else".into())
        }
    };
    Ok((certs, key))
}

pub fn create_server_config() -> UResult<ServerConfig> {
    let (certs, pkey) = load_x509_credentials()?;
    Ok(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, pkey)?)
}
