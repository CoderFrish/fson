use crate::error::{JsonError, JsonResult, SyntaxParseError};
use crate::value::JsonValue;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::str::Chars;

/* 编写顺序 String类型 -> 标识符类型 -> Object类型 -> Array类型 -> Number类型 -> 完善部分 */
/* 完善部分包括：递归深度安全等等... */
/* 目前的目标是：编写Number类型 */
struct Parser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

const WHITESPACE: &[char] = &[' ', '\t', '\n', '\r'];
const HIGH_SURROGATE: RangeInclusive<u32> = 0xD800..=0xDBFF;
const LOW_SURROGATE: RangeInclusive<u32> = 0xDC00..=0xDFFF;

impl<'a> Parser<'a> {
    fn new(json: &'a str) -> Self {
        let mut chars = json.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(c) if WHITESPACE.contains(&c) => self.advance(),
                _ => break,
            }
        }
    }

    fn expect(&mut self, ch: char) -> JsonResult<()> {
        match self.peek() {
            Some(peeked) if peeked == ch => {
                self.advance();
            }

            /* 第一个是实际情况，第二个是在正确情况下应该的情况 */
            Some(peeked) => {
                return Err(JsonError::SyntaxParseError(
                    SyntaxParseError::ExpectedCharacter(peeked, ch),
                ));
            }

            None => return Err(JsonError::EofError),
        };

        Ok(())
    }

    /* 这里是读取String类型，返回String因为是一边可能是key的String，一边可能是value的String */
    fn read_string(&mut self) -> JsonResult<String> {
        let mut builder = String::new();
        self.expect('"')?;

        /* 这里我老想用新特性的，但是考虑到低版本的兼容性，那直接用这个了 */
        loop {
            match self.peek() {
                /* 关于提前退出和到下引号的退出 */
                None => return Err(JsonError::EofError),
                Some('"') => break,

                /* 以下的是关于转义的东西 */
                Some('\\') => {
                    self.advance(); /* 跳过转义符号 */
                    let ch = self.escape()?;
                    builder.push(ch);
                }

                Some(ch) if (ch as u32) < 0x20 => {
                    return Err(JsonError::SyntaxParseError(
                        SyntaxParseError::UnescapedCharacter(ch as u32),
                    ));
                }

                /* 如果不是转义的话就用以下的方案 */
                Some(ch) => {
                    self.advance();
                    builder.push(ch)
                }
            }
        }

        self.expect('"')?;
        Ok(builder)
    }

    fn escape(&mut self) -> JsonResult<char> {
        // 这里都是通用格式 跳过 + 返回对应的字符

        match self.peek() {
            /* 基础转义 - start */
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some('b') => {
                self.advance();
                Ok('\x08')
            }
            Some('f') => {
                self.advance();
                Ok('\x0c')
            }
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            /* 基础转义 - end */

            /* 下面是关于unicode转义的处理 */
            Some('u') => {
                self.advance();
                self.unicode()
            }

            None => Err(JsonError::EofError),
            Some(ch) => Err(JsonError::SyntaxParseError(
                SyntaxParseError::InvalidEscapeSequence(ch),
            )),
        }
    }

    /* unicode处理辅助函数 - start */
    fn unicode_hex(&mut self) -> JsonResult<u32> {
        let mut result: u32 = 0;

        for _ in 0..4 {
            match self.peek() {
                Some(ch) => {
                    let hex = ch.to_digit(16).ok_or(JsonError::SyntaxParseError(
                        SyntaxParseError::InvalidHexCharacter(ch),
                    ))?;
                    result = result * 16 + hex;

                    self.advance();
                }

                None => return Err(JsonError::EofError),
            }
        }

        Ok(result)
    }

    /* 主要的unicode处理 */
    fn unicode(&mut self) -> JsonResult<char> {
        let current_hex = self.unicode_hex()?;

        if HIGH_SURROGATE.contains(&current_hex) {
            match self.peek() {
                None => Err(JsonError::EofError),
                Some('\\') => {
                    self.advance();

                    match self.peek() {
                        None => Err(JsonError::EofError),
                        Some('u') => {
                            self.advance();
                            let next_hex = self.unicode_hex()?;
                            if !LOW_SURROGATE.contains(&next_hex) {
                                return Err(JsonError::SyntaxParseError(
                                    SyntaxParseError::InvalidUnicodeSequence,
                                ));
                            }

                            let unicode =
                                0x1_0000 + ((current_hex - 0xD800) << 10) + (next_hex - 0xDC00);
                            char::from_u32(unicode).ok_or(JsonError::SyntaxParseError(
                                SyntaxParseError::InvalidUnicodeSequence,
                            ))
                        }
                        Some(_) => Err(JsonError::SyntaxParseError(
                            SyntaxParseError::InvalidUnicodeSequence,
                        )),
                    }
                }
                Some(_) => Err(JsonError::SyntaxParseError(
                    SyntaxParseError::InvalidUnicodeSequence,
                )),
            }
        } else if LOW_SURROGATE.contains(&current_hex) {
            Err(JsonError::SyntaxParseError(
                SyntaxParseError::InvalidSurrogateSequence,
            ))
        } else {
            char::from_u32(current_hex).ok_or(JsonError::SyntaxParseError(
                SyntaxParseError::InvalidUnicodeSequence,
            ))
        }
    }
    /* unicode处理辅助函数 - end */

    fn read_true(&mut self) -> JsonResult<JsonValue> {
        self.expect('t')?;
        self.expect('r')?;
        self.expect('u')?;
        self.expect('e')?;

        Ok(JsonValue::Boolean(true))
    }

    fn read_false(&mut self) -> JsonResult<JsonValue> {
        self.expect('f')?;
        self.expect('a')?;
        self.expect('l')?;
        self.expect('s')?;
        self.expect('e')?;

        Ok(JsonValue::Boolean(false))
    }

    fn read_null(&mut self) -> JsonResult<JsonValue> {
        self.expect('n')?;
        self.expect('u')?;
        self.expect('l')?;
        self.expect('l')?;

        Ok(JsonValue::Null)
    }

    fn read_object(&mut self) -> JsonResult<JsonValue> {
        let mut object = BTreeMap::new();
        self.expect('{')?;
        self.skip_whitespace();

        if self.peek() == Some('}') {
            self.expect('}')?; /* 如果是 空 对象的话就提前返回 Map */
            return Ok(JsonValue::Object(object));
        }

        loop {
            if self.peek() == None {
                return Err(JsonError::EofError);
            }

            self.skip_whitespace();
            let key = self.read_string()?;
            self.skip_whitespace();
            self.expect(':')?;
            self.skip_whitespace();
            let value = self.read_value()?;
            self.skip_whitespace();

            object.insert(key, value);

            match self.peek() {
                None => return Err(JsonError::EofError),

                Some('}') => break,

                Some(',') => {
                    self.advance(); /* 跳过 , */
                    self.skip_whitespace();

                    match self.peek() {
                        None => return Err(JsonError::EofError),

                        Some('}') => {
                            return Err(JsonError::SyntaxParseError(
                                SyntaxParseError::UnexpectedCharacter(','),
                            ));
                        }

                        Some(_) => (),
                    }
                }

                Some(ch) => {
                    return Err(JsonError::SyntaxParseError(
                        SyntaxParseError::UnexpectedCharacter(ch),
                    ));
                }
            }
        }

        self.skip_whitespace();
        self.expect('}')?;
        Ok(JsonValue::Object(object))
    }

    fn read_array(&mut self) -> JsonResult<JsonValue> {
        let mut array = Vec::new();
        self.expect('[')?;
        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.expect(']')?;
            return Ok(JsonValue::Array(array));
        }

        loop {
            if self.peek() == None {
                return Err(JsonError::EofError);
            }

            self.skip_whitespace();
            let value = self.read_value()?;
            self.skip_whitespace();

            array.push(value);

            match self.peek() {
                None => return Err(JsonError::EofError),

                Some(']') => break,

                Some(',') => {
                    self.advance(); /* 跳过 , */
                    self.skip_whitespace();

                    match self.peek() {
                        None => return Err(JsonError::EofError),

                        Some(']') => {
                            return Err(JsonError::SyntaxParseError(
                                SyntaxParseError::UnexpectedCharacter(','),
                            ));
                        }

                        Some(_) => (),
                    }
                }

                Some(ch) => {
                    return Err(JsonError::SyntaxParseError(
                        SyntaxParseError::UnexpectedCharacter(ch),
                    ));
                }
            }
        }

        self.skip_whitespace();
        self.expect(']')?;
        Ok(JsonValue::Array(array))
    }

    fn read_value(&mut self) -> JsonResult<JsonValue> {
        match self.peek() {
            Some('"') => Ok(JsonValue::String(self.read_string()?)),

            Some('t') => self.read_true(),

            Some('f') => self.read_false(),

            Some('n') => self.read_null(),

            Some('{') => self.read_object(),

            Some('[') => self.read_array(),

            Some(ch) => Err(JsonError::SyntaxParseError(
                SyntaxParseError::UnexpectedCharacter(ch),
            )),

            None => Err(JsonError::EofError),
        }
    }
}

/* 这里创建一个辅助创建 Parser 的函数，来阻止读取完主要内容后还有意外的多余内容 */
pub fn json(json: &str) -> JsonResult<JsonValue> {
    /* 创建parser实例 */
    let mut parser = Parser::new(json);

    /* 这里跳过头空白和尾空白，中间读取json值 */
    parser.skip_whitespace();
    let result = parser.read_value()?;
    parser.skip_whitespace();

    /* 判断是否还能读取，如果能那就是还有意外内容 */
    if parser.peek().is_some() {
        return Err(JsonError::SyntaxParseError(
            SyntaxParseError::UnexpectedContent,
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::error::JsonResult;
    use crate::parser::json;
    use crate::value::JsonValue;

    #[test]
    fn test_numeric() -> JsonResult<()> {
        // let result = json("+123")?; /* 前导 + 错误 */
        // json("534235a")?;

        Ok(())
    }

    #[test]
    fn test_empty_array() -> JsonResult<()> {
        let result = json(r#"[]"#)?;
        assert!(matches!(result, JsonValue::Array(ref o) if o.is_empty()));
        Ok(())
    }

    #[test]
    fn test_empty_array_with_whitespace() -> JsonResult<()> {
        let result = json(r#"[   ]"#)?;
        assert!(matches!(result, JsonValue::Array(ref o) if o.is_empty()));
        Ok(())
    }

    #[test]
    fn test_nested_array() -> JsonResult<()> {
        let result = json(r#"[ [true, true], [false, false] ]"#)?;
        match result {
            JsonValue::Array(array) => {
                match &array[0] {
                    JsonValue::Array(array0) => {
                        for element in array0 {
                            assert!(matches!(element, JsonValue::Boolean(true)));
                        }
                    }
                    _ => panic!("failed to test."),
                }

                match &array[1] {
                    JsonValue::Array(array1) => {
                        for element in array1 {
                            assert!(matches!(element, JsonValue::Boolean(false)));
                        }
                    }
                    _ => panic!("failed to test."),
                }
            }
            _ => panic!("failed to test."),
        }
        Ok(())
    }

    #[test]
    fn test_array_trailing_comma_should_fail() {
        assert!(json(r#"[ true, ]"#).is_err());
        assert!(json(r#"[ true,   ]"#).is_err());
    }

    #[test]
    fn test_array() -> JsonResult<()> {
        let result = json(
            r#"
        [
            true,
            false
        ]
        "#,
        )?;

        match result {
            JsonValue::Array(array) => {
                assert!(matches!(&array[0], &JsonValue::Boolean(true)));
                assert!(matches!(&array[1], &JsonValue::Boolean(false)));
            }
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    fn test_empty_object() -> JsonResult<()> {
        let result = json(r#"{}"#)?;
        assert!(matches!(result, JsonValue::Object(ref o) if o.is_empty()));
        Ok(())
    }

    #[test]
    fn test_empty_object_with_whitespace() -> JsonResult<()> {
        let result = json(r#"{   }"#)?;
        assert!(matches!(result, JsonValue::Object(ref o) if o.is_empty()));
        Ok(())
    }

    #[test]
    fn test_nested_object() -> JsonResult<()> {
        let result = json(r#"{ "a": { "b": true } }"#)?;
        match result {
            JsonValue::Object(object) => match &object["a"] {
                JsonValue::Object(inner) => {
                    assert!(matches!(inner["b"], JsonValue::Boolean(true)));
                }
                _ => panic!("failed to test."),
            },
            _ => panic!("failed to test."),
        }
        Ok(())
    }

    #[test]
    fn test_object_trailing_comma_should_fail() {
        assert!(json(r#"{ "a": true, }"#).is_err());
        assert!(json(r#"{ "a": true,   }"#).is_err());
    }

    #[test]
    fn test_object() -> JsonResult<()> {
        let result = json(
            r#"
        {
            "message": "Hello World!!!",
            "opening": true
        }
        "#,
        )?;

        match result {
            JsonValue::Object(object) => {
                let message = &object["message"];
                match message {
                    JsonValue::String(s) => assert_eq!(s, "Hello World!!!"),
                    _ => panic!("failed to test."),
                }

                let opening = &object["opening"];
                assert!(matches!(opening, &JsonValue::Boolean(true)));
            }
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    fn test_boolean_true() -> JsonResult<()> {
        /* 测试基础unicode转义 String 类型解析 */
        let result = json(r#"true"#)?;
        assert!(matches!(result, JsonValue::Boolean(true)));

        Ok(())
    }

    #[test]
    fn test_boolean_false() -> JsonResult<()> {
        let result = json(r#"false"#)?;
        assert!(matches!(result, JsonValue::Boolean(false)));

        Ok(())
    }

    #[test]
    fn test_boolean_null() -> JsonResult<()> {
        let result = json(r#"null"#)?;
        assert!(matches!(result, JsonValue::Null));

        Ok(())
    }

    #[test]
    fn test_string_parse_with_based_unicode_escape_surrogate() -> JsonResult<()> {
        let result = json(r#""Hello World!!! \uD83D\uDE00""#)?;
        match result {
            JsonValue::String(s) => assert_eq!(s, "Hello World!!! 😀"),
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    fn test_string_parse_with_basic_unicode_escape() -> JsonResult<()> {
        let result = json(r#""Hello World!!! \u4E2D文""#)?;
        match result {
            JsonValue::String(s) => assert_eq!(s, "Hello World!!! 中文"),
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    fn test_string_parse_with_basic_escape() -> JsonResult<()> {
        let result = json(r#""Hello World!!! This is my name \"Frish2021\"""#)?;
        match result {
            JsonValue::String(s) => assert_eq!(s, "Hello World!!! This is my name \"Frish2021\""),
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    fn test_string_parse() -> JsonResult<()> {
        let result = json(r#""Hello World!!!""#)?;
        match result {
            JsonValue::String(s) => assert_eq!(s, "Hello World!!!"),
            _ => panic!("failed to test."),
        }

        Ok(())
    }

    #[test]
    #[should_panic(expected = "EofError")]
    fn test_eof() {
        let result = json(r#""Hello World!!!"#);
        match result {
            Ok(_) => (),
            Err(err) => panic!("{:?}", err),
        }
    }

    #[test]
    #[should_panic(expected = "SyntaxParseError(UnexpectedContent)")]
    fn test_unexpected_content() {
        let result = json(r#""Hello World!!!"sdasd"#);
        match result {
            Ok(_) => (),
            Err(err) => panic!("{:?}", err),
        }
    }
}
