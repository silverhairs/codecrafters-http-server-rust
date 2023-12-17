use std::{io::Write, net::TcpListener};

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
                _stream
                    .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                    .expect("Failed to respond to client");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
