use std::net::TcpListener;

const ADDRESS: &str = "127.0.0.1";
const PORT: i32 = 4221;

fn main() {
    println!("Logs from your program will appear here!");

    let listener =
        TcpListener::bind(format!("{}:{}", ADDRESS, PORT)).expect("failed to bind TCPListener to");

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
