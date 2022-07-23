use std::{net::TcpListener, sync::Arc};

use crate::{http_server::thread_pool::ThreadPool, tracker_request::request::Request, tracker_status::atomic_tracker_status::AtomicTrackerStatus};
pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
    status: Arc<AtomicTrackerStatus>,
}

impl Server {
    pub fn init(listener: TcpListener, pool: ThreadPool, status: Arc<AtomicTrackerStatus>) -> Server {
        Server {
            listener,
            pool,
            status,
        }
    }

    pub fn serve() -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        let pool = ThreadPool::new(4);

        println!("Serving on http://127.0.0.1:8080"); // Use logger

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let mut request = Request::new(stream);
            pool.execute(move || {
                request.handle();
            });
        }
        Ok(())
    }
}
