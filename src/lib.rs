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

pub enum RespMessage {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespMessage>),
}

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
