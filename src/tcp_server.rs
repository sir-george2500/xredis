use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use xredis::{parse_resp, RespMessage};

pub async fn handle_client(mut stream: TcpStream) {
    let mut buf = vec![0; 1024];
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) => return, // Client closed connection
            Ok(n) => n,
            Err(_) => return, // Handle read errors
        };
        let input = &buf[..n];
        println!("Received raw input: {:?}", input);

        match parse_resp(input) {
            Ok(RespMessage::SimpleString(cmd)) if cmd.to_uppercase() == "PING" => {
                let respond = RespMessage::SimpleString("PONG".to_string());
                stream
                    .write_all(format!("{}", respond).as_bytes())
                    .await
                    .unwrap();
            }
            Ok(RespMessage::Array(vec)) => {
                // We expect the array's first element to be a BulkString command.
                if let Some(RespMessage::BulkString(cmd_bytes)) = vec.get(0) {
                    let cmd = String::from_utf8_lossy(cmd_bytes).to_uppercase();
                    if cmd == "PING" {
                        let respond = RespMessage::SimpleString("PONG".to_string());
                        stream
                            .write_all(format!("{}", respond).as_bytes())
                            .await
                            .unwrap();
                    } else if cmd == "ECHO" && vec.len() > 1 {
                        if let RespMessage::BulkString(msg_bytes) = &vec[1] {
                            let response = RespMessage::BulkString(msg_bytes.clone());
                            stream
                                .write_all(response.to_string().as_bytes())
                                .await
                                .unwrap();
                        }
                    } else {
                        let error_resp = RespMessage::Error("ERR unknown command".to_string());
                        stream
                            .write_all(error_resp.to_string().as_bytes())
                            .await
                            .unwrap();
                    }
                } else {
                    let error_resp = RespMessage::Error("ERR invalid command format".to_string());
                    stream
                        .write_all(error_resp.to_string().as_bytes())
                        .await
                        .unwrap();
                }
            }
            _ => {
                let error_resp = RespMessage::Error("ERR unknown command".to_string());
                stream
                    .write_all(error_resp.to_string().as_bytes())
                    .await
                    .unwrap();
            }
        }
    }
}

