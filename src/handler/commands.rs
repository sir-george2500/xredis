use std::time::{SystemTime, UNIX_EPOCH};

use crate::handler::client_handler::Db;
use crate::handler::value::ValueWithExpiry;
use crate::resp::resp_protocol::RespMessage;

pub fn handle_simple_string(cmd: String) -> RespMessage {
    if cmd.to_uppercase() == "PING" {
        RespMessage::SimpleString("PONG".to_string())
    } else {
        RespMessage::Error("ERR unknown command".to_string())
    }
}

pub async fn handle_array_command(vec: Vec<RespMessage>, db: &Db) -> RespMessage {
    if let Some(RespMessage::BulkString(Some(cmd_bytes))) = vec.get(0) {
        let cmd = String::from_utf8_lossy(cmd_bytes).to_uppercase();

        match cmd.as_str() {
            "PING" => RespMessage::SimpleString("PONG".to_string()),

            "ECHO" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(msg_bytes)) = &vec[1] {
                    RespMessage::BulkString(Some(msg_bytes.clone()))
                } else {
                    RespMessage::Error("ERR invalid ECHO argument".to_string())
                }
            }

            "SET" if vec.len() >= 3 => {
                if let (
                    RespMessage::BulkString(Some(key_bytes)),
                    RespMessage::BulkString(Some(value_bytes)),
                ) = (&vec[1], &vec[2])
                {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let value = String::from_utf8_lossy(value_bytes).to_string();
                    let mut expiry: Option<u128> = None;
                    let mut i = 3;

                    while i < vec.len() {
                        if let RespMessage::BulkString(Some(opt_bytes)) = &vec[i] {
                            let opt = String::from_utf8_lossy(opt_bytes).to_uppercase();

                            if i + 1 >= vec.len() {
                                return RespMessage::Error("ERR invalid SET arguments".to_string());
                            }

                            match opt.as_str() {
                                "EX" => {
                                    if let RespMessage::BulkString(Some(seconds)) = &vec[i + 1] {
                                        if let Ok(secs) =
                                            String::from_utf8_lossy(seconds).parse::<u64>()
                                        {
                                            let now = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis();
                                            expiry = Some(now + (secs as u128 * 1000));
                                            i += 2;
                                        } else {
                                            return RespMessage::Error(
                                                "ERR invalid expire time".to_string(),
                                            );
                                        }
                                    }
                                }
                                "PX" => {
                                    if let RespMessage::BulkString(Some(millis)) = &vec[i + 1] {
                                        if let Ok(ms) =
                                            String::from_utf8_lossy(millis).parse::<u128>()
                                        {
                                            let now = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis();
                                            expiry = Some(now + ms);
                                            i += 2;
                                        } else {
                                            return RespMessage::Error(
                                                "ERR invalid expire time".to_string(),
                                            );
                                        }
                                    }
                                }
                                "EXAT" => {
                                    if let RespMessage::BulkString(Some(seconds)) = &vec[i + 1] {
                                        if let Ok(secs) =
                                            String::from_utf8_lossy(seconds).parse::<u64>()
                                        {
                                            expiry = Some(secs as u128 * 1000);
                                            i += 2;
                                        } else {
                                            return RespMessage::Error(
                                                "ERR invalid expire time".to_string(),
                                            );
                                        }
                                    }
                                }
                                "PXAT" => {
                                    if let RespMessage::BulkString(Some(millis)) = &vec[i + 1] {
                                        if let Ok(ms) =
                                            String::from_utf8_lossy(millis).parse::<u128>()
                                        {
                                            expiry = Some(ms);
                                            i += 2;
                                        } else {
                                            return RespMessage::Error(
                                                "ERR invalid expire time".to_string(),
                                            );
                                        }
                                    }
                                }
                                _ => return RespMessage::Error("ERR syntax error".to_string()),
                            }
                        } else {
                            return RespMessage::Error("ERR invalid option format".to_string());
                        }
                    }

                    db.lock()
                        .await
                        .insert(key, ValueWithExpiry { value, expiry });
                    RespMessage::SimpleString("OK".to_string())
                } else {
                    RespMessage::Error("ERR invalid SET arguments".to_string())
                }
            }

            "GET" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(key_bytes)) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::BulkString(None);
                            }
                        }
                        RespMessage::BulkString(Some(value_with_expiry.value.as_bytes().to_vec()))
                    } else {
                        RespMessage::BulkString(None)
                    }
                } else {
                    RespMessage::Error("ERR invalid GET argument. Expected key".to_string())
                }
            }

            "EXISTS" if vec.len() > 1 => {
                let mut counter = 0;
                let mut db_guard = db.lock().await;
                for i in 1..vec.len() {
                    if let RespMessage::BulkString(Some(key_bytes)) = &vec[i] {
                        let key = String::from_utf8_lossy(key_bytes).to_string();
                        if let Some(value_with_expiry) = db_guard.get(&key) {
                            if let Some(expiry_time) = value_with_expiry.expiry {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis();
                                if now < expiry_time {
                                    counter += 1;
                                } else {
                                    db_guard.remove(&key);
                                }
                            } else {
                                counter += 1;
                            }
                        }
                    } else {
                        return RespMessage::Error("ERR invalid EXISTS argument".to_string());
                    }
                }
                RespMessage::Integer(counter)
            }

            _ => RespMessage::Error("ERR unknown command".to_string()),
        }
    } else {
        RespMessage::Error("ERR invalid command format".to_string())
    }
}

