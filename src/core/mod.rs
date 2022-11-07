use std::{sync::atomic::{AtomicBool, Ordering}, net::{TcpListener, TcpStream}, io::Read};
use std::io;
use std::io::Write;

use slog::Logger;

use crate::prelude::*;

pub trait UpdateHandler {

}

fn parse_http_version(v: u8) -> UResult<http::Version> {
    match v {
        0 => Ok(http::Version::HTTP_09),
        1 => Ok(http::Version::HTTP_10),
        2 => Ok(http::Version::HTTP_2),
        3 => Ok(http::Version::HTTP_3),
        _ => Err("Wrong HTTP version".into())
    }
}

#[derive(Debug)]
pub struct UpdateProvider {
    tcp_handle: TcpListener,
    stop_requested: AtomicBool,
    logger: Logger
}

impl UpdateProvider {
    pub fn new() -> UpdateProviderBuilder {
        UpdateProviderBuilder::default()
    }

    fn read_http_request(&self, stream: &mut TcpStream) -> UResult<http::Request<String>> {
        let mut buffer = Vec::new();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        stream.read_to_end(&mut buffer)?;
        let mut request_infos = httparse::Request::new(&mut headers);
        let parse_result = request_infos.parse(&buffer)?;
        if parse_result.is_partial() {
            return Err("Incoming HTTP request is not complete".into());
        }
        let parse_result = parse_result.unwrap();
        let mut request = http::Request::builder()
            .method(request_infos.method.unwrap())
            .uri(request_infos.path.unwrap())
            .version(parse_http_version(request_infos.version.unwrap())?);
        for header in headers {
            if header != httparse::EMPTY_HEADER {
                request = request.header(header.name, header.value);
            }
        }
        let buffer: Vec<u8> = buffer.into_iter()
            .skip(parse_result)
            .collect();
        let request = request.body(String::from_utf8(buffer)?)?;
        Ok(request)
    }

    fn handle_stream(&self, mut stream: TcpStream) -> UResult {
        let response = http::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(r"{'result':'ok'}")
            .unwrap();
        let request = self.read_http_request(&mut stream)?;
        debug!(self.logger, "Received http data"; "data" => format!("{:#?}", request));
        write!(stream, "{}", response.body())?;
        debug!(self.logger, "Successfully responded");
        Ok(())
    }

    pub async fn listen(&self) -> UResult {
        for stream in self.tcp_handle.incoming() {
            debug!(self.logger, "Handling incoming request");
            if let Err(err) = stream {
                match err.kind() {
                    io::ErrorKind::WouldBlock => continue,
                    _ => {
                        error!(self.logger, "TCP stream error"; "reason" => err.to_string());
                        return Err(err.into());
                    }
                };
            }
            let stream = stream.unwrap();

            if let Err(why) = Self::handle_stream(self, stream) {
                error!(self.logger, "Error while handling a tcp stream"; "reason" => why.to_string());
                continue;
            };
            if self.is_stopped() {
                break;
            }
        }
        Ok(())
    }

    pub fn request_stop(&mut self) {
        self.stop_requested.store(false, Ordering::Relaxed)
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_requested.load(Ordering::Relaxed)
    }
}

#[derive(Default,Debug)]
pub struct UpdateProviderBuilder {
    tcp_listener: Option<TcpListener>,
    logger: Option<Logger>
}

impl UpdateProviderBuilder {
    pub fn listener(self, tcp_listener: TcpListener) -> Self {
        Self {
            tcp_listener: Some(tcp_listener),
            ..self
        }
    }

    pub fn logger(self, logger: Logger) -> Self {
        Self {
            logger: Some(logger),
            ..self
        }
    }

    pub fn build(self) -> UpdateProvider {
        UpdateProvider {
            tcp_handle: self.tcp_listener.unwrap(),
            stop_requested: false.into(),
            logger: self.logger.unwrap()
        }
    }
}
