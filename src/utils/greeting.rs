//! Greeting utility module for testing.

/// Generate a greeting message.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A formatted greeting string in the format "Hello, {name}!"
///
/// # Examples
/// ```
/// use agentd::utils::greeting::greet;
/// let greeting = greet("World");
/// assert_eq!(greeting, "Hello, World!");
/// ```
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_with_name() {
        let result = greet("World");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_greet_with_empty_string() {
        let result = greet("");
        assert_eq!(result, "Hello, !");
    }

    #[test]
    fn test_greet_with_different_names() {
        assert_eq!(greet("Alice"), "Hello, Alice!");
        assert_eq!(greet("Bob"), "Hello, Bob!");
        assert_eq!(greet("Charlie"), "Hello, Charlie!");
    }

    #[test]
    fn test_greet_with_special_characters() {
        assert_eq!(greet("World!"), "Hello, World!!");
        assert_eq!(greet("Test@123"), "Hello, Test@123!");
    }
}
