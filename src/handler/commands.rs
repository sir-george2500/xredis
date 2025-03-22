use crate::handler::client_handler::Db;
use crate::handler::value::ValueWithExpiry;
use crate::resp::resp_protocol::RespMessage;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

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

            "DEL" if vec.len() > 1 => {
                let mut counter = 0;
                let mut db_guard = db.lock().await;
                for i in 1..vec.len() {
                    if let RespMessage::BulkString(Some(key_bytes)) = &vec[i] {
                        let key = String::from_utf8_lossy(key_bytes).to_string();
                        if db_guard.remove(&key).is_some() {
                            counter += 1;
                        }
                    } else {
                        return RespMessage::Error("ERR invalid DEL argument".to_string());
                    }
                }
                RespMessage::Integer(counter)
            }

            "INCR" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(key_bytes)) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get_mut(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::Error("ERR key has expired".to_string());
                            }
                        }

                        let value = value_with_expiry.value.parse::<i64>();
                        if let Ok(mut value) = value {
                            value += 1;
                            value_with_expiry.value = value.to_string();
                            RespMessage::Integer(value)
                        } else {
                            RespMessage::Error("ERR value is not an integer".to_string())
                        }
                    } else {
                        RespMessage::Error("ERR key does not exist".to_string())
                    }
                } else {
                    RespMessage::Error("ERR invalid INC argument".to_string())
                }
            }

            "DECR" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(key_bytes)) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get_mut(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::Error("ERR key has expired".to_string());
                            }
                        }

                        let value = value_with_expiry.value.parse::<i64>();
                        if let Ok(mut value) = value {
                            value -= 1;
                            value_with_expiry.value = value.to_string();
                            RespMessage::Integer(value)
                        } else {
                            RespMessage::Error("ERR value is not an integer".to_string())
                        }
                    } else {
                        RespMessage::Error("ERR key does not exist".to_string())
                    }
                } else {
                    RespMessage::Error("ERR invalid DEC argument".to_string())
                }
            }

            "LPUSH" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(key_bytes)) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get_mut(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::Error("ERR key has expired".to_string());
                            }
                        }

                        if let Some(list) = value_with_expiry
                            .value
                            .split(",")
                            .collect::<Vec<&str>>()
                            .first()
                        {
                            let mut new_list = vec![];
                            for i in 2..vec.len() {
                                if let RespMessage::BulkString(Some(item_bytes)) = &vec[i] {
                                    let item = String::from_utf8_lossy(item_bytes).to_string();
                                    new_list.insert(0, item);
                                } else {
                                    return RespMessage::Error(
                                        "ERR invalid LPUSH argument".to_string(),
                                    );
                                }
                            }
                            new_list.push(list.to_string());
                            value_with_expiry.value = new_list.join(",");
                            RespMessage::Integer(new_list.len() as i64)
                        } else {
                            RespMessage::Error("ERR key is not a list".to_string())
                        }
                    } else {
                        let mut new_list = vec![];
                        for i in 2..vec.len() {
                            if let RespMessage::BulkString(Some(item_bytes)) = &vec[i] {
                                let item = String::from_utf8_lossy(item_bytes).to_string();
                                new_list.push(item);
                            } else {
                                return RespMessage::Error(
                                    "ERR invalid LPUSH argument".to_string(),
                                );
                            }
                        }
                        db_guard.insert(
                            key,
                            ValueWithExpiry {
                                value: new_list.join(","),
                                expiry: None,
                            },
                        );
                        RespMessage::Integer(new_list.len() as i64)
                    }
                } else {
                    RespMessage::Error("ERR invalid LPUSH argument".to_string())
                }
            }

            "RPUSH" if vec.len() > 1 => {
                if let RespMessage::BulkString(Some(key_bytes)) = &vec[1] {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get_mut(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::Error("ERR key has expired".to_string());
                            }
                        }

                        if let Some(list) = value_with_expiry
                            .value
                            .split(",")
                            .collect::<Vec<&str>>()
                            .first()
                        {
                            let mut new_list = vec![];
                            for i in 2..vec.len() {
                                if let RespMessage::BulkString(Some(item_bytes)) = &vec[i] {
                                    let item = String::from_utf8_lossy(item_bytes).to_string();
                                    new_list.push(item);
                                } else {
                                    return RespMessage::Error(
                                        "ERR invalid RPUSH argument".to_string(),
                                    );
                                }
                            }
                            new_list.insert(0, list.to_string());
                            value_with_expiry.value = new_list.join(",");
                            RespMessage::Integer(new_list.len() as i64)
                        } else {
                            RespMessage::Error("ERR key is not a list".to_string())
                        }
                    } else {
                        let mut new_list = vec![];
                        for i in 2..vec.len() {
                            if let RespMessage::BulkString(Some(item_bytes)) = &vec[i] {
                                let item = String::from_utf8_lossy(item_bytes).to_string();
                                new_list.push(item);
                            } else {
                                return RespMessage::Error(
                                    "ERR invalid RPUSH argument".to_string(),
                                );
                            }
                        }
                        db_guard.insert(
                            key,
                            ValueWithExpiry {
                                value: new_list.join(","),
                                expiry: None,
                            },
                        );
                        RespMessage::Integer(new_list.len() as i64)
                    }
                } else {
                    RespMessage::Error("ERR invalid RPUSH argument".to_string())
                }
            }

            "LRANGE" if vec.len() == 4 => {
                if let (
                    RespMessage::BulkString(Some(key_bytes)),
                    RespMessage::BulkString(Some(start_bytes)),
                    RespMessage::BulkString(Some(stop_bytes)),
                ) = (&vec[1], &vec[2], &vec[3])
                {
                    let key = String::from_utf8_lossy(key_bytes).to_string();
                    let start = String::from_utf8_lossy(start_bytes)
                        .parse::<usize>()
                        .unwrap_or(0);
                    let stop = String::from_utf8_lossy(stop_bytes)
                        .parse::<usize>()
                        .unwrap_or(0);
                    let mut db_guard = db.lock().await;

                    if let Some(value_with_expiry) = db_guard.get(&key) {
                        if let Some(expiry_time) = value_with_expiry.expiry {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= expiry_time {
                                db_guard.remove(&key);
                                return RespMessage::Error("ERR key has expired".to_string());
                            }
                        }

                        if let Some(list) = value_with_expiry
                            .value
                            .split(",")
                            .collect::<Vec<&str>>()
                            .first()
                        {
                            let list = list.split(",").collect::<Vec<&str>>();
                            let mut new_list = vec![];
                            for i in start..=stop {
                                if let Some(item) = list.get(i) {
                                    new_list.push(item.to_string());
                                }
                            }
                            RespMessage::Array(
                                new_list
                                    .iter()
                                    .map(|item| {
                                        RespMessage::BulkString(Some(item.as_bytes().to_vec()))
                                    })
                                    .collect(),
                            )
                        } else {
                            RespMessage::Error("ERR key is not a list".to_string())
                        }
                    } else {
                        RespMessage::Error("ERR key does not exist".to_string())
                    }
                } else {
                    RespMessage::Error("ERR invalid LRANGE argument".to_string())
                }
            }

            // let save the database to a file as a JSON object
            "SAVE" => {
                let db_guard = db.lock().await;
                let json = serde_json::to_string(&*db_guard).unwrap();
                let mut file = File::create("db.json").unwrap();
                file.write_all(json.as_bytes()).unwrap();
                RespMessage::SimpleString("OK".to_string())
            }

            _ => RespMessage::Error("ERR unknown command".to_string()),
        }
    } else {
        RespMessage::Error("ERR invalid command format".to_string())
    }
}
