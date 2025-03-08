mod tcp_server;

use crate::tcp_server::handle_client;
use tokio::net::TcpListener;
use tokio::spawn;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("🚀 Redis Lite Server running on port 6379...");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        spawn(async move {
            handle_client(socket).await;
        });
    }
}

