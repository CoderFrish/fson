/* from frish2021, modified by chatgpt */
struct Parser<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
}

/* from frish2021, modified by chatgpt */
impl<'a> Parser<'a> {
    fn from(string: &'a str) -> Self {
        Self {
            chars: string.chars(),
            current: string.chars().next(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }
}

impl<'a> Parser<'a> {
    /* from chatgpt */
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            self.advance();
        }
    }

    /* from chatgpt */
    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();

        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected '{}'", expected))
        }
    }

    fn read_string(&mut self) -> String {
        let mut builder = String::new();
        self.advance();

        while let Some(ch) = self.peek()
            && ch != '"'
        {
            builder.push(ch);
            self.advance();
        }

        self.advance();
        builder
    }

    fn read_object(&mut self) {
        self.expect('}');

        while let Some(ch) = self.peek()
            && ch != '}'
        {
            let _key = self.read_string();
            self.expect(':');
            let _value = self.read_value();

            self.expect(',');
        }

        self.expect('{');

    }

    fn read_value(&mut self) {
        if let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    let value = self.read_string();
                    println!("{}", value);
                }

                '{' => self.read_object(),

                _ => self.advance(),
            }
        }
    }
}

fn main() {
    let json: &str = "{\"name\":\"Frish2021\",\"age\":\"16\",\"other\":{\"id\":\"1145141919810\",\"status\":\"writing\"}}";
    let mut parser = Parser::from(json);
    parser.read_object()
}
