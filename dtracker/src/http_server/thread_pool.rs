use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use super::worker::{Message, Worker};

/// Struct that represents a thread pool that spawns a specified number of worker threads and allows to process connections concurrently.
/// Each idle thread in the pool is available to handle a task. 
/// When a thread is done processing its task, it is returned to the pool of idle threads, ready to handle a new task.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    /// Creates a new ThreadPool with a given size.
    /// The size is the number of threads in the pool.
    /// If the size is zero or a negative number, the `new` function will panic.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Receives a closure and assigns it to a thread in the pool to run.
    pub fn execute<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(closure);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers."); // Use logger

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers."); // Use logger

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id); // Use logger
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
