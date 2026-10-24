use std::collections::HashMap;
use crate::JsonElement;

pub struct Parser<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
}

impl<'a> Parser<'a> {
    pub fn new(string: &'a str) -> Self {
        let mut chars = string.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek()
            && (c.is_whitespace() || c.is_control())
        {
            self.advance();
        }
    }

    fn expect(&mut self, excepted: char) {
        self.skip_whitespace();

        if Some(excepted) == self.peek() {
            self.advance();
        } else {
            panic!("expected {}", excepted); /* 临时顶替的panic不代表以后是panic */
        }

        self.skip_whitespace();
    }

    fn read_string(&mut self) -> String {
        let mut builder = String::new();
        self.expect('"');

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            builder.push(ch);
            self.advance();
        }

        self.expect('"');
        builder
    }

    fn read_json_string(&mut self) -> JsonElement {
        JsonElement::String(self.read_string())
    }

    pub fn read_object(&mut self) -> JsonElement {
        let mut object = HashMap::new();
        self.expect('{');
        while let Some(ch) = self.peek()
            && ch != '}'
        {
            let key = self.read_string();
            self.expect(':');
            let value = self.read_value();

            if let Some(next) = self.peek()
                && next != '}'
            {
                self.expect(',');
            }
            object.insert(key, value);
        }

        self.expect('}');
        JsonElement::Object(object)
    }

    fn read_identifier(&mut self) -> JsonElement {
        /* 这里先预留着，给以后null或者bool留 */
        self.advance();
        JsonElement::Identifier
    }

    fn read_value(&mut self) -> JsonElement {
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.read_json_string(),

                '{' => self.read_object(),

                _ => self.read_identifier(),
            }
        } else {
            panic!("unexpected EOF"); /* 临时顶替的panic不代表以后是panic */
        }
    }
}
