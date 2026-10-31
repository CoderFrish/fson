use crate::utility::{is_control, is_literal};

pub struct Parser<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek()
            && (c.is_whitespace() || is_control(c))
        {
            self.advance();
        }
    }

    fn expect(&mut self, expected: char) {
        if let Some(peek) = self.peek() {
            if peek == expected {
                self.advance();
            } else {
                panic!("about expect error(1).")
            }
        } else {
            panic!("about expect error(0).")
        }
    }

    fn read_object(&mut self) -> crate::JsonValue {
        let mut object = std::collections::BTreeMap::new();
        self.skip_whitespace();
        self.expect('{');
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            let key = self.read_string();
            self.skip_whitespace();
            self.expect(':');
            self.skip_whitespace();

            let value = self.read_value();

            /* skip , */
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                match ch {
                    ',' => {
                        self.skip_whitespace();
                        self.advance();
                    }

                    '"' => {
                        panic!("about read_object error(1).")
                    }

                    _ => ()
                }
            } else {
                panic!("about read_object error(0)")
            }

            object.insert(key, value);
        }

        self.skip_whitespace();
        self.expect('}');
        self.skip_whitespace();
        crate::JsonValue::Object(object)
    }

    pub fn read_value(&mut self) -> crate::JsonValue {
        if let Some(ch) = self.peek() {
            match ch {
                '"' => crate::JsonValue::String(self.read_string()),

                '{' => self.read_object(),

                '[' => self.read_array(),

                _ if is_literal(ch) => self.read_literal(),

                _ => panic!("about read_value error(1)."),
            }
        } else {
            panic!("about read_value error(0).");
        }
    }

    fn read_array(&mut self) -> crate::JsonValue {
        let mut array = Vec::new();
        self.skip_whitespace();
        self.expect('[');
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != ']'
        {
            self.skip_whitespace();
            let value = self.read_value();
            self.skip_whitespace();

            if let Some(ch) = self.peek() {
                match ch {
                    ',' => {
                        self.advance();
                    }

                    '"' => {
                        panic!("about read_array error(1).")
                    }

                    _ => ()
                }
            } else {
                panic!("about read_array error(0)")
            }

            array.push(value);
        }

        self.skip_whitespace();
        self.expect(']');
        self.skip_whitespace();

        crate::JsonValue::Array(array)
    }

    fn read_literal(&mut self) -> crate::JsonValue {
        let mut builder = String::new();

        while let Some(ch) = self.peek()
            && is_literal(ch)
        {
            builder.push(ch);
            self.advance();
        }

        if builder == "false" {
            crate::JsonValue::Bool(false)
        } else if builder == "true" {
            crate::JsonValue::Bool(true)
        } else if builder == "null" {
            crate::JsonValue::Null
        } else {
            let number = crate::JsonNumber { value: builder };
            crate::JsonValue::Numeric(number)
        }
    }

    fn read_string(&mut self) -> String {
        let mut builder = String::new();
        self.skip_whitespace();
        self.expect('"');

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            builder.push(ch);
            self.advance();
        }

        self.expect('"');
        self.skip_whitespace();
        builder
    }
}
