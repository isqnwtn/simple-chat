use std::io::{self, Read, Write};
use std::net::TcpStream;

const DEFAULT_ADDR: &'static str = "127.0.0.1";
const DEFAULT_PORT: &'static str = "7878";

fn main() {
    // Attempt to connect to the server running on the same machine.
    let addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let mut stream = match TcpStream::connect(format!("{}:{}", addr, port)) {
        Ok(stream) => {
            println!("Successfully connected to the server!");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // Loop to allow the user to send multiple messages.
    loop {
        // Read user input from the console.
        let mut user_input = String::new();
        println!("Enter a message to send to the server (or 'exit' to quit):");
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        // Trim whitespace from the input.
        let trimmed_input = user_input.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        // Check for the exit command.
        if trimmed_input.eq_ignore_ascii_case("exit") {
            println!("Disconnecting from the server...");
            break;
        }

        // Write the message to the server.
        // `write_all` ensures the entire message is sent.
        if let Err(e) = stream.write_all(trimmed_input.as_bytes()) {
            eprintln!("Failed to write to server: {}", e);
            break;
        }

        // A buffer to store the server's response.
        let mut buffer = [0; 512];
        let read_size = match stream.read(&mut buffer) {
            Ok(size) if size > 0 => size,
            _ => {
                eprintln!("Server disconnected or no response received.");
                break;
            }
        };

        // Convert the response to a string and print it.
        let response = String::from_utf8_lossy(&buffer[..read_size]);
        println!("Received from server: {}", response);
    }
}
