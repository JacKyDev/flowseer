use crate::util::JsonFormat;
use anyhow::Result;
use serde::Serialize;

pub fn serialize_to_json<T: Serialize>(value: &T, format: JsonFormat) -> Result<String> {
    match format {
        JsonFormat::Pretty => Ok(serde_json::to_string_pretty(value)?),
        JsonFormat::Compact => Ok(serde_json::to_string(value)?),
    }
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
