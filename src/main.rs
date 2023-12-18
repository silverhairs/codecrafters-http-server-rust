use std::{
    io::{BufRead, BufReader, Write},
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
                let mut reader = BufReader::new(&mut stream);
                let received = reader.fill_buf().unwrap().to_vec();
                reader.consume(received.len());
                let req = String::from_utf8_lossy(&received);
                let lines: Vec<&str> = req.split("\r\n").collect();
                let first_line: &Vec<&str> = &lines[0].split(" ").collect();
                let maybe_path = first_line.iter().find(|s| s.starts_with("/")).cloned();
                let res = match maybe_path {
                    Some(path) => {
                        if path.starts_with("/echo/") {
                            let msg = match path.strip_prefix("/echo/") {
                                Some(body) => body,
                                None => "",
                            };
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", msg.len(), msg)
                        } else if path == "/user-agent" {
                            let user_agent = find_user_agent(lines);
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent)
                        } else if path == "/" {
                            "HTTP/1.1 200 OK\r\n\r\n".to_string()
                        } else {
                            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                        }
                    }
                    None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                };
                println!("{}", res);
                stream.write_all(res.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn find_user_agent(lines: Vec<&str>) -> &str {
    let maybe_line = lines.iter().find(|line| line.contains("User-Agent"));
    match maybe_line {
        Some(line) => &line.split(" ").last().unwrap(),
        None => "",
    }
}
