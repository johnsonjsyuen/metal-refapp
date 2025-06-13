use std::{env, thread};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::Thread;
use std::time::{SystemTime, UNIX_EPOCH};

fn handle_client(mut stream: TcpStream, ok_text: Arc<String>) {
    let ok = format!("HTTP/1.1 200 {}\r\n\r\n", ok_text);
    let ok_response = ok.as_str();
    const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    const INTERNAL_SERVER_ERROR_RESPONSE: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

    let peer_addr_str = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown".to_string(),
    };

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    let response = match request_line {
        line if line.starts_with("GET /healthcheck")
            || line.starts_with("GET /ok")
            || line.starts_with("GET /heartbeat") =>
        {
            println!("Health check request processed from {}: {}", peer_addr_str, request_line);
            ok_response
        }
        line if line.starts_with("GET /failing-deepcheck") => {
            println!("Failing deepcheck request processed from {}: {}", peer_addr_str, request_line);
            INTERNAL_SERVER_ERROR_RESPONSE
        }
        line if line.starts_with("GET /flakey-deepcheck") => {
            let now_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            if now_millis % 100 < 20 {
                // Roughly 20% chance
                println!(
                    "Flakey deepcheck (FAILING) request processed from {}: {}",
                    peer_addr_str, request_line
                );
                INTERNAL_SERVER_ERROR_RESPONSE
            } else {
                println!(
                    "Flakey deepcheck (OK) request processed from {}: {}",
                    peer_addr_str, request_line
                );
                ok_response
            }
        }
        _ => NOT_FOUND_RESPONSE,
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let ok_text = Arc::new(env::var("OK_TEXT").unwrap_or_else(|_| String::from("OK")));
    let listen_address =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1:8080"));

    let listener = TcpListener::bind(&listen_address).unwrap();
    println!("Server listening on port {}", listen_address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let ok_text = Arc::clone(&ok_text);
                thread::spawn(move || {
                    handle_client(stream, ok_text);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}