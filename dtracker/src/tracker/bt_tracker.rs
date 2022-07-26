use logger::{logger_receiver::Logger, logger_error::LoggerError};
use std::sync::Arc;
use std::io;

use crate::{
    http_server::server::Server, tracker_status::atomic_tracker_status::AtomicTrackerStatus,
};

#[derive(Debug)]
pub enum BtTrackerError {
    LoggerInitError(LoggerError),
    CreatingServerError(io::Error),
    StartingServerError(io::Error),
}

pub struct BtTracker {
    logger: Logger,
    tracker_status: Arc<AtomicTrackerStatus>,
    server: Server,
}

impl BtTracker {
    pub fn init() -> Result<Self, BtTrackerError> {
        let logger = Logger::new("./logs", 1000000).map_err(|err| BtTrackerError::LoggerInitError(err))?; // TODO: Sacar de configs
        let logger_sender = logger.new_sender();

        let tracker_status = Arc::new(AtomicTrackerStatus::default());

        let server = Server::init(tracker_status.clone(), logger_sender).map_err(|err| BtTrackerError::CreatingServerError(err))?;

        Ok(Self {
            logger,
            tracker_status,
            server,
        })
    }

    pub fn run(&self) -> Result<(), BtTrackerError> {
        self.server.serve().map_err(|err| BtTrackerError::StartingServerError(err))
    }
}
