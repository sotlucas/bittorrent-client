use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::http_server::thread_pool::ThreadPool;

fn serve() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let pool = ThreadPool::new(4);

    println!("Serving on http://127.0.0.1:8080"); // Use logger

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    Ok(())
}

fn create_response(buffer: &[u8]) -> std::io::Result<String> {
    let (status_line, filename) = if buffer.starts_with(b"GET") {
        ("HTTP/1.1 200 OK", "/dtracker/templates/get.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "/dtracker/templates/404.html")
    };

    let contents = fs::read_to_string(filename)?;

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    Ok(response)
}

fn handle_read(stream: &mut TcpStream, buffer: &mut [u8]) -> std::io::Result<()> {
    match stream.read(buffer) {
        Ok(_) => {
            println!("Request: {}", String::from_utf8_lossy(buffer));
            Ok(())
        } // Use logger
        Err(e) => Err(e),
    }
}

fn handle_write(mut stream: TcpStream, buffer: &[u8]) -> std::io::Result<()> {
    stream
        .write_all(create_response(buffer)?.as_bytes())
        .unwrap();
    stream.flush().unwrap();

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    if handle_read(&mut stream, &mut buffer).is_ok() {
        _ = handle_write(stream, &buffer);
    }
}
