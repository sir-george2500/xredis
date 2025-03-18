use super::resp_protocol::*; // Import everything from resp_protocol

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
