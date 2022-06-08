use super::http::http_handler::{HttpHandler, HttpHandlerError};
use super::http::query_params::QueryParams;
use super::http::url_parser::{ConnectionProtocol, TrackerUrl, TrackerUrlError};
use super::tracker_response::FromTrackerResponseError;
use crate::torrent_parser::torrent::Torrent;
use crate::tracker::tracker_response::TrackerResponse;

/// `TrackerHandler` struct for communicating to a bt tracker.
///
/// To create a new `TrackerHandler` use the method builder `new()`.
///
/// To get the tracker's peer list use the method `get_peer_list()`.
#[derive(Debug)]
pub struct TrackerHandler {
    pub torrent: Torrent,
    pub tracker_url: TrackerUrl,
    pub client_port: u32,
}
/// Posible `TrackerHandler` errors.
#[derive(Debug)]
pub enum TrackerHandlerError {
    HttpHandlerError(HttpHandlerError),
    FromTrackerResponseError(FromTrackerResponseError),
    UrlParseError(TrackerUrlError),
}

impl TrackerHandler {
    /// Builds a new `TrackerHandler` from a **Torrent** and a **client_port** passed by paramaters.
    ///
    /// It returns an `TrackerHandlerError` if:
    /// - There was an error parsing the torrent's announce_url.
    pub fn new(torrent: Torrent, client_port: u32) -> Result<Self, TrackerHandlerError> {
        let tracker_url = match TrackerUrl::parse(torrent.announce_url.as_str()) {
            Ok(url) => url,
            Err(err) => return Err(TrackerHandlerError::UrlParseError(err)),
        };

        Ok(Self {
            torrent,
            tracker_url,
            client_port,
        })
    }

    /// Gets the tracker's peers list.
    ///
    /// On success it returns a `TrackerResponse` struct cointaining the tracker's response.
    ///
    /// It returns an `TrackerHandlerError` if:
    /// - There was a problem writing to the tracker.
    /// - There was a problem reading the tracker's response.
    /// - There was a problem decoding the parser response.
    pub fn get_peers_list(&self) -> Result<TrackerResponse, TrackerHandlerError> {
        let query_params = QueryParams::new(
            self.torrent.info_hash.clone(),
            self.client_port,
            self.torrent.info.length,
        );

        let http_handler = HttpHandler::new(self.tracker_url.clone(), query_params);

        let response = if self.tracker_url.protocol == ConnectionProtocol::Https {
            match http_handler.https_request() {
                Ok(response) => response,
                Err(err) => return Err(TrackerHandlerError::HttpHandlerError(err)),
            }
        } else {
            match http_handler.http_request() {
                Ok(response) => response,
                Err(err) => return Err(TrackerHandlerError::HttpHandlerError(err)),
            }
        };
        match TrackerResponse::from(response) {
            Ok(tracker_response) => Ok(tracker_response),
            Err(err) => Err(TrackerHandlerError::FromTrackerResponseError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::torrent_parser::info::Info;

    use super::*;

    #[test]
    fn test_get_peers_list() {
        let torrent = create_test_torrent(
            "https://torrent.ubuntu.com:443/announce",
            "2c6b6858d61da9543d4231a71db4b1c9264b0685",
        );
        let test_port = 6969;

        let tracker_handler = TrackerHandler::new(torrent, test_port).unwrap();

        assert!(tracker_handler.get_peers_list().unwrap().peers.len() > 0);
    }

    // Test con http funciona, solo que faltaría parsear cuando vienen los peers como string de bytes en vez de dict.

    // #[test]
    // fn test_http_request() {
    //     let torrent = create_test_torrent(
    //         "http://vps02.net.orel.ru/announce",
    //         "f834824904be1854c89ba007c01678ff797f8dc7",
    //     );
    //     let test_port = 6969;

    //     let tracker_handler = TrackerHandler::new(torrent, test_port);

    //     let actual_res = tracker_handler.get_peers_list();
    //     println!("{:?}", actual_res);
    // }

    // Auxiliar

    fn create_test_torrent(announce: &str, info_hash: &str) -> Torrent {
        let info = Info {
            length: 100,
            name: "test".to_string(),
            piece_length: 100,
            pieces: vec![],
        };

        Torrent {
            announce_url: announce.to_string(),
            info,
            info_hash: info_hash.to_string(),
        }
    }
}
