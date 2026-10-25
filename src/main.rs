use std::vec::IntoIter;

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
}

#[derive(Debug)]
enum Token {
    NameSeparator,
    ValueSeparator,
    String(String),
    Literal(String),
    BeginObject,
    EndObject,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current = chars.next();
        Self { chars, current }
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn read_string(&mut self) -> Token {
        let mut builder = String::new();
        self.advance(); /* skip " */

        while let Some(c) = self.peek()
            && c != '"'
        {
            builder.push(c);
            self.advance();
        }

        self.advance(); /* skip " */
        Token::String(builder)
    }

    fn read_literal(&mut self) -> Token {
        let mut builder = String::new();
        while let Some(c) = self.peek()
            && utility::is_literal(c)
        {
            builder.push(c);
            self.advance();
        }

        Token::Literal(builder)
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                '}' => {
                    tokens.push(Token::EndObject);
                    self.advance();
                }

                '{' => {
                    tokens.push(Token::BeginObject);
                    self.advance();
                }

                ',' => {
                    tokens.push(Token::ValueSeparator);
                    self.advance();
                }

                ':' => {
                    tokens.push(Token::NameSeparator);
                    self.advance();
                }

                '"' => {
                    let token = self.read_string();
                    tokens.push(token);
                }

                _ if utility::is_literal(ch) => tokens.push(self.read_literal()),

                _ => {
                    self.advance();
                }
            }
        }

        tokens
    }
}

mod utility {
    pub fn is_literal(c: char) -> bool {
        c.is_ascii_alphabetic() || c.is_ascii_digit() || ['.', '+', '-'].contains(&c)
    }
}

struct Parser {
    iter: IntoIter<Token>,
    current: Option<Token>,
}

impl Parser {
    fn new(mut lexer: Lexer) -> Self {
        let tokens = lexer.tokenize();
        let mut iter = tokens.into_iter();
        let current = iter.next();
        Self { iter, current }
    }

    fn peek(&mut self) -> Option<Token> {
        self.current.take()
    }

    fn advance(&mut self) {
        self.current = self.iter.next();
    }

    fn parse(mut self) {
        while let Some(token) = self.peek() {
            println!("{:?}", token);

            self.advance();
        }
    }
}

fn main() {
    let json =
        r#"{ "name": "Frish2021", "age": 16, "other": { "id":1145141919810, "writing": true } }"#;
    let lexer = Lexer::new(json);
    let mut parser = Parser::new(lexer);
    parser.parse();
}
