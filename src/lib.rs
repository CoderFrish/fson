use std::collections::HashMap;

mod parser;

pub enum JsonElement {
    Object(HashMap<String, JsonElement>),
    String(String),
    Identifier
}

pub fn new(source: &str) -> JsonElement {
    parser::Parser::new(source).read_object()
}
