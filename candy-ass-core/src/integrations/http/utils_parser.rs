use crate::integrations::http::HttpResponseError;

pub fn parse_f64(value: &serde_json::Value, field: &str) -> Result<f64, HttpResponseError> {
    match value {
        serde_json::Value::String(s) => s
            .parse::<f64>()
            .map_err(|_| HttpResponseError::Unexpected(format!("Failed to parse f64 from string for {}", field))),
        serde_json::Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| HttpResponseError::Unexpected(format!("Invalid f64 number for {}", field))),
        _ => Err(HttpResponseError::Unexpected(format!("Unexpected type for {}", field))),
    }
}

pub fn parse_u64(value: &serde_json::Value, field: &str) -> Result<u64, HttpResponseError> {
    match value {
        serde_json::Value::Number(n) => n
            .as_u64()
            .ok_or_else(|| HttpResponseError::Unexpected(format!("Invalid u64 number for {}", field))),
        serde_json::Value::String(s) => s
            .parse::<u64>()
            .map_err(|_| HttpResponseError::Unexpected(format!("Failed to parse u64 from string for {}", field))),
        _ => Err(HttpResponseError::Unexpected(format!("Unexpected type for {}", field))),
    }
}

#[cfg(test)]
mod tests {
    use crate::integrations::http::HttpResponseError;
    use crate::integrations::http::utils_parser::{parse_f64, parse_u64};
    use serde_json::json;

    #[test]
    fn test_parse_f64_from_string() {
        let value = json!("42.5");
        let result = parse_f64(&value, "valid_string");
        assert_eq!(result.unwrap(), 42.5);
    }

    #[test]
    fn test_parse_f64_from_number() {
        let value = json!(13.14);
        let result = parse_f64(&value, "valid_number");
        assert_eq!(result.unwrap(), 13.14);
    }

    #[test]
    fn test_parse_f64_invalid_string() {
        let value = json!("abc");
        let result = parse_f64(&value, "invalid_string");
        assert!(matches!(result, Err(HttpResponseError::Unexpected(msg)) if msg.contains("invalid_string")));
    }

    #[test]
    fn test_parse_f64_unexpected_type() {
        let value = json!(true);
        let result = parse_f64(&value, "invalid_type");
        assert!(matches!(result, Err(HttpResponseError::Unexpected(msg)) if msg.contains("Unexpected type")));
    }

    #[test]
    fn test_parse_u64_from_string() {
        let value = json!("42");
        let result = parse_u64(&value, "valid_string");
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_parse_u64_from_number() {
        let value = json!(3);
        let result = parse_u64(&value, "valid_number");
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_parse_u64_invalid_string() {
        let value = json!("42.2");
        let result = parse_u64(&value, "invalid_string");
        assert!(matches!(result, Err(HttpResponseError::Unexpected(msg)) if msg.contains("invalid_string")));
    }

    #[test]
    fn test_parse_u64_invalid_number() {
        let value = json!(1.123);
        let result = parse_u64(&value, "invalid_number");
        assert!(matches!(result, Err(HttpResponseError::Unexpected(msg)) if msg.contains("invalid_number")));
    }

    #[test]
    fn test_parse_u64_unexpected_type() {
        let value = json!(true);
        let result = parse_u64(&value, "invalid_type");
        assert!(matches!(result, Err(HttpResponseError::Unexpected(msg)) if msg.contains("Unexpected type")));
    }
}
