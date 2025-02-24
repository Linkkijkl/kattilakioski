/// Module containing all the validators for the API.
mod validators {
    /// Validates that a string's length is within the specified range.
    ///
    /// # Arguments
    /// * `min_length`: The minimum allowed length of the string.
    /// * `max_length`: The maximum allowed length of the string.
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    fn length(min_length: usize, max_length: usize, value: String) -> Result<(), String> {
        if value.len() < min_length {
            Err(format!(
                "Value must be at least {} characters long",
                min_length
            ))
        } else if value.len() > max_length {
            Err(format!(
                "Value must be at most {} characters long",
                max_length
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a string contains only alphanumeric characters.
    ///
    /// # Arguments
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())`` on success.
    fn alphanumeric(value: String) -> Result<(), String> {
        if !value.chars().all(|c| c.is_alphanumeric()) {
            Err("Value must be alphanumeric".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates that a string represents a valid currency value, containing only numbers and optionally one comma ("." or ",") for up to 2 decimal places.
    ///
    /// # Arguments
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    fn currency(value: String) -> Result<(), String> {
        let split: Vec<&str> = value.split(['.', ',']).collect();
        let flattened = split.concat();
        let splits = split.len();
        if (splits >= 2 && split[1].len() > 2) || !flattened.chars().all(char::is_numeric) {
            Err("Value must be a valid currency value (e.g., 1234.56).".to_string())
        } else {
            Ok(())
        }
    }

    mod tests {
        use super::*;

        #[test]
        fn test_length_validator() {
            assert!(length(5, 10, "hello".to_string()).is_ok());
            assert!(length(5, 10, "helloworld".to_string()).is_ok());
            assert!(length(7, 10, "short".to_string()).is_err());
            assert!(length(5, 10, "toolongforthisfield".to_string()).is_err());
        }

        #[test]
        fn test_alphanumeric_validator() {
            assert!(alphanumeric("hello".to_string()).is_ok());
            assert!(alphanumeric("hello123".to_string()).is_ok());
            assert!(alphanumeric("hello-world".to_string()).is_err());
        }

        #[test]
        fn test_currency_validator() {
            assert!(currency("1234.56".to_string()).is_ok());
            assert!(currency("1,234.56".to_string()).is_err());
            assert!(currency("1234,56".to_string()).is_ok());
            assert!(currency("1234.567".to_string()).is_err());
        }
    }
}
