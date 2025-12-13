use tabled::Tabled;

#[derive(Tabled, Debug, PartialEq, Eq, Hash)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub enum JsonFormat {
    Pretty,
    Compact,
}
