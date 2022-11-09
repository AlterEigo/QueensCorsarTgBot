use std::io::{BufRead, BufReader};
use std::iter;

use rustls::version::TLS13;
use rustls::{Certificate, ConfigBuilder, PrivateKey, ServerConfig, SupportedProtocolVersion};
use rustls_pemfile::{read_one, Item};
use telegram_bot_api::types::InputFile;

use crate::config::Config;
use crate::prelude::UResult;

pub fn load_input_file(file_path: &str) -> UResult<InputFile> {
    let file = std::fs::File::open(std::path::Path::new(file_path))?;
    let mut reader = BufReader::new(file);

    let bytes = reader.fill_buf()?.to_vec();
    Ok(InputFile::FileBytes(file_path.into(), bytes))
}

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

pub fn load_x509_credentials(config: &Config) -> UResult<(Vec<Certificate>, PrivateKey)> {
    let certs = load_x509_certs(&config.certificate_path)?;
    let key = load_x509_secret_key(&config.private_key_path)?;
    Ok((certs, key))
}

pub fn create_server_config(config: &Config) -> UResult<ServerConfig> {
    let (certs, pkey) = load_x509_credentials(config)?;
    let protocols: &[&'static SupportedProtocolVersion] = &[&TLS13];
    Ok(ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(protocols)?
        .with_no_client_auth()
        .with_single_cert(certs, pkey)?)
}
