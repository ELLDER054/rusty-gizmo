pub mod token;
pub mod error;

use self::token::Token;
use self::error::Error;
use self::error::ErrorType;
use self::token::TokenType;

// Stores information for a "Lexer"
pub struct Lexer {
    pub pos: usize,   // Current position in code
    pub code: String, // Input string
    pub col: usize,   // Current column in the code
}

impl Lexer {
    // Returns whether or not "c" is a valid identifier start
    fn is_identifier_start(&self, c: char) -> bool {
        (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
    }

    // Returns whether or not "c" is a valid digit
    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    // Returns whether or not "c" is a valid continuing identifier character
    fn is_identifier(&self, c: char) -> bool {
        self.is_identifier_start(c) || self.is_digit(c)
    }

    // Advances current place in code by incrementing position and column
    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    // Returns the next character in code
    fn peek(&self) -> char {
        // If the next character is past the end of the input, return ' '
        if self.pos + 1 >= self.code.len() {
            return ' ';
        }
        return self.code.chars().nth(self.pos + 1).unwrap();
    }

    // Loops through the input and collects the tokens
    pub fn lex(&mut self) -> Vec<Token> {
        // Initialize a new vector to store the tokens
        let mut tokens: Vec<Token> = Vec::new();

        // Stores the current line number
        let mut lineno: usize = 1;

        // Clones the code so that splitting it into different lines doesn't borrow "self.code"
        let cloned = self.code.clone();
        let lines: Vec<&str> = cloned.split('\n').collect();

        // Loop while our position is not at the end of the input
        while self.pos < self.code.len() {
            // Stores the current line
            let line: &str = lines[lineno - 1];

            // Stores te current character
            let mut c: char = self.code.chars().nth(self.pos).unwrap();

            // Allocates a possible string/name/digit for later
            let mut string: String = String::new();
            let mut name: String = String::new();
            let mut digit: String = String::new();

            // Save off the column before collecting a token
            let begin = self.col;

            // Match the character and get the token's type and value
            let (value, typ): (&str, TokenType) = match c {
                // TODO: Make "advance()" take a parameter that tells how many to advance
                '+' if self.peek() == '=' => {self.advance(); self.advance(); ("+=", TokenType::PlusEqual)},
                '-' if self.peek() == '=' => {self.advance(); self.advance(); ("-=", TokenType::DashEqual)},
                '*' if self.peek() == '=' => {self.advance(); self.advance(); ("*=", TokenType::StarEqual)},
                '/' if self.peek() == '=' => {self.advance(); self.advance(); ("/=", TokenType::SlashEqual)},
                '!' if self.peek() == '=' => {self.advance(); self.advance(); ("!=", TokenType::NotEqual)},
                '+' if self.peek() == '+' => {self.advance(); self.advance(); ("++", TokenType::PlusPlus)},
                '-' if self.peek() == '-' => {self.advance(); self.advance(); ("--", TokenType::DashDash)},
                '*' if self.peek() == '*' => {self.advance(); self.advance(); ("**", TokenType::StarStar)},
                '/' if self.peek() == '/' => {self.advance(); self.advance(); ("//", TokenType::SlashSlash)},
                '+' => {self.advance(); ("+", TokenType::Plus)},
                '-' => {self.advance(); ("-", TokenType::Dash)},
                '*' => {self.advance(); ("*", TokenType::Star)},
                '/' => {self.advance(); ("/", TokenType::Slash)},
                '(' => {self.advance(); ("(", TokenType::LeftParen)},
                ')' => {self.advance(); (")", TokenType::RightParen)},
                '=' if self.peek() == '=' => {self.advance(); self.advance(); ("==", TokenType::EqualEqual)},
                '=' => {self.advance(); ("=", TokenType::Equal)},
                ';' => {self.advance(); (";", TokenType::SemiColon)},
                '>' if self.peek() == '=' => {self.advance(); self.advance(); (">=", TokenType::GreaterEqual)},
                '>' => {self.advance(); (">", TokenType::GreaterThan)},
                '<' if self.peek() == '=' => {self.advance(); self.advance(); ("<=", TokenType::LessEqual)},
                '<' => {self.advance(); ("<", TokenType::LessThan)},
                ':' => {self.advance(); (":", TokenType::Colon)},
                ',' => {self.advance(); (",", TokenType::Comma)},
                // Ignore whitespace
                ' ' | '\t' => {
                    self.advance();
                    continue;
                },
                // For a newline, increment "lineno", increment position, and reset the column
                '\n' => {
                    lineno += 1;
                    self.advance();
                    self.col = 0;
                    continue;
                }
                '"' => {
                    // Set c to the character after the '"'
                    c = self.peek();

                    // Advance and consume the '"'
                    self.advance();

                    // Loop until the end of the string
                    while c != '"' {
                        // When it reaches the end of the line without finding
                        // a second '"', give error
                        if c == '\n' || c == '\0' {
                            let error = Error {typ: ErrorType::StringWithoutEnd, msg: "Closing double quote was not found", helpers: "help: Add a closing double quote to signal the end of the string".to_string()};
                            error.emit_error(&Token {typ: TokenType::Error, value: " ".to_string(), lineno: lineno, col: begin + string.len() + 1, line: line.to_string()});
                            continue;
                        }
                        // Add the character to allocated "string" variable
                        string.push(c);

                        // Change character to the next character
                        c = self.peek();

                        // Advance our position and column
                        self.advance();
                    }

                    // Advance and consume the second '"'
                    self.advance();

                    // Return the new string token
                    (string.as_str(), TokenType::Str)
                },
                id if self.is_identifier_start(id) => {
                    // Loop through while we keep finding identifier characters
                    while self.is_identifier(c) {
                        // Add the character to the identifier
                        name.push(c);

                        // Change character to the next character
                        c = self.peek();

                        // Advance the position and column
                        self.advance();
                    }

                    // Match the identifier against all the keywords to find the appropriate token type
                    let id_type: TokenType = match name.as_str() {
                        "let" => TokenType::Let,
                        "if" => TokenType::If,
                        "while" => TokenType::While,
                        "for" => TokenType::For,
                        "not" => TokenType::Not,
                        "and" => TokenType::And,
                        "or" => TokenType::Or,
                        "true" | "false" => TokenType::Bool,
                        "int" | "string" | "char" | "bool" | "dec" => TokenType::Type,
                        _ => TokenType::Id,
                    };

                    // Return the identifier token
                    (name.as_str(), id_type)
                },
                num if self.is_digit(num) => {
                    // Store the type of the number (int), if the number turns
                    // out to be a floating point number, override it
                    let mut typ: TokenType = TokenType::Int;

                    // Loop through while digits continue to be found
                    while self.is_digit(c) {
                        // Add the character to the number
                        digit.push(c);
                        
                        // Change character to the next character
                        c = self.peek();
                        
                        // Advance the postion and column
                        self.advance();
                    }

                    // If the next character is a dot, the number must be a
                    // floating point number
                    if c == '.' {
                        // Skip over the dot
                        c = self.peek();
                        self.advance();
                        digit.push('.');

                        // If a digit is not found after the dot, print an error
                        if !self.is_digit(c) {
                            let err = Error {typ: ErrorType::DecNotFound, msg: "", helpers: "help: Take away this dot or insert a number after the dot".to_string()};
                            err.emit_error(&Token {typ: TokenType::Error, value: ".".to_string(), lineno: lineno, col: begin + digit.len(), line: line.to_string()});
                            continue;
                        }

                        // Otherwise, continue to collect digits and add to the
                        // number
                        while self.is_digit(c) {
                            digit.push(c);
                            c = self.peek();
                            self.advance();
                        }

                        // If a dot is not found after that, override the token
                        // type, otherwise, print an error
                        if c != '.' {
                            typ = TokenType::Dec;
                        } else {
                            let mut err = Error {typ: ErrorType::DecTooManyDots, msg: "", helpers: "help: Take away this dot".to_string()};
                            err.note("note: floating point numbers may only have 1 dot");
                            err.emit_error(&Token {typ: TokenType::Error, value: ".".to_string(), lineno: lineno, col: begin + digit.len() + 1, line: line.to_string()});
                            continue;
                        };
                    }
                    
                    // Return the number token
                    (digit.as_str(), typ)
                },
                // Finding unknown characters results in an error
                _ => {
                    Error {typ: ErrorType::UnknownChar, msg: format!("Unknown character '{}'", c).as_str(), helpers: "".to_string()}.emit_error(&Token {typ: TokenType::Error, value: format!("{}", c), lineno: lineno, col: begin, line: line.to_string()});
                    continue;
                },
            };

            // Add the token to the tokens vector
            tokens.push(Token {typ: typ, value : value.to_string(), lineno: lineno, col: begin, line: line.to_string()});
        }

        // Returns the tokens vector
        return tokens;
    }
}

#[test]
fn test_operators() {
    let mut lexer = Lexer {code: "+ - * / ++ -- ** // += -= *= /= == != < > <= >=".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
        Token {typ: TokenType::Plus, value: "+".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
        Token {typ: TokenType::Dash, value: "-".to_string(), lineno: 1, col: 2, line: lexer.code.clone()},
        Token {typ: TokenType::Star, value: "*".to_string(), lineno: 1, col: 4, line: lexer.code.clone()},
        Token {typ: TokenType::Slash, value: "/".to_string(), lineno: 1, col: 6, line: lexer.code.clone()},
        Token {typ: TokenType::PlusPlus, value: "++".to_string(), lineno: 1, col: 8, line: lexer.code.clone()},
        Token {typ: TokenType::DashDash, value: "--".to_string(), lineno: 1, col: 11, line: lexer.code.clone()},
        Token {typ: TokenType::StarStar, value: "**".to_string(), lineno: 1, col: 14, line: lexer.code.clone()},
        Token {typ: TokenType::SlashSlash, value: "//".to_string(), lineno: 1, col: 17, line: lexer.code.clone()},
        Token {typ: TokenType::PlusEqual, value: "+=".to_string(), lineno: 1, col: 20, line: lexer.code.clone()},
        Token {typ: TokenType::DashEqual, value: "-=".to_string(), lineno: 1, col: 23, line: lexer.code.clone()},
        Token {typ: TokenType::StarEqual, value: "*=".to_string(), lineno: 1, col: 26, line: lexer.code.clone()},
        Token {typ: TokenType::SlashEqual, value: "/=".to_string(), lineno: 1, col: 29, line: lexer.code.clone()},
        Token {typ: TokenType::EqualEqual, value: "==".to_string(), lineno: 1, col: 32, line: lexer.code.clone()},
        Token {typ: TokenType::NotEqual, value: "!=".to_string(), lineno: 1, col: 35, line: lexer.code.clone()},
        Token {typ: TokenType::LessThan, value: "<".to_string(), lineno: 1, col: 38, line: lexer.code.clone()},
        Token {typ: TokenType::GreaterThan, value: ">".to_string(), lineno: 1, col: 40, line: lexer.code.clone()},
        Token {typ: TokenType::LessEqual, value: "<=".to_string(), lineno: 1, col: 42, line: lexer.code.clone()},
        Token {typ: TokenType::GreaterEqual, value: ">=".to_string(), lineno: 1, col: 45, line: lexer.code.clone()},
    ]);
}

#[test]
fn test_identifiers_keywords_types() {
    let mut lexer = Lexer {code: "abc int dec bool string if let while for and or not".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
		Token {typ: TokenType::Id, value: "abc".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "int".to_string(), lineno: 1, col: 4, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "dec".to_string(), lineno: 1, col: 8, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "bool".to_string(), lineno: 1, col: 12, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "string".to_string(), lineno: 1, col: 17, line: lexer.code.clone()},
		Token {typ: TokenType::If, value: "if".to_string(), lineno: 1, col: 24, line: lexer.code.clone()},
		Token {typ: TokenType::Let, value: "let".to_string(), lineno: 1, col: 27, line: lexer.code.clone()},
		Token {typ: TokenType::While, value: "while".to_string(), lineno: 1, col: 31, line: lexer.code.clone()},
		Token {typ: TokenType::For, value: "for".to_string(), lineno: 1, col: 37, line: lexer.code.clone()},
		Token {typ: TokenType::And, value: "and".to_string(), lineno: 1, col: 41, line: lexer.code.clone()},
		Token {typ: TokenType::Or, value: "or".to_string(), lineno: 1, col: 45, line: lexer.code.clone()},
		Token {typ: TokenType::Not, value: "not".to_string(), lineno: 1, col: 48, line: lexer.code.clone()}
    ]);
}

#[test]
fn test_const_values() {
    let mut lexer = Lexer {code: "5 5.5 true false \"abc\"".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
		Token {typ: TokenType::Int, value: "5".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
		Token {typ: TokenType::Dec, value: "5.5".to_string(), lineno: 1, col: 2, line: lexer.code.clone()},
		Token {typ: TokenType::Bool, value: "true".to_string(), lineno: 1, col: 6, line: lexer.code.clone()},
		Token {typ: TokenType::Bool, value: "false".to_string(), lineno: 1, col: 11, line: lexer.code.clone()},
		Token {typ: TokenType::Str, value: "abc".to_string(), lineno: 1, col: 17, line: lexer.code.clone()},
    ]);
}
