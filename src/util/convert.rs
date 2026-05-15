use crate::util::KeyValue;
use serde::Serialize;
use serde_json::{Value, to_value};

pub fn convert_struct_to_table_with_keys<T: Serialize>(
    item: &T,
    key_field: &str,
    value_field: &str,
) -> Vec<KeyValue> {
    let value = to_value(item).expect("Failed to serialize struct");

    match value {
        Value::Array(arr) => arr
            .into_iter()
            .filter_map(|entry| match entry {
                Value::Object(map) => {
                    let key = map.get(key_field)?;
                    let value = map.get(value_field)?;
                    Some(KeyValue {
                        key: convert_value_to_simple_string(key.clone()),
                        value: convert_value_to_simple_string(value.clone()),
                    })
                }
                _ => None,
            })
            .collect(),
        _ => vec![KeyValue {
            key: "value".to_string(),
            value: convert_value_to_simple_string(value),
        }],
    }
}

pub fn convert_struct_to_table<T: Serialize>(item: &T) -> Vec<KeyValue> {
    let value = to_value(item).expect("Failed to serialize struct");

    match value {
        Value::Object(map) => map
            .into_iter()
            .map(|(k, v)| KeyValue {
                key: k,
                value: match v {
                    Value::String(s) => s,
                    Value::Null => "".to_string(),
                    Value::Array(arr) => arr
                        .into_iter()
                        .map(convert_value_to_simple_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    other => convert_value_to_simple_string(other),
                },
            })
            .collect(),
        _ => vec![KeyValue {
            key: "value".to_string(),
            value: serde_json::to_string_pretty(&value).unwrap_or("(error)".to_string()),
        }],
    }
}

pub fn convert_value_to_simple_string(value: Value) -> String {
    match value {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!("[{}]", arr.len()),
        Value::Object(obj) => format!("{{{} fields}}", obj.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    #[derive(Serialize)]
    struct MyStruct {
        key: String,
        value: String,
    }

    #[derive(Serialize)]
    struct Wrapper {
        key: String,
        something_else: i32,
    }

    #[derive(Serialize)]
    struct SimpleStruct {
        name: String,
        age: u32,
    }

    #[derive(Serialize)]
    struct StructWithNull {
        name: String,
        middle_name: Option<String>,
    }

    #[derive(Serialize)]
    struct StructWithArray {
        tags: Vec<String>,
    }

    #[derive(Serialize)]
    struct StructWithMixedArray {
        values: Vec<serde_json::Value>,
    }

    #[test]
    fn test_array_of_structs() {
        let items = vec![
            MyStruct {
                key: "foo".into(),
                value: "bar".into(),
            },
            MyStruct {
                key: "baz".into(),
                value: "qux".into(),
            },
        ];

        let result = convert_struct_to_table_with_keys(&items, "key", "value");

        assert_eq!(
            result,
            vec![
                KeyValue {
                    key: "foo".into(),
                    value: "bar".into()
                },
                KeyValue {
                    key: "baz".into(),
                    value: "qux".into()
                }
            ]
        );
    }

    #[test]
    fn test_array_with_missing_field() {
        let items = vec![Wrapper {
            key: "a".into(),
            something_else: 123,
        }];

        let result = convert_struct_to_table_with_keys(&items, "key", "value");

        // Field "value" missing → filtered out
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_struct_not_array() {
        let single = MyStruct {
            key: "one".into(),
            value: "only".into(),
        };

        let result = convert_struct_to_table_with_keys(&single, "key", "value");

        // When not an array, it becomes a single KeyValue with "value" as the key
        assert_eq!(
            result,
            vec![KeyValue {
                key: "value".into(),
                value: "{2 fields}".into(),
            }]
        );
    }

    #[test]
    fn test_non_object_in_array_is_skipped() {
        let data = vec!["simple string", "another"];
        let result = convert_struct_to_table_with_keys(&data, "key", "value");
        assert!(result.is_empty());
    }

    #[test]
    fn test_struct_with_array() {
        let input = StructWithArray {
            tags: vec!["rust".to_string(), "serde".to_string()],
        };

        let result = convert_struct_to_table(&input);

        assert_eq!(
            result,
            vec![KeyValue {
                key: "tags".to_string(),
                value: "rust, serde".to_string(),
            }]
        );
    }

    #[test]
    fn test_struct_with_mixed_array() {
        let input = StructWithMixedArray {
            values: vec![
                serde_json::json!(1),
                serde_json::json!("text"),
                serde_json::json!(null),
            ],
        };

        let result = convert_struct_to_table(&input);

        assert_eq!(
            result,
            vec![KeyValue {
                key: "values".to_string(),
                value: "1, text, null".to_string(),
            }]
        );
    }

    #[test]
    fn test_non_object_input() {
        let input = 42;

        let result = convert_struct_to_table(&input);
        let expected = vec![KeyValue {
            key: "value".to_string(),
            value: "42".to_string(),
        }];

        assert_eq!(
            result.into_iter().collect::<HashSet<_>>(),
            expected.into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn test_simple_struct() {
        let input = SimpleStruct {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = convert_struct_to_table(&input);
        let expected = vec![
            KeyValue {
                key: "name".into(),
                value: "Alice".into(),
            },
            KeyValue {
                key: "age".into(),
                value: "30".into(),
            },
        ];

        assert_eq!(
            result.into_iter().collect::<HashSet<_>>(),
            expected.into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn test_struct_with_null() {
        let input = StructWithNull {
            name: "Bob".to_string(),
            middle_name: None,
        };

        let result = convert_struct_to_table(&input);
        let expected = vec![
            KeyValue {
                key: "name".into(),
                value: "Bob".into(),
            },
            KeyValue {
                key: "middle_name".into(),
                value: "".into(),
            },
        ];

        assert_eq!(
            result.into_iter().collect::<HashSet<_>>(),
            expected.into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn test_string_value() {
        let val = json!("hello");
        assert_eq!(convert_value_to_simple_string(val), "hello");
    }

    #[test]
    fn test_number_value() {
        let val = json!(42);
        assert_eq!(convert_value_to_simple_string(val), "42");

        let val = json!(3.14);
        assert_eq!(convert_value_to_simple_string(val), "3.14");
    }

    #[test]
    fn test_boolean_value() {
        assert_eq!(convert_value_to_simple_string(json!(true)), "true");
        assert_eq!(convert_value_to_simple_string(json!(false)), "false");
    }

    #[test]
    fn test_null_value() {
        assert_eq!(convert_value_to_simple_string(json!(null)), "null");
    }

    #[test]
    fn test_array_value() {
        let val = json!(["a", "b", "c"]);

        assert_eq!(convert_value_to_simple_string(val), "[3]");
    }

    #[test]
    fn test_object_value() {
        let val = json!({ "key": "value", "x": 1 });
        let result = convert_value_to_simple_string(val);

        assert!(result.contains("{2 fields}"));
    }
}
