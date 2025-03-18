use std::fmt::Display;

/*
Enum representing the different types of RESP messages that can be serialized or deserialized.
The RESP protocol defines several types of messages, each with its own specific format.
This enum provides variants for each of these types:

- `SimpleString`: Represents a simple string, prefixed with `+`.
- `Error`: Represents an error message, prefixed with `-`.
- `Integer`: Represents an integer value, prefixed with `:`.
- `BulkString`: Represents a bulk string (could be null or have actual content), prefixed with `$`.
- `Array`: Represents an array of RESP messages, prefixed with `*`.
*/
#[derive(Debug, PartialEq)]
pub enum RespMessage {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespMessage>),
}

// Implement the Display trait for the RespMessage enum.
impl Display for RespMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RespMessage::SimpleString(s) => write!(f, "+{}\r\n", s),
            RespMessage::Error(s) => write!(f, "-{}\r\n", s),
            RespMessage::Integer(i) => write!(f, ":{}\r\n", i),
            RespMessage::BulkString(b) => {
                write!(f, "${}\r\n", b.len())?;
                let s = std::str::from_utf8(b).map_err(|_| std::fmt::Error)?;
                f.write_str(s)?;
                write!(f, "\r\n")
            }
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

// Implement a function to parse a RESP message from a byte slice.
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
            // Simple String: +<string>\r\n
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for simple string")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            Ok((RespMessage::SimpleString(s.to_string()), &input[pos + 2..]))
        }
        b'-' => {
            // Error: -<string>\r\n
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for error message")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            Ok((RespMessage::Error(s.to_string()), &input[pos + 2..]))
        }
        b':' => {
            // Integer: :<number>\r\n
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF for integer")?;
            let s = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            let i = s.parse().map_err(|_| "Invalid integer")?;
            Ok((RespMessage::Integer(i), &input[pos + 2..]))
        }
        b'$' => {
            // Bulk String: $<length>\r\n<data>\r\n
            let pos = input
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Missing CRLF in bulk string header")?;
            let len_str = std::str::from_utf8(&input[1..pos]).map_err(|_| "Invalid UTF-8")?;
            let len: usize = len_str.parse().map_err(|_| "Invalid bulk string length")?;
            let start = pos + 2; // skip CRLF
            let end = start + len;
            if input.len() < end + 2 || &input[end..end + 2] != b"\r\n" {
                return Err("Invalid bulk string data".to_string());
            }
            let data = input[start..end].to_vec();
            Ok((RespMessage::BulkString(data), &input[end + 2..]))
        }
        b'*' => {
            // Array: *<number>\r\n<elements...>
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
