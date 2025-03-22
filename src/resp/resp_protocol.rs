use core::fmt;
use std::fmt::Display;

/*
Enum representing the different types of RESP messages that can be serialized or deserialized.
The RESP protocol defines several types of messages, each with its own specific format.
This enum provides variants for each of these types:

- `SimpleString`: Represents a simple string, prefixed with `+` (e.g., `+OK\r\n`).
- `Error`: Represents an error message, prefixed with `-` (e.g., `-ERR invalid command\r\n`).
- `Integer`: Represents an integer value, prefixed with `:` (e.g., `:123\r\n`).
- `BulkString`: Represents a bulk string, prefixed with `$`, where `Option<Vec<u8>>` indicates:
  - `Some(bytes)` for a string with content (e.g., `$5\r\nHello\r\n`).
  - `None` for a null bulk string (e.g., `$-1\r\n`), used for absent or expired values.
- `Array`: Represents an array of RESP messages, prefixed with `*` (e.g., `*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n`).
*/

#[derive(Debug, PartialEq)]
pub enum RespMessage {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RespMessage>),
}

impl Display for RespMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RespMessage::SimpleString(s) => write!(f, "+{}\r\n", s),
            RespMessage::Error(s) => write!(f, "-{}\r\n", s),
            RespMessage::Integer(i) => write!(f, ":{}\r\n", i),
            RespMessage::BulkString(Some(bytes)) => {
                write!(f, "${}\r\n", bytes.len())?;
                let s = std::str::from_utf8(bytes).map_err(|_| fmt::Error)?;
                f.write_str(s)?;
                write!(f, "\r\n")
            }
            RespMessage::BulkString(None) => write!(f, "$-1\r\n"),
            RespMessage::Array(a) => {
                write!(f, "*{}\r\n", a.len())?;
                for m in a {
                    write!(f, "{}", m)?;
                }
                Ok(())
            }
        }
    }
}

pub fn parse_resp(input: &[u8]) -> Result<RespMessage, String> {
    let (msg, remaining) = parse_resp_recursive(input)?;
    if !remaining.is_empty() {
        return Err("Trailing data".to_string());
    }
    Ok(msg)
}

fn parse_resp_recursive(input: &[u8]) -> Result<(RespMessage, &[u8]), String> {
    if input.is_empty() {
        return Err("Empty input".to_string());
    }

    match input[0] {
        b'+' => {
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for simple string")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            Ok((RespMessage::SimpleString(s.to_string()), &input[pos + 2..]))
        }
        b'-' => {
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for error message")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            Ok((RespMessage::Error(s.to_string()), &input[pos + 2..]))
        }
        b':' => {
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for integer")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            let i = s.parse().map_err(|_| "Invalid integer")?;
            Ok((RespMessage::Integer(i), &input[pos + 2..]))
        }
        b'$' => {
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF in bulk string header")?;
            let len_str = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            let len = len_str
                .parse::<i64>()
                .map_err(|_| "Invalid bulk string length")?;
            if len == -1 {
                // Null bulk string
                return Ok((RespMessage::BulkString(None), &input[pos + 2..]));
            }
            let len = len as usize; // Safe cast since -1 is handled above
            let start = pos + 2;
            let end = start + len;
            if input.len() < end + 2 || &input[end..end + 2] != b"\r\n" {
                return Err("Invalid bulk string data".to_string());
            }
            let data = input[start..end].to_vec();
            Ok((RespMessage::BulkString(Some(data)), &input[end + 2..]))
        }
        b'*' => {
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF in array header")?;
            let count_str = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            let count: usize = count_str.parse().map_err(|_| "Invalid array length")?;
            let mut remaining = &input[pos + 2..];
            let mut elements = Vec::new();
            for _ in 0..count {
                let (element, rem) = parse_resp_recursive(remaining)?;
                elements.push(element);
                remaining = rem;
            }
            Ok((RespMessage::Array(elements), remaining))
        }
        _ => Err("Invalid message type".to_string()),
    }
}
