use xredis::RespMessage; // Import the module you're testing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let message = RespMessage::SimpleString("OK".to_string());
        assert_eq!(format!("{}", message), "+OK\r\n");
    }

    #[test]
    fn test_error() {
        let message = RespMessage::Error("Error message".to_string());
        assert_eq!(format!("{}", message), "-Error message\r\n");
    }

    #[test]
    fn test_integer() {
        let message = RespMessage::Integer(100);
        assert_eq!(format!("{}", message), ":100\r\n");
    }

    #[test]
    fn test_bulk_string() {
        let message = RespMessage::BulkString(vec![104, 101, 108, 108, 111]);
        assert_eq!(format!("{}", message), "$5\r\nhello\r\n");
    }

    #[test]
    fn test_array() {
        let message = RespMessage::Array(vec![
            RespMessage::SimpleString("ping".to_string()),
            RespMessage::SimpleString("echo".to_string()),
        ]);
        assert_eq!(format!("{}", message), "*2\r\n+ping\r\n+echo\r\n");
    }
}
