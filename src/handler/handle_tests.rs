// Define ValueWithExpiry here if it’s not already in commands.rs, or import it
#[derive(Clone, Debug, PartialEq)]
pub struct ValueWithExpiry {
    pub value: String,
    pub expiry: Option<u128>,
}

#[cfg(test)]
mod tests {
    use super::ValueWithExpiry;

    #[test]
    fn test_value_with_expiry_creation() {
        // Test creating a ValueWithExpiry with no expiry
        let no_expiry = ValueWithExpiry {
            value: "Hello".to_string(),
            expiry: None,
        };
        assert_eq!(no_expiry.value, "Hello");
        assert_eq!(no_expiry.expiry, None);

        // Test creating a ValueWithExpiry with an expiry
        let with_expiry = ValueWithExpiry {
            value: "World".to_string(),
            expiry: Some(123456789),
        };
        assert_eq!(with_expiry.value, "World");
        assert_eq!(with_expiry.expiry, Some(123456789));
    }

    #[test]
    fn test_value_with_expiry_clone() {
        // Test cloning a ValueWithExpiry with no expiry
        let original_no_expiry = ValueWithExpiry {
            value: "Foo".to_string(),
            expiry: None,
        };
        let cloned_no_expiry = original_no_expiry.clone();
        assert_eq!(original_no_expiry, cloned_no_expiry);
        assert_eq!(cloned_no_expiry.value, "Foo");
        assert_eq!(cloned_no_expiry.expiry, None);

        // Test cloning a ValueWithExpiry with an expiry
        let original_with_expiry = ValueWithExpiry {
            value: "Bar".to_string(),
            expiry: Some(987654321),
        };
        let cloned_with_expiry = original_with_expiry.clone();
        assert_eq!(original_with_expiry, cloned_with_expiry);
        assert_eq!(cloned_with_expiry.value, "Bar");
        assert_eq!(cloned_with_expiry.expiry, Some(987654321));
    }

    #[test]
    fn test_value_with_expiry_field_access() {
        // Test accessing fields directly
        let mut item = ValueWithExpiry {
            value: "Test".to_string(),
            expiry: Some(1000),
        };

        // Verify initial values
        assert_eq!(item.value, "Test");
        assert_eq!(item.expiry, Some(1000));

        // Modify fields and verify changes
        item.value = "Modified".to_string();
        item.expiry = None;
        assert_eq!(item.value, "Modified");
        assert_eq!(item.expiry, None);
    }
}
