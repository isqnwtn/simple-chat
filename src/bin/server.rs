use simple_lib::{actor_impl::server_impl::init_central_controller};
use tokio::{io, net::TcpListener};

// The `#[tokio::main]` macro transforms the `main` function into an asynchronous one,
// setting up the Tokio runtime and executing the async code.
#[tokio::main]
async fn main() -> io::Result<()> {
    // let s = ServerHandler::new().await?;
    // s.main_loop().await?;
    _launch_server().await?;
    Ok(())
}

async fn _launch_server() -> io::Result<()> {
    let addr = std::env::var("SIMPLE_CHAT_ADDR").unwrap_or("127.0.0.1:7878".to_string());
    let listener = TcpListener::bind(addr).await?;
    let join_handle = init_central_controller(1024, listener).await;
    join_handle.await?;
    Ok(())
}
