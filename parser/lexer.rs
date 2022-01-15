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
                '<' if self.peek() == '=' => {self.advance(); self.advance(); (">=", TokenType::LessEqual)},
                '<' => {self.advance(); (">", TokenType::LessThan)},
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
