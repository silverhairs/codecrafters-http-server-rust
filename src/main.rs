use std::{
    io::{Read, Write},
    net::TcpListener,
};

use regex::Regex;

const ADDRESS: &str = "127.0.0.1";
const PORT: i32 = 4221;

fn main() {
    let url = format!("{}:{}", ADDRESS, PORT);
    println!("Logs from your program will appear here!");
    println!("Listening on {}", url);

    let listener = TcpListener::bind(url).expect("failed to bind TCPListener to");

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("Accepted connection");
                let mut res = String::new();
                let mut req = String::new();
                match _stream.read_to_string(&mut req) {
                    Ok(_) => {
                        let pattern = Regex::new(r"^\/[\w\-\/]*$").unwrap();
                        match req.split("\r\n").find(|s| pattern.is_match(s)) {
                            Some(path) => {
                                if path == "/" {
                                    res = "HTTP/1.1 200 OK\r\n\r\n".to_string();
                                }
                            }
                            None => {
                                res = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to read request {}", e);
                    }
                };
                _stream
                    .write_all(res.as_bytes())
                    .expect("Failed to respond to client");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
