use std::{net::TcpStream, io::Read};

use crate::http::http_parser::{Http, HttpError};

pub struct Request {
    pub stream: TcpStream,
}

pub enum RequestError {
    InvalidEndpointError,
    ParseHttpError   
}

impl Request {
    pub fn new(stream: TcpStream) -> Request {
        Request { stream }
    }

    pub fn handle(&self) -> Result<(), RequestError> {
        let mut buf = vec![];
        let body = self.stream.read_to_end(&mut buf).unwrap();

        let http_request = Http::parse(&buf).map_err(|_| RequestError::ParseHttpError)?;
        // TODO: Validate method in request
        match http_request.endpoint.as_str() {
            "/announce" => {
                self.handle_announce(http_request);
            }
            "/stats" => {
                self.handle_stats(http_request);
            }
            _ => return Err(RequestError::InvalidEndpointError),
        };
        Ok(())
    }

    fn handle_announce(&self, http_request: Http) {
        let announce_response = AnnounceResponse::from(http_request.params);
    }

    fn handle_stats(&self, http_request: Http) {
        let stats_response = Stats::from(http_request.params);
        self.send_response(response);
    }

    fn send_response(&self, response: String) {
        stream
            .write_all(create_response(buffer)?.as_bytes())
            .unwrap();
        stream.flush().unwrap();
    }
}
