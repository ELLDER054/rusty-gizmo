pub mod token;

use self::token::Token;
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

    fn match_operator(&mut self, c: char) -> (&str, TokenType) {
        let peek: char = self.peek();
        if peek == '=' || peek == c {
            self.advance();
            self.advance();
        } else {
            self.advance();
        }
        return match c {
            '+' if peek == '=' => ("+=", TokenType::PlusEqual),
            '-' if peek == '=' => ("-=", TokenType::DashEqual),
            '*' if peek == '=' => ("*=", TokenType::StarEqual),
            '/' if peek == '=' => ("/=", TokenType::SlashEqual),
            '!' if peek == '=' => ("!=", TokenType::NotEqual),
            '+' if peek == '+' => ("++", TokenType::PlusPlus),
            '-' if peek == '-' => ("--", TokenType::DashDash),
            '*' if peek == '*' => ("**", TokenType::StarStar),
            '/' if peek == '/' => ("//", TokenType::SlashSlash),
            '+' => ("+", TokenType::Plus),
            '-' => ("-", TokenType::Dash),
            '*' => ("*", TokenType::Star),
            '/' => ("/", TokenType::Slash),
            _ => ("", TokenType::Error) // Never gets here
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut lineno: usize = 1;
        let cloned = self.code.clone();
        let lines: Vec<&str> = cloned.split('\n').collect();

        while self.pos < self.code.len() {
            let mut c: char = self.code.chars().nth(self.pos).unwrap();
            let mut string: String = String::new();
            let mut name: String = String::new();
            let mut digit: String = String::new();
            let begin = self.col;
            let (value, typ): (&str, TokenType) = match c {
                '+' | '-' | '*' | '/' if self.peek() == c => self.match_operator(c),
                '+' | '-' | '*' | '/' | '!' if self.peek() == '=' => self.match_operator(c),
                '+' | '-' | '*' | '/' => self.match_operator(c),
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
                        string.push(c);
                        c = self.peek();
                        self.advance();
                    }
                    self.advance();
                    (&string, TokenType::Str)
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
                    (&name, id_type)
                },
                num if self.is_digit(num) => {
                    while self.is_digit(c) {
                        digit.push(c);
                        c = self.peek();
                        self.advance();
                    }
                    (&digit, TokenType::Int)
                },
                _ => {
                    eprintln!("Error: Unkown Character");
                    break;
                },
            };
            tokens.push(Token {typ: typ, value : value.to_string(), lineno: lineno, col: begin, line: lines[lineno - 1].to_string()});
        }
        return tokens;
    }
}
