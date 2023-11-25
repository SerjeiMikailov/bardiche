#![cfg_attr(feature = "nightly", feature(drain_filter))]

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

const THREAD_POOL_SIZE: usize = 4;

fn handle_client(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Hello, Rust!</h1></body></html>";

    let _ = stream.write(response.as_bytes());
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");

    println!("Server running at http://127.0.0.1:8080");

    let pool = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool = Arc::clone(&pool);
                let handle = thread::spawn(move || {
                    handle_client(stream);
                });

                let mut pool = pool.lock().expect("Mutex lock failed");
                pool.push(handle);
                #[cfg(feature = "nightly")]
                pool.drain_filter(|t| t.join().is_err()).for_each(|_| {});
                if pool.len() >= THREAD_POOL_SIZE {
                    if let Some(handle) = pool.pop() {
                        handle.join().expect("Thread join failed");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
