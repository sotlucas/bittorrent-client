use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};

use crate::{
    tracker_status::atomic_tracker_status::AtomicTrackerStatus,
    tracker_peer::peer::Peer,
    announce::announce_response::AnnounceResponse,
    http::{http_method::HttpMethod, http_parser::Http, http_status::HttpStatus},
};

pub struct Request {
    pub stream: TcpStream,
    status: Arc<AtomicTrackerStatus>
}

#[derive(Debug)]
pub enum RequestError {
    InvalidEndpointError,
    ParseHttpError,
}

impl Request {
    pub fn new(stream: TcpStream, 
        status: Arc<AtomicTrackerStatus>
    ) -> Request {
        Request { stream, status }
    }

    pub fn handle(&mut self) -> Result<(), RequestError> {
        let mut buf = [0; 1024];
        self.stream.read(&mut buf).unwrap();

        // TODO: should match and send error (400 BAD REQUEST) through stream before returning error
        let http_request = Http::parse(&buf).map_err(|_| RequestError::ParseHttpError)?;

        let (status_line, response) = if http_request.method.eq(&HttpMethod::Get) {
            let response = match http_request.endpoint.as_str() {
                "/announce" => self.handle_announce(http_request),
                "/stats" => self.handle_stats(http_request),
                _ => return Err(RequestError::InvalidEndpointError),
            };
            (HttpStatus::Ok, response)
        } else {
            (HttpStatus::NotFound, "".to_string())
        };

        self.send_response(response.as_str(), status_line).unwrap();

        Ok(())
    }

    fn handle_announce(&self, http_request: Http) -> String {
        AnnounceResponse::from(http_request.params);
        String::from("announce")
    }

    /// Receives a `since` param that represents the period for statistics in hours. 
    fn handle_stats(&self, http_request: Http) -> String {
        let since = http_request.params.get("since").unwrap();
        let since_hours = chrono::Duration::hours(since.parse::<i64>().unwrap());
        let timestamp_limit = chrono::Local::now() - since_hours;

        // Obtener cantidades de peers conectados, seeders, leechers y torrents
        let torrents = self.status.get_torrents();
        let peers: Vec<Peer> = torrents.values().map(|peer_status| peer_status.peers.clone()).collect();

        
        let pepe: Vec<Peer> = peers.iter().filter(|peer| std::cmp::Ordering::is_lt(timestamp_limit.cmp(&peer.status.last_seen))).collect();

        // Distribuir en "buckets" de a minutos / horas 

        // Armar string JSON
        String::from("stats")

    }

    fn create_response(contents: &str, status_line: HttpStatus) -> std::io::Result<String> {
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            status_line.to_string(),
            contents.len(),
            contents
        ); 

        Ok(response)
    }

    fn send_response(&mut self, contents: &str, status_line: HttpStatus) -> std::io::Result<()> {
        let response = Self::create_response(contents, status_line)?;

        self.stream
            .write(response.as_bytes())
            .unwrap();
        self.stream.flush().unwrap();

        Ok(())
    }
}
