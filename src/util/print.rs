use crate::util::{JsonFormat, serialize_to_json};
use serde::Serialize;
use tabled::{Table, Tabled};

pub fn print_lines(lines: &str, title: &str) {
    let value = format!("\n--- {title} ---\n{lines}");
    print_raw(value);
}

pub fn print_table<T: Tabled>(rows: &[T], title: &str) {
    let table = Table::new(rows);
    let value = format!("\n--- {title} ---\n{}", table);
    print_raw(value);
}

pub fn print_json<T>(json_data: T, format: Option<JsonFormat>)
where
    T: Serialize,
{
    let format = format.unwrap_or(JsonFormat::Pretty);
    print_raw(serialize_to_json(&json_data, format).expect("json could not be serialized"));
}

pub fn print_raw(raw_data: String) {
    println!("{}", raw_data);
}
