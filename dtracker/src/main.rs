use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use dtracker::thread_pool::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    println!("Serving on http://127.0.0.1:8080"); // Use logger

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn create_response(buffer: &[u8]) -> String {
    let (status_line, filename) = if buffer.starts_with(b"GET") {
        ("HTTP/1.1 200 OK", "/dtracker/templates/get.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "/dtracker/templates/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    response
}

fn handle_read(stream: &mut TcpStream, buffer: &mut [u8]) {
    stream.read(buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..])); // Use logger
}

fn handle_write(mut stream: TcpStream, buffer: &[u8]) {
    stream.write_all(create_response(buffer).as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    handle_read(&mut stream, &mut buffer);
    handle_write(stream, &buffer);
}
