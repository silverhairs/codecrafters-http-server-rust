use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::Arc,
};

use itertools::Itertools;

const ADDRESS: &str = "127.0.0.1";
const PORT: i32 = 4221;

fn main() {
    let url = format!("{}:{}", ADDRESS, PORT);
    println!("Logs from your program will appear here!");
    println!("Listening on {}", url);

    let shared_dir = Arc::new(get_dir());

    let listener = TcpListener::bind(url).expect("failed to bind TCPListener to");

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let maybe_dir = Arc::clone(&shared_dir);
                let dir = match maybe_dir.as_ref() {
                    Some(path) => path.to_string(),
                    None => "".to_string(),
                };

                std::thread::spawn(move || {
                    println!("Accepted connection");
                    let mut reader = BufReader::new(&mut stream);
                    let received = reader.fill_buf().unwrap().to_vec();
                    reader.consume(received.len());
                    let req = String::from_utf8_lossy(&received);
                    let lines: Vec<&str> = req.split("\r\n").collect();
                    let first_line: &Vec<&str> = &lines[0].split(" ").collect();
                    let maybe_path = first_line.iter().find(|s| s.starts_with("/"));
                    let res = match maybe_path {
                        Some(path) => {
                            if path.starts_with("/files/") {
                                match path.strip_prefix("/files/") {
                                    Some(file_name) => {
                                        match handle_file_request(&file_name.to_string(), dir) {
                                            Some(body) => {
                                                format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
                                            }
                                            None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                                        }
                                    }
                                    None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                                }
                            } else if path.starts_with("/echo/") {
                                let msg = match path.strip_prefix("/echo/") {
                                    Some(body) => body,
                                    None => "",
                                };
                                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", msg.len(), msg)
                            } else if path.eq(&"/user-agent") {
                                let user_agent = find_user_agent(lines);
                                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent)
                            } else if path.eq(&"/") {
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
                });
            }
            Err(e) => {
                println!("error: {}", e)
            }
        }
    }
}

fn find_user_agent(lines: Vec<&str>) -> &str {
    let maybe_line = lines.iter().find(|line| line.contains("User-Agent"));
    match maybe_line {
        Some(line) => &line.split(" ").last().expect("No user agent passed"),
        None => "",
    }
}

fn handle_file_request(file_name: &String, dir_name: String) -> Option<String> {
    if file_name.starts_with("..") || file_name.starts_with("~") {
        return None;
    }

    let path = format!("{}{}", dir_name, file_name);
    println!("Opening file: {}", path);
    return match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let received = reader.fill_buf().expect("failed to read file").to_vec();
            reader.consume(received.len());
            let content = String::from_utf8(received);
            return match content {
                Ok(data) => Some(data),
                Err(_) => None,
            };
        }
        Err(_) => None,
    };
}

fn get_dir() -> Option<String> {
    let args = env::args().collect_vec();
    if args.len() < 3 {
        return None;
    }
    if args[1].eq("--directory") {
        return Some(args[2].clone());
    }
    return None;
}
