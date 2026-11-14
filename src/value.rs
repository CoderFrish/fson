use std::collections::BTreeMap;

pub enum JsonValue {
    String(String),
    Null,
    Boolean(bool),
    Array(Vec<JsonValue>),
    Numeric(),
    Object(BTreeMap<String, JsonValue>),
}

pub enum JsonNumeric {
}
