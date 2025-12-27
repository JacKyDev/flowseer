use crate::util::JsonFormat;
use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn serialize_to_json<T: Serialize>(value: &T, format: JsonFormat) -> Result<String> {
    match format {
        JsonFormat::Pretty => Ok(serde_json::to_string_pretty(value)?),
        JsonFormat::Compact => Ok(serde_json::to_string(value)?),
    }
}

pub fn deserialize_from_json<T>(input: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(input).context("Failed to deserialize JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        age: u32,
    }

    fn test_data() -> TestStruct {
        TestStruct {
            name: "Alice".into(),
            age: 30,
        }
    }

    #[test]
    fn test_serialize_compact() {
        let data = test_data();
        let result = serialize_to_json(&data, JsonFormat::Compact).unwrap();
        assert!(result.contains("\"name\":\"Alice\""));
        assert!(result.contains("\"age\":30"));
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_serialize_pretty() {
        let data = test_data();
        let result = serialize_to_json(&data, JsonFormat::Pretty).unwrap();
        assert!(result.contains("\n"));
        assert!(result.contains("\"name\": \"Alice\""));
        assert!(result.contains("\"age\": 30"));
    }

    #[test]
    fn test_serialize_empty_struct() {
        #[derive(Serialize)]
        struct Empty;

        let data = Empty;
        let result = serialize_to_json(&data, JsonFormat::Compact).unwrap();
        assert_eq!(result, "null");
    }
}

#[cfg(test)]
mod tests_deserialize_from_json {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        value: u32,
    }

    #[test]
    fn deserializes_valid_json_struct() {
        let json = r#"{ "name": "test", "value": 42 }"#;

        let result: TestStruct = deserialize_from_json(json).unwrap();

        assert_eq!(
            result,
            TestStruct {
                name: "test".into(),
                value: 42
            }
        );
    }

    #[test]
    fn deserializes_valid_json_vec() {
        let json = r#"[1, 2, 3]"#;

        let result: Vec<u32> = deserialize_from_json(json).unwrap();

        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn fails_on_invalid_json_syntax() {
        let json = r#"{ "name": "test", "#;

        let err = deserialize_from_json::<TestStruct>(json).unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("Failed to deserialize JSON"));
    }

    #[test]
    fn fails_on_type_mismatch() {
        let json = r#"{ "name": "test", "value": "not_a_number" }"#;

        let err = deserialize_from_json::<TestStruct>(json).unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("Failed to deserialize JSON"));
    }

    #[test]
    fn fails_on_empty_input() {
        let json = "";

        let err = deserialize_from_json::<TestStruct>(json).unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("Failed to deserialize JSON"));
    }
}
