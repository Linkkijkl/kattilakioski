use actix_web::{Error, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

/// Module containing all the validator functions for the API.
pub mod validators {
    /// Validates that a string's length is within the specified range.
    ///
    /// # Arguments
    /// * `min_length`: The minimum allowed length of the string.
    /// * `max_length`: The maximum allowed length of the string.
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn length(min_length: usize, max_length: usize, value: &str) -> Result<(), String> {
        if value.len() < min_length {
            Err(format!(
                "must be at least {} characters long",
                min_length
            ))
        } else if value.len() > max_length {
            Err(format!(
                "must be at most {} characters long",
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
    pub fn alphanumeric(value: &str) -> Result<(), String> {
        if !value.chars().all(|c| c.is_alphanumeric()) {
            Err("must be alphanumeric".to_string())
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
    pub fn currency(value: &str) -> Result<(), String> {
        let split: Vec<&str> = value.split(['.', ',']).collect();
        let flattened = split.concat();
        let splits = split.len();
        if (splits >= 2 && split[1].len() > 2) || !flattened.chars().all(char::is_numeric) {
            Err("must be a valid currency value (e.g., 1234.56).".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates that a string contains at least one uppercase letter and one lowercase letter.
    ///
    /// # Arguments
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn contains_upper_and_lower_case(value: &str) -> Result<(), String> {
        let has_uppercase = value.chars().any(|c| c.is_uppercase());
        let has_lowercase = value.chars().any(|c| c.is_lowercase());
        if has_uppercase && has_lowercase {
            Ok(())
        } else {
            Err("must contain at least one uppercase letter and one lowercase letter.".to_string())
        }
    }

    /// Validates that a string contains at least one number.
    ///
    /// # Arguments
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn contains_number(value: &str) -> Result<(), String> {
        if value.chars().any(char::is_numeric) {
            Ok(())
        } else {
            Err("must contain at least one number.".to_string())
        }
    }

    /// Validates that a string contains at least one special character.
    /// The special characters are defined in the function as "!@#$%^&*()_+-=[]{}|;:,.<>?".
    ///
    /// # Arguments
    /// * `value`: The string to be validated.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn contains_special(value: &str) -> Result<(), String> {
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        if value.chars().any(|c| special_chars.contains(c)) {
            Ok(())
        } else {
            Err("must contain at least one special character.".to_string())
        }
    }

    /// Validates that a string is a valid username.
    ///
    /// # Arguments
    /// * `value`: The string to be validated as a username.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn username(value: &str) -> Result<(), String> {
        let helper = |value: &str| -> Result<(), String> {
            length(3, 20, value)?;
            alphanumeric(value)?;
            Ok(())
        };
        helper(value).map_err(|e| format!("Username {e}"))
    }

    /// Validates that a string is a valid password, ensuring it meets the following criteria:
    /// - Length between 8 and 64 characters.
    /// - Contains at least one uppercase letter and one lowercase letter.
    /// - Contains at least one number.
    /// - Contains at least one special character.
    ///
    /// # Arguments
    /// * `value`: The string to be validated as a password.
    ///
    /// # Returns
    /// A `Result` that is an error with a message if the value does not meet the criteria, or `Ok(())` on success.
    pub fn password(value: &str) -> Result<(), String> {
        let helper = |value: &str| -> Result<(), String> {
            length(8, 64, value)?;
            contains_upper_and_lower_case(value)?;
            contains_number(value)?;
            contains_special(value)?;
            Ok(())
        };
        helper(value).map_err(|e| format!("Password {e}"))
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

        #[test]
        fn test_contains_upper_and_lower_case() {
            assert!(contains_upper_and_lower_case("Password1!").is_ok());
            assert!(contains_upper_and_lower_case("password").is_err());
            assert!(contains_upper_and_lower_case("PASSWORD").is_err());
        }

        #[test]
        fn test_contains_number() {
            assert!(contains_number("Password1!").is_ok());
            assert!(contains_number("password").is_err());
            assert!(contains_number("PASSWORD").is_err());
        }

        #[test]
        fn test_contains_special() {
            assert!(contains_special("Password1!").is_ok());
            assert!(contains_special("password").is_err());
            assert!(contains_special("PASSWORD").is_err());
        }

        #[test]
        fn test_password() {
            assert!(password("Password1!").is_ok());
            assert!(password("password").is_err());
            assert!(password("PASSWORD").is_err());
            assert!(password("Pass1!").is_err());
            assert!(password("Pass!").is_err());
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ValidateQuery {
    value: String,
}

#[post("/validate/username")]
pub async fn validate_username(query: web::Json<ValidateQuery>) -> Result<HttpResponse, Error> {
    match validators::username(&query.value) {
        Ok(_) => Ok(HttpResponse::Ok().body("OK")),
        Err(str) => Ok(HttpResponse::Ok().body(str)),
    }
}

#[post("/validate/currency")]
pub async fn validate_currency(query: web::Json<ValidateQuery>) -> Result<HttpResponse, Error> {
    match validators::currency(&query.value) {
        Ok(_) => Ok(HttpResponse::Ok().body("OK")),
        Err(str) => Ok(HttpResponse::Ok().body(str)),
    }
}

#[post("/validate/password")]
pub async fn validate_password(query: web::Json<ValidateQuery>) -> Result<HttpResponse, Error> {
    match validators::password(&query.value) {
        Ok(_) => Ok(HttpResponse::Ok().body("OK")),
        Err(str) => Ok(HttpResponse::Ok().body(str)),
    }
}
