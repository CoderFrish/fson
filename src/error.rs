pub type JsonResult<T> = Result<T, JsonError>;

#[derive(Debug)]
pub enum JsonError {
    /* 这里是语法解析错误，比如 少了个 { */
    SyntaxParseError(SyntaxParseError),
    /* 这里是数据语义解析错误，比如 String类型解析数字类型 */
    DataParseError(DataParseError),
    EofError, /* 这里是关于提前结束解析的错误 */
}

#[derive(Debug)]
pub enum SyntaxParseError {
    UnexpectedCharacter(char),
    UnexpectedContent,
    ExpectedCharacter(char, char),
    UnescapedCharacter(u32),
    InvalidSurrogateSequence,
    InvalidEscapeSequence(char),
    InvalidHexCharacter(char),
    InvalidDecimalCharacter(char),
    InvalidUnicodeSequence,
    InvalidNumericSequence,
}

#[derive(Debug)]
pub enum DataParseError {}
