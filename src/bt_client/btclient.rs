use crate::{
    bt_client::btclient_error::BtClientError, bt_client::error_message::ErrorMessage,
    config::cfg::Cfg, logger::logger_receiver::Logger, logger::logger_sender::LoggerSender,
    torrent_handler::handler::TorrentHandler, torrent_parser::parser::TorrentParser,
    torrent_parser::torrent::Torrent,
};

use super::{statistics::Runner, statistics::Statistics};

use gtk::glib;
use std::{
    fs, io,
    thread::{self, JoinHandle},
};

const CONFIG_FILE_PATH: &str = "config.cfg";

/**
Represents the BitTorrent client application.

It holds the code for initializing the client, and for starting the torrent downloading process.

```rust
# use bit_torrent_rustico::bt_client::btclient_error::BtClientError;
# use std::fs::remove_dir_all;
# use gtk::glib;
use bit_torrent_rustico::bt_client::btclient::BtClient;

# fn main() -> Result<(), BtClientError> {
let torrents_directory = String::from("./logs");
let bt_client = BtClient::init(torrents_directory)?;
let (sender, _receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

let output = bt_client.run(sender);
# remove_dir_all("./logs").unwrap();
# Ok(())
# }
```
*/
pub struct BtClient {
    config: Cfg,
    logger: Logger,
    torrents: Vec<Torrent>,
}

impl BtClient {
    /**
    Method for initializing the BitTorrent client application.

    Recieves a path to a directory containing the .torrent files to download.

    It reads the configuration file (./config.cfg), starts a Logger writing to the folder indicated by that configuration file, and then attempts to parse the torrent files placed inside the provided torrents directory.

    The corrently parsed torrents are stored inside the BtClient struct, and will begin downloading when the '.run()' method is called.
    */
    pub fn init(torrents_directory: String) -> Result<Self, BtClientError> {
        let config = Self::read_configuration_file(CONFIG_FILE_PATH)?;
        let logger = Logger::new(&config.log_directory, config.max_log_file_kb_size * 1000)?;

        let logger_sender = logger.new_sender();
        logger_sender.info("Initializing client...");
        logger_sender.info("Configuration file loaded correctly.");

        let torrents = Self::parse_torrents_in_directory(logger_sender, torrents_directory)?;

        Ok(Self {
            config,
            logger,
            torrents,
        })
    }

    /// Method for starting the torrent downloading process.
    pub fn run(&self, sender: glib::Sender<Vec<Statistics>>) {
        let logger = self.logger.new_sender();
        logger.info("Starting client...");

        let mut handler_status_list = Vec::new();
        let mut torrent_handlers_joins = Vec::new();
        self.torrents.iter().for_each(|torrent| {
            let handler = TorrentHandler::new(torrent.clone(), self.config.clone(), logger.clone());
            handler_status_list.push(handler.status());
            let thread_handle = self.spawn_torrent_handler(torrent, handler);
            match thread_handle {
                Ok(handle) => {
                    torrent_handlers_joins.push(handle);
                }
                Err(error) => {
                    let error_message = format!("An error occurred while trying to spawn a new thread for a torrent_handler: {:?}", error);
                    logger.error(&error_message);
                    handler_status_list.pop();
                }
            }
        });

        let runner = Runner::new(handler_status_list, sender);
        let _jh = self.spawn_statistics_runner(runner);

        self.join_handles(torrent_handlers_joins);
    }

    fn spawn_torrent_handler(
        &self,
        torrent: &Torrent,
        mut torrent_handler: TorrentHandler,
    ) -> Result<JoinHandle<()>, io::Error> {
        let logger = self.logger.new_sender();

        let builder = thread::Builder::new().name(format!("Torrent handler: {}", torrent.name()));
        builder.spawn(move || {
            if let Err(torrent_error) = torrent_handler.handle() {
                logger.error(&format!("{:?}", torrent_error));
            }
        })
    }

    fn spawn_statistics_runner(&self, runner: Runner) -> Result<JoinHandle<()>, io::Error> {
        let logger = self.logger.new_sender();

        let builder = thread::Builder::new().name("Torrent statistics".to_string());
        builder.spawn(move || {
            if let Err(runner_error) = runner.run() {
                logger.error(&format!("{:?}", runner_error));
            }
        })
    }

    fn join_handles(&self, torrent_handlers: Vec<JoinHandle<()>>) {
        torrent_handlers.into_iter().for_each(|torrent_handler| {
            if torrent_handler.join().is_err() {
                self.logger
                    .new_sender()
                    .error("An error occurred while attempting to join a torrent_handler thread.");
            };
        });
    }

    fn read_configuration_file(filename: &str) -> Result<Cfg, BtClientError> {
        match Cfg::new(filename) {
            Ok(config) => Ok(config),
            Err(io_error) => {
                let message = format!("Couldn't read configuration file: {}", io_error);
                let config_error =
                    BtClientError::ConfigurationFileError(ErrorMessage::new(message));
                Err(config_error)
            }
        }
    }

    fn parse_torrents_in_directory(
        log_sender: LoggerSender,
        torrents_directory: String,
    ) -> Result<Vec<Torrent>, BtClientError> {
        let torrents: Vec<Torrent> =
            Self::list_torrent_filenames_in_directory(&log_sender, torrents_directory.clone())?
                .iter()
                .filter_map(|filename| {
                    Self::parse_torrent(
                        &log_sender,
                        &format!("{}/{}", torrents_directory, filename),
                    )
                })
                .collect();

        Ok(torrents)
    }

    fn parse_torrent(log_sender: &LoggerSender, torrent_filename: &str) -> Option<Torrent> {
        match TorrentParser::parse(torrent_filename.to_string()) {
            Ok(parsed_torrent) => {
                log_sender.info(&format!("Torrent {} parsed correctly.", torrent_filename));
                Some(parsed_torrent)
            }
            Err(error) => {
                log_sender.warn(&format!(
                    "Couldn't parse torrent file {}: {:?}",
                    torrent_filename, error
                ));
                None
            }
        }
    }

    fn list_torrent_filenames_in_directory(
        log_sender: &LoggerSender,
        directory: String,
    ) -> Result<Vec<String>, BtClientError> {
        let filenames = Self::open_directory(log_sender, directory)?
            .flatten()
            .flat_map(|dir_entry| dir_entry.file_name().into_string())
            .filter(|filename| filename.ends_with(".torrent"))
            .collect();

        Ok(filenames)
    }

    fn open_directory(
        log_sender: &LoggerSender,
        directory: String,
    ) -> Result<fs::ReadDir, BtClientError> {
        match fs::read_dir(directory) {
            Ok(dir) => Ok(dir),
            Err(error) => {
                let directory_error = BtClientError::TorrentDirectoryError(ErrorMessage::new(
                    format!("Failed to read the given torrents directory: {}", error),
                ));
                log_sender.error(&format!("{:?}", directory_error));
                Err(directory_error)
            }
        }
    }
}
