pub mod token;
pub mod error;

use self::token::Token;
use self::error::Error;
use self::error::ErrorType;
use self::token::TokenType;

pub struct Lexer {
    pub pos: usize,
    pub code: String,
    pub col: usize,
}

impl Lexer {
    fn is_identifier_start(&self, c: char) -> bool {
        (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_identifier(&self, c: char) -> bool {
        self.is_identifier_start(c) || self.is_digit(c)
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    fn peek(&self) -> char {
        if self.pos + 1 >= self.code.len() {
            return ' ';
        }
        return self.code.chars().nth(self.pos + 1).unwrap();
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut lineno: usize = 1;
        let cloned = self.code.clone();
        let lines: Vec<&str> = cloned.split('\n').collect();

        while self.pos < self.code.len() {
            let line: &str = lines[lineno - 1];
            let mut c: char = self.code.chars().nth(self.pos).unwrap();
            let mut string: String = String::new();
            let mut name: String = String::new();
            let mut digit: String = String::new();
            let begin = self.col;
            let (value, typ): (&str, TokenType) = match c {
                '+' if self.peek() == '=' => {self.advance(); ("+=", TokenType::PlusEqual)},
                '-' if self.peek() == '=' => {self.advance(); ("-=", TokenType::DashEqual)},
                '*' if self.peek() == '=' => {self.advance(); ("*=", TokenType::StarEqual)},
                '/' if self.peek() == '=' => {self.advance(); ("/=", TokenType::SlashEqual)},
                '!' if self.peek() == '=' => {self.advance(); ("!=", TokenType::NotEqual)},
                '+' if self.peek() == '+' => {self.advance(); ("++", TokenType::PlusPlus)},
                '-' if self.peek() == '-' => {self.advance(); ("--", TokenType::DashDash)},
                '*' if self.peek() == '*' => {self.advance(); ("**", TokenType::StarStar)},
                '/' if self.peek() == '/' => {self.advance(); ("//", TokenType::SlashSlash)},
                '+' => {self.advance(); ("(", TokenType::Plus)},
                '-' => {self.advance(); ("(", TokenType::Dash)},
                '*' => {self.advance(); ("(", TokenType::Star)},
                '/' => {self.advance(); ("(", TokenType::Slash)},
                '(' => {self.advance(); ("(", TokenType::LeftParen)},
                ')' => {self.advance(); (")", TokenType::RightParen)},
                '=' => {self.advance(); ("=", TokenType::LeftParen)},
                ';' => {self.advance(); (";", TokenType::SemiColon)},
                ' ' | '\t' => {
                    self.advance();
                    continue;
                },
                '\n' => {
                    lineno += 1;
                    self.advance();
                    self.col = 0;
                    continue;
                }
                '"' => {
                    c = self.peek();
                    self.advance();
                    while c != '"' {
                        if c == '\n' || c == '\0' {
                            let mut error = Error {typ: ErrorType::StringWithoutEnd, msg: "Closing double quote was not found", helpers: "help: Add a closing double quote to signal the end of the string".to_string()};
                            error.emit_error(Token {typ: TokenType::Error, value: " ".to_string(), lineno: lineno, col: begin + string.len() + 1, line: line.to_string()});
                            break;
                        }
                        string.push(c);
                        c = self.peek();
                        self.advance();
                    }
                    self.advance();
                    (string.as_str(), TokenType::Str)
                },
                id if self.is_identifier_start(id) => {
                    while self.is_identifier(c) {
                        name.push(c);
                        c = self.peek();
                        self.advance();
                    }
                    let id_type: TokenType = match name.as_str() {
                        "let" => TokenType::Let,
                        "if" => TokenType::If,
                        "while" => TokenType::While,
                        "for" => TokenType::For,
                        _ => TokenType::Id,
                    };
                    (name.as_str(), id_type)
                },
                num if self.is_digit(num) => {
                    while self.is_digit(c) {
                        digit.push(c);
                        c = self.peek();
                        self.advance();
                    }
                    if c == '.' {
                        c = self.peek();
                        self.advance();
                        if !self.is_digit(c) {
                            let mut err = Error {typ: ErrorType::DecNotFound, msg: "", helpers: "help: Take away this dot or insert a number after the dot".to_string()};
                            err.emit_error(Token {typ: TokenType::Error, value: ".".to_string(), lineno: lineno, col: begin + digit.len(), line: line.to_string()});
                            break;
                        }
                        while self.is_digit(c) {
                            digit.push(c);
                            c = self.peek();
                            self.advance();
                        }
                        if c != '.' {
                            (digit.as_str(), TokenType::Dec)
                        } else {
                            let mut err = Error {typ: ErrorType::DecTooManyDots, msg: "", helpers: "help: Take away this dot".to_string()};
                            err.note("note: floating point numbers may only have 1 dot");
                            err.emit_error(Token {typ: TokenType::Error, value: ".".to_string(), lineno: lineno, col: begin + digit.len() + 1, line: line.to_string()});
                            break;
                        };
                    }
                    (digit.as_str(), TokenType::Int)
                },
                _ => {
                    Error {typ: ErrorType::UnknownChar, msg: format!("Unknown character '{}'", c).as_str(), helpers: "".to_string()}.emit_error(Token {typ: TokenType::Error, value: format!("{}", c), lineno: lineno, col: begin, line: line.to_string()});
                    break;
                },
            };
            tokens.push(Token {typ: typ, value : value.to_string(), lineno: lineno, col: begin, line: line.to_string()});
        }
        return tokens;
    }
}
