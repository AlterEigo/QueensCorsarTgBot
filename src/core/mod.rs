use std::io;
use std::io::Write;
use std::sync::Arc;
use std::thread::{JoinHandle, ScopedJoinHandle};
use std::{
    io::{BufRead, BufReader, BufWriter, Read},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
};

use http::{Request, Version};
use rustls::{ServerConfig, ServerConnection};
use slog::Logger;
use telegram_bot_api::types::{Message, Update};

use crate::prelude::*;

pub mod application;
pub struct UpdateContext;

mod common;
mod dispatcher;
mod handler;
mod sender;
mod server;

pub use common::*;
pub use dispatcher::*;
pub use handler::*;
pub use sender::*;
pub use server::*;

// pub trait UpdateHandler: Send + Sync {
// fn message(&self, context: &UpdateContext, message: Message) {}
// }

// struct DefaultHandler;
// impl UpdateHandler for DefaultHandler {
// fn message(&self, context: &UpdateContext, message: Message) {
// todo!()
// }
// }

fn parse_http_version(v: u8) -> UResult<http::Version> {
    match v {
        0 => Ok(http::Version::HTTP_09),
        1 => Ok(http::Version::HTTP_10),
        2 => Ok(http::Version::HTTP_2),
        3 => Ok(http::Version::HTTP_3),
        _ => Err("Wrong HTTP version".into()),
    }
}

// pub struct UpdateProvider {
// tcp_handle: TcpListener,
// stop_requested: AtomicBool,
// logger: Logger,
// tls_config: Arc<ServerConfig>,
// handler: Box<dyn UpdateHandler>,
// }

// impl UpdateProvider {
// pub fn new() -> UpdateProviderBuilder {
// UpdateProviderBuilder::default()
// }

// fn read_http_request<T>(&self, stream: &mut T) -> UResult<http::Request<String>>
// where
// T: std::io::Read,
// {
// let mut headers = [httparse::EMPTY_HEADER; 16];
// let mut reader = BufReader::new(stream);
// let buffer = reader.fill_buf()?.to_vec();
// let mut request_infos = httparse::Request::new(&mut headers);
// let parse_result = request_infos.parse(&buffer)?;
// if parse_result.is_partial() {
// return Err("Incoming HTTP request is not complete".into());
// }
// let parse_result = parse_result.unwrap();
// let mut request = http::Request::builder()
// .method(request_infos.method.unwrap())
// .uri(request_infos.path.unwrap())
// .version(parse_http_version(request_infos.version.unwrap())?);
// for header in headers {
// if header != httparse::EMPTY_HEADER {
// request = request.header(header.name, header.value);
// }
// }
// let buffer: Vec<u8> = buffer.into_iter().skip(parse_result).collect();
// let request = request.body(String::from_utf8(buffer)?)?;
// Ok(request)
// }

// fn write_http_response<T>(&self, stream: &mut T, response: http::Response<&str>) -> UResult
// where
// T: std::io::Write,
// {
// let (parts, body) = response.into_parts();
// let version = match parts.version {
// Version::HTTP_09 => "HTTP/0.9",
// Version::HTTP_10 => "HTTP/1.0",
// Version::HTTP_11 => "HTTP/1.1",
// Version::HTTP_2 => "HTTP/2.0",
// Version::HTTP_3 => "HTTP/3.0",
// _ => return Err("Impossible version enum error".into()),
// };
// let buffer = Vec::new();
// let mut bufwriter = BufWriter::new(buffer);
// write!(
// bufwriter,
// "{} {} {}\r\n",
// version,
// parts.status.as_str(),
// parts.status.canonical_reason().unwrap()
// )?;
// for (key, value) in parts.headers {
// let key = key.unwrap();
// write!(
// bufwriter,
// "{}: {}\r\n",
// key,
// String::from_utf8(value.as_bytes().to_vec())?
// )?;
// }
// write!(bufwriter, "\r\n{}", body)?;
// let data = String::from_utf8(bufwriter.buffer().to_vec())?;
// debug!(self.logger, "{}", &data);
// write!(stream, "{}", &data)?;
// Ok(())
// }

// fn dispatch_request(&self, request: Request<String>) -> UResult {
// let update = serde_json::from_str::<Update>(request.body())?;
// debug!(self.logger, "Received an update"; "update" => format!("{:#?}", update));
// let dummy_context = UpdateContext {};
// if let Some(message) = update.message {
// self.handler.message(&dummy_context, message);
// }
// Ok(())
// }

// fn handle_stream(&self, mut stream: TcpStream) -> UResult {
// let response = http::Response::builder()
// .version(http::Version::HTTP_11)
// .status(200)
// .header("Content-Type", "application/json")
// .body(r#"{"result":"ok"}"#)
// .unwrap();
// let mut conn = ServerConnection::new(self.tls_config.clone())?;
// let mut stream = rustls::Stream::new(&mut conn, &mut stream);
// let request = self.read_http_request(&mut stream)?;
// self.write_http_response(&mut stream, response)?;
// self.dispatch_request(request)?;
// info!(self.logger, "Successfully responded");
// Ok(())
// }

// pub async fn listen(self) -> UResult {
// std::thread::scope(|scope| -> UResult {
// let mut workers: Vec<ScopedJoinHandle<UResult>> = Vec::new();
// for stream in self.tcp_handle.incoming() {
// debug!(self.logger, "Handling incoming request");
// if let Err(err) = stream {
// match err.kind() {
// io::ErrorKind::WouldBlock => continue,
// _ => {
// error!(self.logger, "TCP stream error"; "reason" => err.to_string());
// return Err(err.into());
// }
// };
// }
// let stream = stream.unwrap();
// let worker = scope.spawn(|| {
// if let Err(why) = self.handle_stream(stream) {
// error!(self.logger, "TCP stream handling error"; "error" => format!("{:#?}", why));
// }
// Ok(())
// });
// workers.push(worker);

// if self.is_stopped() {
// break;
// }
// }
// for w in workers {
// if let Err(why) = w.join() {
// error!(self.logger, "Error while joining the worker thread"; "reason" => format!("{:#?}", why));
// }
// }
// Ok(())
// })
// }

// pub fn request_stop(&mut self) {
// self.stop_requested.store(false, Ordering::Relaxed)
// }

// pub fn is_stopped(&self) -> bool {
// self.stop_requested.load(Ordering::Relaxed)
// }
// }

// #[derive(Default)]
// pub struct UpdateProviderBuilder {
// tcp_listener: Option<TcpListener>,
// logger: Option<Logger>,
// tls_config: Option<ServerConfig>,
// update_handler: Option<Box<dyn UpdateHandler>>,
// }

// impl UpdateProviderBuilder {
// pub fn listener(self, tcp_listener: TcpListener) -> Self {
// Self {
// tcp_listener: Some(tcp_listener),
// ..self
// }
// }

// pub fn logger(self, logger: Logger) -> Self {
// Self {
// logger: Some(logger),
// ..self
// }
// }

// pub fn tls_config(self, config: ServerConfig) -> Self {
// Self {
// tls_config: Some(config),
// ..self
// }
// }

// pub fn update_handler(self, handler: Box<dyn UpdateHandler>) -> Self {
// Self {
// update_handler: Some(handler),
// ..self
// }
// }

// pub fn build(self) -> UResult<UpdateProvider> {
// let provider = UpdateProvider {
// tcp_handle: self
// .tcp_listener
// .ok_or("TCP Listener not provided".to_owned())?,
// stop_requested: false.into(),
// logger: self.logger.ok_or("Logger not provided".to_owned())?,
// tls_config: Arc::new(
// self.tls_config
// .ok_or("TLS Config not provided".to_owned())?,
// ),
// handler: self.update_handler.unwrap_or(Box::new(DefaultHandler {})),
// };
// Ok(provider)
// }
// }
