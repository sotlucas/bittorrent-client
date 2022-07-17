use std::{thread, sync::{Arc, Mutex, mpsc::Receiver}};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub struct Worker {
    // TODO: solve public attributes
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id); // Use logger
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id); // use logger
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
