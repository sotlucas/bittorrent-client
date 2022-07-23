use std::{net::TcpListener, sync::Arc};

use crate::{
    http_server::thread_pool::ThreadPool, tracker_request::request::Request,
    tracker_status::atomic_tracker_status::AtomicTrackerStatus,
};
pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
    status: Arc<AtomicTrackerStatus>,
}

impl Server {
    pub fn init(status: Arc<AtomicTrackerStatus>) -> std::io::Result<Server> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        Ok(Server {
            listener,
            pool: ThreadPool::new(4),
            status,
        })
    }

    pub fn serve(&self) -> std::io::Result<()> {
        println!("Serving on http://127.0.0.1:8080"); // Use logger

        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let mut request = Request::new(stream);
            self.pool.execute(move || {
                request.handle();
                // TODO: if error -> Log
            });
        }
        Ok(())
    }
}
