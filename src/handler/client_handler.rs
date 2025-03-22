use crate::handler::commands::{handle_array_command, handle_simple_string};
use crate::resp::resp_protocol::{parse_resp, RespMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::value::ValueWithExpiry;

pub type Db = Arc<Mutex<HashMap<String, ValueWithExpiry>>>;

pub async fn handle_client(mut stream: TcpStream, db: Db) {
    let mut buf = vec![0; 1024];

    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            return;
        }

        let input = &buf[..n];
        let response = match parse_resp(input) {
            Ok(RespMessage::SimpleString(cmd)) => handle_simple_string(cmd),
            Ok(RespMessage::Array(vec)) => handle_array_command(vec, &db).await,
            _ => RespMessage::Error("ERR unknown command".to_string()),
        };

        if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
            eprintln!("Failed to write response: {}", e);
            return;
        }
    }
}
