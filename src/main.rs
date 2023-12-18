use std::{
    io::{Read, Write},
    net::TcpListener,
};

const ADDRESS: &str = "127.0.0.1";
const PORT: i32 = 4221;

fn main() {
    let url = format!("{}:{}", ADDRESS, PORT);
    println!("Logs from your program will appear here!");
    println!("Listening on {}", url);

    let listener = TcpListener::bind(url).expect("failed to bind TCPListener to");

    for maybe_stream in listener.incoming() {
        match maybe_stream {
            Ok(mut stream) => {
                println!("Accepted connection");
                let mut req = String::new();
                let res = match stream.read_to_string(&mut req) {
                    Ok(_) => {
                        let lines: Vec<&str> = req.split("\r\n").collect();
                        let first_line: Vec<&str> = lines[0].split(" ").collect();
                        let maybe_path = first_line.iter().find(|s| s.starts_with("/")).cloned();
                        match maybe_path {
                            Some(path) => {
                                if path == "/" {
                                    "HTTP/1.1 200 OK\r\n\r\n"
                                } else {
                                    "HTTP/1.1 404 Not Found\r\n\r\n"
                                }
                            }
                            None => "HTTP/1.1 404 Not Found\r\n\r\n",
                        }
                    }
                    Err(_) => "HTTP/1.1 404 Not Found\r\n\r\n",
                };

                stream
                    .write_all(res.as_bytes())
                    .expect("Failed to respond to client");
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
