use std::collections::HashMap;
use std::str::Chars;

use crate::{JsonValue};
use crate::error::{JsonError};

pub struct Parser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

/// 这里以后会处理的一些问题，先放着作为todo
/// 继续完善数字的解析与处理
/// 完善关于unicode的代理对处理
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
    fn read_unicode(&mut self) -> Result<char, JsonError> {
        let mut count: u32 = 0;
        let mut result: u32 = 0;
        while let Some(ch) = self.peek()
            && count < 4
        {
            /* 这里3减去计数是因为最低16进制的第一位在右边，而解析器是从左读的，所以需要3减计数取反 */
            result = result + (Self::hex_char_to_digit(ch)? * 16u32.pow(3 - count));

            self.advance();
            count += 1;
        }

        /* 代理对的取值范围 */
        let high_surrogate = 55296..=56319u32;
        let low_surrogate = 56320..=57343u32;
        if !(low_surrogate.contains(&result) && high_surrogate.contains(&result)) {
            unsafe {
                /* 这里为什么不检查，因为我已经确定这里result是合法的unicode编码 */
                Ok(char::from_u32_unchecked(result))
            }
        } else {
            /* 这里留着以后写代理对，Frish这下可惨了 */
            Err(JsonError::InvalidUnicode(format!("{:x}", result)))
        }
    }

    fn hex_char_to_digit(ch: char) -> Result<u32, JsonError> {
        if !ch.is_ascii_hexdigit() {
            return Err(JsonError::InvalidHexChar(ch));
        }

        match ch {
            '0'..='9' => Ok(ch as u32 - '0' as u32),
            'a'..='f' => Ok(ch as u32 - 'a' as u32 + 10),
            'A'..='F' => Ok(ch as u32 - 'A' as u32 + 10),

            _ => Err(JsonError::UnknownHexChar(ch)),
        }
    }
    /* 转义部分 end - 写的有点不行，而且unicode的转义部分还没完善 */

    fn skip_format_symbol(&mut self) -> Result<(), JsonError> {
        while let Some(ch) = self.peek()
            && (ch.is_whitespace() || ['\n', '\t'].contains(&ch))
        {
            self.advance();
        }

        Ok(())
    }

    fn expect_entry_separator(&mut self, end_flag: char) -> Result<(), JsonError> {
        /* 处理逗号逻辑 - start - 有点屎，但是还行 */
        match self.peek() {
            None => return Err(JsonError::EOF),
            Some(',') => {
                self.expect(',')?;
                self.skip_format_symbol()?;

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
        self.skip_format_symbol()?;

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            self.skip_format_symbol()?;
            let key = self.read_string()?;
            self.skip_format_symbol()?;
            self.expect(':')?;
            self.skip_format_symbol()?;
            let value = self.read_value()?;
            self.skip_format_symbol()?;

            self.expect_entry_separator('}')?;

            object_map.insert(key, value);
        }

        self.skip_format_symbol()?;
        self.expect('}')?;
        Ok(object_map)
    }

    fn read_array(&mut self) -> Result<Vec<JsonValue>, JsonError> {
        let mut array = Vec::new();
        self.expect('[')?;
        self.skip_format_symbol()?;

        while let Some(ch) = self.peek()
            && ch != ']'
        {
            self.skip_format_symbol()?;
            array.push(self.read_value()?);
            self.skip_format_symbol()?;

            self.expect_entry_separator(']')?;
        }

        self.skip_format_symbol()?;
        self.expect(']')?;
        Ok(array)
    }

    fn is_literal(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ['+', '-', '_', '.'].contains(&ch)
    }

    fn read_literal(&mut self) -> Result<JsonValue, JsonError> {
        let mut builder = String::new();

        while let Some(ch) = self.peek()
            && self.is_literal(ch)
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
            match builder.parse::<f64>() {
                Ok(result) => Ok(JsonValue::Numeric(result)),
                Err(_) => Err(JsonError::UnknownLiteral(builder)),
            }
        }
    }

    /* 临时先 pub 后面设计api的时候再考虑是否 private */
    pub fn read_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_format_symbol()?;

        match self.peek() {
            None => Err(JsonError::EOF),
            Some(ch) => match ch {
                '"' => Ok(JsonValue::String(self.read_string()?)),

                '{' => Ok(JsonValue::Object(self.read_object()?)),

                '[' => Ok(JsonValue::Array(self.read_array()?)),

                _ if self.is_literal(ch) => self.read_literal(),

                _ => Err(JsonError::UnexpectedSymbol(ch)),
            },
        }
    }
}
