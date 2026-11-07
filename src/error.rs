use crate::JsonValue;

/// 这里错误的类型以后会重构一下的，暂时这样
#[derive(Debug)]
pub enum JsonError {
    EOF,
    UnknownHexChar(char),
    UnknownLiteral(String),
    InvalidUnicode(String),
    InvalidHexChar(char),
    IncompleteEscape,
    KeyTypeNotString(JsonValue),
    ExpectedSymbol(char, char),
    UnexpectedSymbol(char),
}
