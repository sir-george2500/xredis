use super::resp_protocol::*; // Import all RESP-related functions & types

#[test]
fn test_parse_simple_string() {
    let input = b"+OK\r\n";
    let expected = RespMessage::SimpleString("OK".to_string());
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_error() {
    let input = b"-Error message\r\n";
    let expected = RespMessage::Error("Error message".to_string());
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_integer() {
    let input = b":1000\r\n";
    let expected = RespMessage::Integer(1000);
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_bulk_string() {
    let input = b"$6\r\nfoobar\r\n";
    let expected = RespMessage::BulkString(Some(b"foobar".to_vec()));
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_empty_bulk_string() {
    let input = b"$0\r\n\r\n";
    let expected = RespMessage::BulkString(Some(Vec::new()));
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_null_bulk_string() {
    let input = b"$-1\r\n";
    let expected = RespMessage::BulkString(None);
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_array() {
    let input = b"*3\r\n+Hello\r\n:123\r\n$3\r\nfoo\r\n";
    let expected = RespMessage::Array(vec![
        RespMessage::SimpleString("Hello".to_string()),
        RespMessage::Integer(123),
        RespMessage::BulkString(Some(b"foo".to_vec())),
    ]);
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_empty_array() {
    let input = b"*0\r\n";
    let expected = RespMessage::Array(vec![]);
    let result = parse_resp(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_resp_trailing_data_error() {
    let input = b"+OK\r\n+Extra\r\n"; // Extra data after valid message
    let result = parse_resp(input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Trailing data");
}

#[test]
fn test_parse_resp_invalid_integer() {
    let input = b":abc\r\n"; // Invalid integer format
    let result = parse_resp(input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid integer");
}

#[test]
fn test_parse_resp_invalid_bulk_string_length() {
    let input = b"$xyz\r\nfoobar\r\n"; // Invalid bulk string length
    let result = parse_resp(input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid bulk string length");
}

