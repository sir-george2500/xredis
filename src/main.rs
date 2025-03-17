mod tcp_server;

use std::collections::HashMap;
use std::sync::Arc;
use tcp_server::handle_client;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("🚀 xRedis Lite Server running on port 6379...");

    let db = Arc::new(Mutex::new(HashMap::new())); // Use tokio::sync::Mutex

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = Arc::clone(&db); // Clone Arc for each client

        spawn(async move {
            handle_client(socket, db).await;
        });
    }
}
