use std::io::BufReader;
use std::iter;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{read_one, Item};

use crate::prelude::UResult;

pub fn load_x509_certs(crt_path: &str) -> UResult<Vec<Certificate>> {
    let cert_file = std::fs::File::open(std::path::Path::new(crt_path))?;
    let mut cert_reader = BufReader::new(cert_file);

    let mut certs = Vec::new();
    for item in iter::from_fn(|| read_one(&mut cert_reader).transpose()) {
        match item.unwrap() {
            Item::X509Certificate(cert) => certs.push(Certificate(cert)),
            _ => return Err("Expected X509 certificates, found something else".into()),
        }
    }
    Ok(certs)
}

pub fn load_x509_secret_key(key_path: &str) -> UResult<PrivateKey> {
    let key_file = std::fs::File::open(std::path::Path::new(key_path))?;
    let mut key_reader = BufReader::new(key_file);

    let key = {
        let item = read_one(&mut key_reader)?.unwrap();
        match item {
            Item::RSAKey(key) => PrivateKey(key),
            _ => return Err("Expected an RSA private key, found something else".into()),
        }
    };
    Ok(key)
}

pub fn load_x509_credentials() -> UResult<(Vec<Certificate>, PrivateKey)> {
    let certs = load_x509_certs("tgbot.crt")?;
    let key = load_x509_secret_key("tgbot.key")?;
    Ok((certs, key))
}

pub fn create_server_config() -> UResult<ServerConfig> {
    let (certs, pkey) = load_x509_credentials()?;
    Ok(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, pkey)?)
}
