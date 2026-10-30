struct Parser<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
}

#[derive(Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::BTreeMap<String, JsonValue>),
}

impl<'a> Parser<'a> {
    fn peek(&mut self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek()
            && c.is_whitespace()
        {
            self.advance();
        }
    }

    fn read_object(&mut self) -> JsonValue {
        let mut object = std::collections::BTreeMap::new();
        self.skip_whitespace();
        self.advance(); /* skip { */
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            let key = self.read_string();
            self.skip_whitespace();
            self.advance(); /* skip : */
            self.skip_whitespace();

            let value = self.read_value();

            /* skip , */
            self.skip_whitespace();
            if let Some(ch) = self.peek()
                && ch == ','
            {
                self.advance();
                self.skip_whitespace();
            }
            object.insert(key, value);
        }

        self.skip_whitespace();
        self.advance(); /* skip } */
        self.skip_whitespace();
        JsonValue::Object(object)
    }

    fn read_value(&mut self) -> JsonValue {
        if let Some(ch) = self.peek() {
            match ch {
                '"' => JsonValue::String(self.read_string()),

                '{' => self.read_object(),

                '[' => self.read_array(),

                _ if self.is_literal(ch) => self.read_literal(),

                _ => panic!("about read_value error(1)."),
            }
        } else {
            panic!("about read_value error(0).");
        }
    }

    fn read_array(&mut self) -> JsonValue {
        let mut array = Vec::new();
        self.skip_whitespace();
        self.advance(); /* skip [ */
        self.skip_whitespace();

        while let Some(ch) = self.peek()
            && ch != ']'
        {
            let value = self.read_value();
            self.skip_whitespace();
            if let Some(ch) = self.peek()
                && ch == ','
            {
                self.advance();
                self.skip_whitespace();
            }
            array.push(value);
        }

        self.skip_whitespace();
        self.advance(); /* skip ] */
        self.skip_whitespace();

        JsonValue::Array(array)
    }

    fn read_literal(&mut self) -> JsonValue {
        let mut builder = String::new();

        while let Some(ch) = self.peek()
            && self.is_literal(ch)
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
            self.parse_number(builder)
        }
    }

    fn parse_number(&self, number_str: String) -> JsonValue {
        let parsed = number_str.parse::<f64>();

        if let Ok(result) = parsed {
            JsonValue::Number(result)
        } else {
            panic!("about parse_number error(0).");
        }
    }

    fn is_literal(&self, ch: char) -> bool {
        ('0'..='9').contains(&ch) || ('a'..='z').contains(&ch) || ['+', '-', '.'].contains(&ch)
    }

    fn read_string(&mut self) -> String {
        let mut builder = String::new();
        self.skip_whitespace();
        self.advance(); /* skip " */

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            builder.push(ch);
            self.advance();
        }

        self.advance(); /* skip " */
        self.skip_whitespace();
        builder
    }
}

fn main() {
    let json = r#"{ "username": "CoderFrish", "nickname": "Frish2021", "identifier": { "std_id": 1145141919810 }, "status": { "writing": true, "testing": null, "playing": false }, "vec": [1, 2, 3] }"#;
    let mut parser = Parser::new(json);
    let object = parser.read_object();
    if let JsonValue::Object(object) = object {
        for (key, value) in object {
            println!("{} | {:?}", key, value);
        }
    }
}
