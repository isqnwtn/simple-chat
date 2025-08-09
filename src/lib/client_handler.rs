use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// An async function to handle a single client connection.
// This function will be spawned as a "task" on the Tokio runtime.
pub async fn handle_client(mut stream: TcpStream) {
    // A buffer to store incoming data from the client.
    let mut buffer = [0; 512];

    // The peer_addr call is still synchronous.
    println!("Client connected from: {}", stream.peer_addr().unwrap());

    // Loop to continuously read from the client stream asynchronously.
    loop {
        // `read_size` is the number of bytes read.
        // `await` pauses the execution of this task until data is available.
        let read_size = match stream.read(&mut buffer).await {
            Ok(size) if size == 0 => {
                // If the size is 0, the client has disconnected.
                println!("Client disconnected from: {}", stream.peer_addr().unwrap());
                return;
            }
            Ok(size) => size,
            Err(e) => {
                // Handle potential read errors.
                eprintln!("An error occurred while reading: {}", e);
                return;
            }
        };

        // Convert the received bytes to a string and print it.
        let message = String::from_utf8_lossy(&buffer[..read_size]);
        println!("Received from client: {}", message);

        // Define a response message.
        let response = "Server received your message!";

        // Write the response back to the client asynchronously.
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("An error occurred while writing: {}", e);
            return;
        }
    }
}
