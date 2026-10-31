mod parser;
mod utility;

use std::num::{ParseFloatError, ParseIntError};

pub fn from_json(str: &str) -> Result<JsonValue, String> {
    let mut parser = parser::Parser::new(str);
    parser.read_value()
}

pub enum JsonValue {
    Null,
    Bool(bool),
    Numeric(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::BTreeMap<String, JsonValue>),
}

pub struct JsonNumber {
    value: String,
}

impl JsonNumber {
    pub fn as_f64(&self) -> f64 {
        match self.try_f64() {
            Ok(value) => value,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_f64(&self) -> Result<f64, ParseFloatError> {
        self.value.parse::<f64>()
    }

    pub fn as_i64(&self) -> i64 {
        match self.try_i64() {
            Ok(value) => value,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_i64(&self) -> Result<i64, ParseIntError> {
        self.value.parse::<i64>()
    }
}
