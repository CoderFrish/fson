use std::collections::HashMap;
use crate::parser::Parser;

mod parser;
pub mod error;

/*  */
#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Boolean(bool),
    Null,
    Numeric(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub fn parse(json: &str) -> Result<JsonValue, error::JsonError> {
    let mut parser = Parser::new(json);
    parser.read_value()
}
