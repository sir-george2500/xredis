use crate::handler::client_handler::Db;
use crate::resp::resp_protocol::RespMessage;
pub fn handle_simple_string(cmd: String) -> RespMessage {
    if cmd.to_uppercase() == "PING" {
        RespMessage::SimpleString("PONG".to_string())
    } else {
        RespMessage::Error("ERR unknown command".to_string())
    }
}

pub async fn handle_array_command(vec: Vec<RespMessage>, db: &Db) -> RespMessage {
    if let Some(RespMessage::BulkString(cmd_bytes)) = vec.get(0) {
        let cmd = String::from_utf8_lossy(cmd_bytes).to_uppercase();

        match cmd.as_str() {
            "PING" => RespMessage::SimpleString("PONG".to_string()),

            "ECHO" if vec.len() > 1 => {
                if let RespMessage::BulkString(msg_bytes) = &vec[1] {
                    RespMessage::BulkString(msg_bytes.clone())
                } else {
                    RespMessage::Error("ERR invalid ECHO argument".to_string())
                }
            }

            "SET" if vec.len() > 2 => {
                if let (RespMessage::BulkString(key_bytes), RespMessage::BulkString(value_bytes)) =
                    (&vec[1], &vec[2])
                {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let value = String::from_utf8_lossy(value_bytes).to_string();
                    db.lock().await.insert(key, value);
                    RespMessage::SimpleString("OK".to_string())
                } else {
                    RespMessage::Error("ERR invalid SET arguments".to_string())
                }
            }

            "GET" if vec.len() > 1 => {
                if let RespMessage::BulkString(key_bytes) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    // Use Option to handle the absence of a key
                    let response: Option<RespMessage> =
                        if let Some(value) = db.lock().await.get(&key) {
                            Some(RespMessage::BulkString(value.as_bytes().to_vec()))
                        } else {
                            // If the key doesn't exist, return a detailed error message
                            None
                        };

                    // If response is None, return an error with a custom message
                    response.unwrap_or_else(|| {
                        RespMessage::Error(format!("ERR key '{}' not found", key))
                    })
                } else {
                    RespMessage::Error("ERR invalid GET argument. Expected key".to_string())
                }
            }

            _ => RespMessage::Error("ERR unknown command".to_string()),
        }
    } else {
        RespMessage::Error("ERR invalid command format".to_string())
    }
}
