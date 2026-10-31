use crate::utility::{is_control, is_literal};
use crate::{JsonNumber, JsonValue};

pub struct Parser<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
    position: usize,
    line: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
        self.position += 1;
    }

    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Self { chars, current, position: 1, line: 1 }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek()
            && (is_control(c) || c == ' ')
        {
            if c == '\n' || c == '\r' {
                self.position = 1;
                self.line += 1;
            }

            self.advance();
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if let Some(peek) = self.peek() {
            if peek == expected {
                self.advance();
                Ok(())
            } else {
                Err(format!("({}, {}) 未知符号 `{}`, 这里应该是 `{}`", self.position, self.line, peek, expected).to_string())
            }
        } else {
            Err("解析器指针超出了可读取范围".to_string())
        }
    }

    fn read_object(&mut self) -> Result<JsonValue, String> {
        let mut object = std::collections::BTreeMap::new();
        self.expect('{')?;
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            let key = self.read_string()?;
            self.skip_whitespace();
            self.expect(':')?;
            self.skip_whitespace();

            let value = self.read_value()?;

            /* skip , */
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                match ch {
                    ',' => {
                        self.advance();
                        self.skip_whitespace();

                        if Some('}') == self.peek() {
                            return Err(format!("({}, {}) 对象不允许出现尾逗号", self.position, self.line));
                        }
                    }

                    '"' => return Err(format!("({}, {}) 键值对之间需要用逗号连接", self.position, self.line)),

                    _ => (),
                }
            } else {
                return Err("解析器指针超出了可读取范围".to_string());
            }
            self.skip_whitespace();

            object.insert(key, value);
        }

        self.skip_whitespace();
        self.expect('}')?;
        self.skip_whitespace();
        Ok(JsonValue::Object(object))
    }

    pub fn read_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        if let Some(ch) = self.peek() {
            match ch {
                '"' => Ok(JsonValue::String(self.read_string()?)),

                '{' => self.read_object(),

                '[' => self.read_array(),

                _ if is_literal(ch) => Ok(self.read_literal()),

                _ => Err(format!("({}, {}) 未知json类型符号 `{}`", self.position, self.line, ch)),
            }
        } else {
            Err("解析器指针超出了可读取范围".to_string())
        }
    }

    fn read_array(&mut self) -> Result<JsonValue, String> {
        let mut array = Vec::new();
        self.expect('[')?;
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != ']'
        {
            let value = self.read_value();

            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                match ch {
                    ',' => {
                        self.advance();
                        self.skip_whitespace();

                        if Some(']') == self.peek() {
                            return Err(format!("({}, {}) 数组不允许出现尾逗号", self.position, self.line));

                        }
                    }

                    '"' => return Err(format!("({}, {}) 键值对之间需要用逗号连接", self.position, self.line)),

                    _ => (),
                }
            } else {
                return Err("解析器指针超出了可读取范围".to_string())
            }
            self.skip_whitespace();

            array.push(value?);
        }

        self.skip_whitespace();
        self.expect(']')?;
        self.skip_whitespace();

        Ok(JsonValue::Array(array))
    }

    fn read_literal(&mut self) -> JsonValue {
        let mut builder = String::new();

        while let Some(ch) = self.peek()
            && is_literal(ch)
        {
            builder.push(ch);
            self.advance();
        }

        if builder == "false" {
            JsonValue::Bool(false)
        } else if builder == "true" {
            JsonValue::Bool(true)
        } else if builder == "null" {
            JsonValue::Null
        } else {
            let number = JsonNumber { value: builder };
            JsonValue::Numeric(number)
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        let mut builder = String::new();
        self.expect('"')?;

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            builder.push(ch);
            self.advance();
        }

        self.expect('"')?;
        Ok(builder)
    }
}
