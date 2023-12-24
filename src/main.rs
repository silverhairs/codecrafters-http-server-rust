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
const HTTP_GET: &str = "GET";
const HTTP_POST: &str = "POST";

fn main() {
    let url = format!("{}:{}", ADDRESS, PORT);
    println!("Logs from your program will appear here!");
    println!("Listening on {}", url);

    let listener = TcpListener::bind(url).expect("failed to bind TCPListener to");
    let shared_dir = Arc::new(get_dir());

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let dir = match Arc::clone(&shared_dir).as_ref() {
                    Some(path) => path.to_string(),
                    None => "".to_string(),
                };

                std::thread::spawn(move || {
                    println!("Accepted connection");
                    let mut reader = BufReader::new(&mut stream);
                    let received = reader.fill_buf().unwrap().to_vec();
                    reader.consume(received.len());
                    let req = String::from_utf8_lossy(&received);
                    let res = on_request(&req, dir);
                    println!("{}", res);
                    stream
                        .write_all(res.as_bytes())
                        .expect("failed to write response");
                    stream.flush().unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn on_request(req: &str, dir: String) -> String {
    let lines: Vec<&str> = req.split("\r\n").collect();
    let first_line: &Vec<&str> = &lines[0].split(" ").collect();
    let method = first_line[0];
    let maybe_path = first_line.iter().find(|s| s.starts_with("/"));
    return match maybe_path {
        Some(path) => {
            if path.starts_with("/files/") {
                match path.strip_prefix("/files/") {
                    Some(file_name) => {
                        if method.eq(HTTP_GET) {
                            match get_file_content(file_name, dir) {
                                Some(body) => {
                                    return format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
                                }

                                None => return "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                            }
                        } else if method.eq(HTTP_POST) {
                            let path = format!("{}{}", dir, file_name);
                            println!("Creating file at {}", path);
                            let mut file = File::create(path).expect("Failed to create file");
                            let content = lines.last().expect("failed to extract content");
                            println!("File content: =>{}", content);
                            file.write_all(content.as_bytes())
                                .expect("failed to write data to file");
                            file.flush().unwrap();
                            return format!(
                                "HTTP/1.1 201 OK\r\nContent-Type: application/octet-stream\r\n\r\n: {}",
                                lines.last().unwrap().len()
                            );
                        }
                        return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
                    }
                    None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                }
            } else if path.starts_with("/echo/") {
                let msg = match path.strip_prefix("/echo/") {
                    Some(body) => body,
                    None => "",
                };
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    msg.len(),
                    msg
                )
            } else if path.eq(&"/user-agent") {
                let user_agent = find_header_value(lines, "User-Agent");
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    user_agent.len(),
                    user_agent
                )
            } else if path.eq(&"/") {
                "HTTP/1.1 200 OK\r\n\r\n".to_string()
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
            }
        }
        None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };
}

fn find_header_value(lines: Vec<&str>, header_key: &str) -> String {
    let maybe_line = lines.iter().find(|line| line.contains(header_key));
    match maybe_line {
        Some(line) => {
            let val = line.replace(format!("{}:", header_key).as_str(), "");
            return val.trim().to_string();
        }
        None => "".to_string(),
    }
}

fn get_file_content(file_name: &str, dir_name: String) -> Option<String> {
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
