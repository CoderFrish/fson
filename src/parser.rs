use std::collections::HashMap;
use std::str::Chars;

use crate::JsonValue;
use crate::error::JsonError;

pub struct Parser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

/// 这里以后会处理的一些问题，先放着作为todo
/// 继续完善数字的解析与处理
/// 添加递归深度检查，防止大量递归出现导致进程出事
/// 搞定处理 BOM 的逻辑
/// 完成基础生产测试
impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    /* 临时先 pub 后面设计api的时候再考虑是否 private */
    pub fn new(json: &'a str) -> Self {
        let mut chars = json.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn expect(&mut self, expected: char) -> Result<(), JsonError> {
        match self.peek() {
            None => Err(JsonError::EOF),
            Some(ch) => {
                if ch == expected {
                    self.advance();
                    Ok(())
                } else {
                    Err(JsonError::ExpectedSymbol(ch, expected))
                }
            }
        }
    }

    fn read_string(&mut self) -> Result<String, JsonError> {
        let mut builder = String::new();
        self.expect('"')?;

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            if ch == '\\' {
                self.process_escape(&mut builder)?;
            } else if (ch as u32) < 0x20 {
                return Err(JsonError::InvalidStringChar(ch));
            } else {
                builder.push(ch);
                self.advance();
            }
        }

        self.expect('"')?;
        Ok(builder)
    }

    /* 转义部分 start - 写的有点不行，而且unicode的转义部分还没完善 */
    fn process_escape(&mut self, builder: &mut String) -> Result<(), JsonError> {
        /* 跳过转义符 */
        self.advance();

        /* 处理转义标志 */
        match self.peek() {
            None => return Err(JsonError::EOF),
            Some(ch) => match ch {
                '"' => {
                    builder.push('"');
                    self.advance();
                }

                '\\' => {
                    builder.push('\\');
                    self.advance();
                }

                'n' => {
                    builder.push('\n');
                    self.advance();
                }

                'r' => {
                    builder.push('\r');
                    self.advance();
                }

                't' => {
                    builder.push('\t');
                    self.advance();
                }

                'b' => {
                    builder.push('\x08');
                    self.advance();
                }

                'f' => {
                    builder.push('\x0c');
                    self.advance();
                }

                'u' => {
                    self.advance(); /* 跳过 u */
                    builder.push(self.read_unicode()?);
                }

                _ => return Err(JsonError::IncompleteEscape),
            },
        };

        Ok(())
    }

    /* 重新编写了 read_unicode 的逻辑，为以后代理对做准备 */
    fn read_unicode_hex(&mut self) -> Result<u32, JsonError> {
        let mut count: u32 = 0;
        let mut result: u32 = 0;
        while let Some(ch) = self.peek()
            && count < 4
        {
            result = result * 16 + Self::hex_char_to_digit(ch)?;

            self.advance();
            count += 1;
        }

        if count != 4 {
            return Err(JsonError::InvalidUnicodeSize);
        }

        Ok(result)
    }

    fn read_unicode(&mut self) -> Result<char, JsonError> {
        let current_unicode = self.read_unicode_hex()?;

        /* 代理对的取值范围 */
        let high_surrogate = 55296..=56319u32;
        let low_surrogate = 56320..=57343u32;
        if high_surrogate.contains(&current_unicode) {
            match self.peek() {
                None => Err(JsonError::EOF),

                Some('\\') => {
                    self.advance(); /* skip \ */
                    match self.peek() {
                        None => Err(JsonError::EOF),
                        Some('u') => {
                            self.advance(); /* skip u */
                            let next_unicode = self.read_unicode_hex()?;

                            if !low_surrogate.contains(&next_unicode) {
                                return Err(JsonError::InvalidSurrogate)
                            }

                            Ok(Self::marge_surrogates(next_unicode, current_unicode)?)
                        }

                        Some(_) => Err(JsonError::IncompleteSurrogate),
                    }
                }

                /* 这里是处理不完成缓冲对 */
                Some(_) => Err(JsonError::IncompleteSurrogate),
            }
        } else if low_surrogate.contains(&current_unicode) {
            Err(JsonError::IncompleteSurrogate)
        } else {
            char::from_u32(current_unicode)
                .ok_or(JsonError::InvalidUnicode(format!("{:x}", current_unicode)))
        }
    }

    fn marge_surrogates(low: u32, high: u32) -> Result<char, JsonError> {
        let cp = 0x1_0000 + ((high - 0xD800) << 10) + (low - 0xDC00);
        char::from_u32(cp).ok_or(JsonError::InvalidUnicode(format!("{:x}, {:x}", high, low)))
    }

    fn hex_char_to_digit(ch: char) -> Result<u32, JsonError> {
        match ch {
            '0'..='9' => Ok(ch as u32 - '0' as u32),
            'a'..='f' => Ok(ch as u32 - 'a' as u32 + 10),
            'A'..='F' => Ok(ch as u32 - 'A' as u32 + 10),

            _ => Err(JsonError::UnknownHexChar(ch)),
        }
    }
    /* 转义部分 end - 写的有点不行，而且unicode的转义部分还没完善 */

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek()
            && ['\t', '\r', '\n', ' '].contains(&ch)
        {
            self.advance();
        }
    }

    fn expect_entry_separator(&mut self, end_flag: char) -> Result<(), JsonError> {
        /* 处理逗号逻辑 - start - 有点屎，但是还行 */
        match self.peek() {
            None => return Err(JsonError::EOF),
            Some(',') => {
                self.expect(',')?;
                self.skip_whitespace();

                match self.peek() {
                    None => return Err(JsonError::EOF),
                    Some(ch) => {
                        if ch == end_flag {
                            return Err(JsonError::UnexpectedSymbol(','));
                        }
                    }
                }
            }

            Some(ch) if ch == end_flag => return Ok(()),

            Some(ch) => return Err(JsonError::ExpectedSymbol(ch, ',')),
        }
        /* 处理逗号逻辑 - end */
        Ok(())
    }

    fn read_object(&mut self) -> Result<HashMap<String, JsonValue>, JsonError> {
        let mut object_map = HashMap::new();
        self.expect('{')?;
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            self.skip_whitespace();
            let key = self.read_string()?;
            self.skip_whitespace();
            self.expect(':')?;
            self.skip_whitespace();
            let value = self.read_value()?;
            self.skip_whitespace();

            self.expect_entry_separator('}')?;

            object_map.insert(key, value);
        }

        self.skip_whitespace();
        self.expect('}')?;
        Ok(object_map)
    }

    fn read_array(&mut self) -> Result<Vec<JsonValue>, JsonError> {
        let mut array = Vec::new();
        self.expect('[')?;
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != ']'
        {
            self.skip_whitespace();
            array.push(self.read_value()?);
            self.skip_whitespace();

            self.expect_entry_separator(']')?;
        }

        self.skip_whitespace();
        self.expect(']')?;
        Ok(array)
    }

    fn is_literal_char(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ['+', '-', '_', '.'].contains(&ch)
    }

    fn read_literal(&mut self) -> Result<JsonValue, JsonError> {
        let mut builder = String::new();

        while let Some(ch) = self.peek()
            && self.is_literal_char(ch)
        {
            builder.push(ch);
            self.advance();
        }

        if builder == "true" {
            Ok(JsonValue::Boolean(true))
        } else if builder == "false" {
            Ok(JsonValue::Boolean(false))
        } else if builder == "null" {
            Ok(JsonValue::Null)
        } else {
            /* 这里数字的解析与判断以后再来写 */
            Ok(self.read_numeric(&builder)?)
        }
    }

    /* 这里数字处理等我秋假回来再研究，完成后对他进行生产化测试 */
    fn read_numeric(&mut self, builder: &String) -> Result<JsonValue, JsonError> {
        match builder.parse::<f64>() {
            Ok(result) => Ok(JsonValue::Numeric(result)),
            Err(_) => Err(JsonError::UnknownLiteral(builder.to_owned())),
        }
    }

    /* 临时先 pub 后面设计api的时候再考虑是否 private */
    pub fn read_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();

        match self.peek() {
            None => Err(JsonError::EOF),
            Some(ch) => match ch {
                '"' => Ok(JsonValue::String(self.read_string()?)),

                '{' => Ok(JsonValue::Object(self.read_object()?)),

                '[' => Ok(JsonValue::Array(self.read_array()?)),

                _ if self.is_literal_char(ch) => self.read_literal(),

                _ => Err(JsonError::UnexpectedSymbol(ch)),
            },
        }
    }
}
