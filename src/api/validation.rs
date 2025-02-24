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
    fn length(min_length: usize, max_length: usize, value: &str) -> Result<(), String> {
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
    fn alphanumeric(value: &str) -> Result<(), String> {
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
    fn currency(value: &str) -> Result<(), String> {
        let split: Vec<&str> = value.split(['.', ',']).collect();
        let flattened = split.concat();
        let splits = split.len();
        if (splits >= 2 && split[1].len() > 2) || !flattened.chars().all(char::is_numeric) {
            Err("Value must be a valid currency value (e.g., 1234.56).".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates that a string is a valid username.
    /// The username must be between 3 and 20 characters long, contain only alphanumeric characters, and be in lowercase.
    ///
    /// # Arguments
    /// * `value`: The string to be validated as a username.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    fn username(value: &str) -> Result<(), String> {
        let processed = value.to_lowercase().trim().to_string();
        length(3, 20, &processed)?;
        alphanumeric(&processed)?;
        Ok(())
    }

    mod tests {
        use super::*;

        #[test]
        fn test_length_validator() {
            assert!(length(5, 10, "hello").is_ok());
            assert!(length(5, 10, "helloworld").is_ok());
            assert!(length(7, 10, "short").is_err());
            assert!(length(5, 10, "toolongforthisfield").is_err());
        }

        #[test]
        fn test_alphanumeric_validator() {
            assert!(alphanumeric("hello").is_ok());
            assert!(alphanumeric("hello123").is_ok());
            assert!(alphanumeric("hello-world").is_err());
        }

        #[test]
        fn test_currency_validator() {
            assert!(currency("1234.56").is_ok());
            assert!(currency("1,234.56").is_err());
            assert!(currency("1234,56").is_ok());
            assert!(currency("1234.567").is_err());
        }

        #[test]
        fn test_username_validator() {
            assert!(username("val1duser").is_ok());
            assert!(username("VaLiDuSER").is_ok());
            assert!(username("1234567890").is_ok());
            assert!(username("invalid user").is_err());
            assert!(username("inv@lid").is_err());
            assert!(username("").is_err());
        }
    }
}
