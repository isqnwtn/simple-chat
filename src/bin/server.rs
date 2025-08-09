
use tokio::io;
use simple_lib::server_handler::ServerHandler;

// The `#[tokio::main]` macro transforms the `main` function into an asynchronous one,
// setting up the Tokio runtime and executing the async code.
#[tokio::main]
async fn main() -> io::Result<()> {
    let s = ServerHandler::new().await?;
    s.main_loop().await?;
    Ok(())
}
