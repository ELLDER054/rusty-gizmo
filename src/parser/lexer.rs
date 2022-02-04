pub mod token;
pub mod error;

use self::token::Token;
use self::error::ErrorType;
use self::error::emit_error;
use self::token::TokenType;

/// Stores information for a "Lexer"
pub struct Lexer {
    /// Current position in `code`
    pub pos: usize,

    /// Input string
    pub code: String,

    /// Current column in the code
    pub col: usize,
}

impl Lexer {
    /// Returns whether or not "c" is a valid identifier start
    fn is_identifier_start(&self, c: char) -> bool {
        (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
    }

    /// Returns whether or not "c" is a valid digit
    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    /// Returns whether or not "c" is a valid continuing identifier character
    fn is_identifier(&self, c: char) -> bool {
        self.is_identifier_start(c) || self.is_digit(c)
    }

    /// Advances current place in code by incrementing position and column
    fn advance(&mut self, sight: usize) {
        self.pos += sight;
        self.col += sight;
    }

    /// Returns the next character in code
    fn peek(&self, sight: usize) -> char {
        // If the next character is past the end of the input, return ' '
        if self.pos + sight >= self.code.len() {
            return ' ';
        }
        return self.code.chars().nth(self.pos + sight).unwrap();
    }

    /// Parses a character
    /// # Example
    /// `a` or `\n`
    fn parse_character(&mut self) -> String {
        if self.peek(0) == '\\' {
            self.advance(1);
            return match self.peek(0) {
                'n'  => "\\0A",
                't'  => "\\09",
                '\'' => "\\27",
                '\"' => "\\22",
                 _   => "\\"
            }.to_string();
        }
        return self.peek(0).to_string();
    }

    /// Loops through the input and collects the tokens
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
            let mut name:   String = String::new();
            let mut digit:  String = String::new();

            let mut _chr:   String = String::new();

            // Save off the column before collecting a token
            let begin = self.col;

            // Match the character and get the token's type and value
            let (value, typ): (&str, TokenType) = match c {
                '/' if self.peek(1) == '/' => {
                    // Skip over the '//'
                    self.advance(2);

                    // Contine advancing until a newline is found or the end
                    // of the input is reached
                    while c != '\n' {
                        if self.pos >= self.code.len() {
                            break;
                        }
                        c = self.peek(1);
                        self.advance(1);
                    }
                    continue;
                },
                '!' if self.peek(1) == '=' => {self.advance(2); ("!=", TokenType::NotEqual)},
                '+' => {self.advance(1); ("+", TokenType::Plus)},
                '-' => {self.advance(1); ("-", TokenType::Dash)},
                '*' => {self.advance(1); ("*", TokenType::Star)},
                '/' => {self.advance(1); ("/", TokenType::Slash)},
                '(' => {self.advance(1); ("(", TokenType::LeftParen)},
                ')' => {self.advance(1); (")", TokenType::RightParen)},
                '{' => {self.advance(1); ("{", TokenType::LeftBrace)},
                '}' => {self.advance(1); ("}", TokenType::RightBrace)},
                '[' => {self.advance(1); ("[", TokenType::LeftBracket)},
                ']' => {self.advance(1); ("]", TokenType::RightBracket)},
                '=' if self.peek(1) == '=' => {self.advance(2); ("==", TokenType::EqualEqual)},
                '=' => {self.advance(1); ("=", TokenType::Equal)},
                ';' => {self.advance(1); (";", TokenType::SemiColon)},
                '>' if self.peek(1) == '=' => {self.advance(2); (">=", TokenType::GreaterEqual)},
                '>' => {self.advance(1); (">", TokenType::GreaterThan)},
                '<' if self.peek(1) == '=' => {self.advance(2); ("<=", TokenType::LessEqual)},
                '<' => {self.advance(1); ("<", TokenType::LessThan)},
                ':' => {self.advance(1); (":", TokenType::Colon)},
                ',' => {self.advance(1); (",", TokenType::Comma)},
                '.' => {self.advance(1); (".", TokenType::Dot)},
                // Ignore whitespace
                ' ' | '\t' => {
                    self.advance(1);
                    continue;
                },
                // For a newline, increment the line number, increment the
                // position, and reset the column
                '\n' => {
                    lineno += 1;
                    self.advance(1);
                    self.col = 0;
                    continue;
                },
                '\'' => {
                    self.advance(1);
                    _chr = self.parse_character();
                    self.advance(1);
                    if self.peek(0) != '\'' {
                        let empty_token = Token {typ: TokenType::Error, value: " ".to_string(), lineno: lineno, col: self.col, line: lines[lineno - 1].to_string()};
                        emit_error(
                            "Expected a single quote".to_string(),
                            "help: Insert a single quote after this character",
                            &empty_token,
                            ErrorType::ExpectedToken
                        );
                    }
                    self.advance(1);
                    (&_chr, TokenType::Char)
                },
                '"' => {
                    // Set c to the character after the '"'
                    c = self.peek(1);

                    // Advance and consume the '"'
                    self.advance(1);

                    let mut len = 0;

                    // Loop until the end of the string
                    while c != '"' {
                        // When it reaches the end of the line without finding
                        // a second '"', give error
                        if c == '\n' || c == '\0' {
                            let empty_token = Token {typ: TokenType::Error, value: " ".to_string(), lineno: lineno, col: self.col, line: lines[lineno - 1].to_string()};
                            emit_error(
                                "Closing double quote was not found".to_string(),
                                "help: Add a closing double quote to signal the end of the string",
                                &empty_token,
                                ErrorType::UnexpectedEOF
                           );
                        }
                        // Add the character to allocated "string" variable
                        string.push_str(self.parse_character().as_str());

                        // Change character to the next character
                        c = self.peek(1);

                        // Advance our position and column
                        self.advance(1);
                        len += 1;
                    }

                    // Advance and consume the second '"'
                    self.advance(1);

                    string = format!("{}.{}", len, string);

                    // Return the new string token
                    (string.as_str(), TokenType::Str)
                },
                id if self.is_identifier_start(id) => {
                    // Loop through while we keep finding identifier characters
                    while self.is_identifier(c) {
                        // Add the character to the identifier
                        name.push(c);

                        // Change character to the next character
                        c = self.peek(1);

                        // Advance the position and column
                        self.advance(1);
                    }

                    // Match the identifier against all the keywords to find the appropriate token type
                    let id_type: TokenType = match name.as_str() {
                        "let"    => TokenType::Let,
                        "if"    => TokenType::If,
                        "else"    => TokenType::Else,
                        "ret"    => TokenType::Ret,
                        "break"    => TokenType::Break,
                        "continue"    => TokenType::Continue,
                        "func"    => TokenType::Func,
                        "while"  => TokenType::While,
                        "new"    => TokenType::New,
                        "struct" => TokenType::Struct,
                        "not"    => TokenType::Not,
                        "and"    => TokenType::And,
                        "or"     => TokenType::Or,
                        "true" | "false" => TokenType::Bool,
                        "int" | "string" | "char" | "bool" | "dec" => TokenType::Type,
                        _        => TokenType::Id,
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
                        c = self.peek(1);
                        
                        // Advance the postion and column
                        self.advance(1);
                    }

                    // If the next character is a dot, the number must be a
                    // floating point number
                    if c == '.' {
                        // Skip over the dot
                        c = self.peek(1);
                        self.advance(1);
                        digit.push('.');

                        // If a digit is not found after the dot, print an error
                        if !self.is_digit(c) {
                            let empty_token = Token {typ: TokenType::Error, value: "".to_string(), lineno: lineno, col: 0, line: lines[lineno - 1].to_string()};
                            emit_error(
                                "Expected number after dot".to_string(),
                                "help: Take away the dot or insert a number after the dot",
                                &empty_token,
                                ErrorType::DecNotFound
                            );
                        }

                        // Otherwise, continue to collect digits and add to the
                        // number
                        while self.is_digit(c) {
                            digit.push(c);
                            c = self.peek(1);
                            self.advance(1);
                        }

                        // If a dot is not found after that, override the token
                        // type, otherwise, print an error
                        if c != '.' {
                            typ = TokenType::Dec;
                        } else {
                            let empty_token = Token {typ: TokenType::Error, value: "".to_string(), lineno: lineno, col: 0, line: lines[lineno - 1].to_string()};
                            emit_error(
                                "Unexpected dot".to_string(),
                                "help: Take away this dot",
                                &empty_token,
                                ErrorType::DecTooManyDots
                            );
                        };
                    }
                    
                    // Return the number token
                    (digit.as_str(), typ)
                },
                // Finding unknown characters results in an error
                _ => {
                    let empty_token = Token {typ: TokenType::Error, value: c.to_string(), lineno: lineno, col: begin, line: line.to_string()};
                    emit_error(
                        format!("Unknown character '{}'", c),
                        "",
                        &empty_token,
                        ErrorType::UnknownChar
                    );
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
    let mut lexer = Lexer {code: "+ - * / == != < > <= >=".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
        Token {typ: TokenType::Plus, value: "+".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
        Token {typ: TokenType::Dash, value: "-".to_string(), lineno: 1, col: 2, line: lexer.code.clone()},
        Token {typ: TokenType::Star, value: "*".to_string(), lineno: 1, col: 4, line: lexer.code.clone()},
        Token {typ: TokenType::Slash, value: "/".to_string(), lineno: 1, col: 6, line: lexer.code.clone()},
        Token {typ: TokenType::EqualEqual, value: "==".to_string(), lineno: 1, col: 8, line: lexer.code.clone()},
        Token {typ: TokenType::NotEqual, value: "!=".to_string(), lineno: 1, col: 11, line: lexer.code.clone()},
        Token {typ: TokenType::LessThan, value: "<".to_string(), lineno: 1, col: 14, line: lexer.code.clone()},
        Token {typ: TokenType::GreaterThan, value: ">".to_string(), lineno: 1, col: 16, line: lexer.code.clone()},
        Token {typ: TokenType::LessEqual, value: "<=".to_string(), lineno: 1, col: 18, line: lexer.code.clone()},
        Token {typ: TokenType::GreaterEqual, value: ">=".to_string(), lineno: 1, col: 21, line: lexer.code.clone()},
    ]);
}

#[test]
fn test_identifiers_keywords_types() {
    let mut lexer = Lexer {code: "abc int dec bool string let while struct new and or not".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
		Token {typ: TokenType::Id, value: "abc".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "int".to_string(), lineno: 1, col: 4, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "dec".to_string(), lineno: 1, col: 8, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "bool".to_string(), lineno: 1, col: 12, line: lexer.code.clone()},
		Token {typ: TokenType::Type, value: "string".to_string(), lineno: 1, col: 17, line: lexer.code.clone()},
		Token {typ: TokenType::Let, value: "let".to_string(), lineno: 1, col: 24, line: lexer.code.clone()},
		Token {typ: TokenType::While, value: "while".to_string(), lineno: 1, col: 28, line: lexer.code.clone()},
		Token {typ: TokenType::Struct, value: "struct".to_string(), lineno: 1, col: 34, line: lexer.code.clone()},
		Token {typ: TokenType::New, value: "new".to_string(), lineno: 1, col: 41, line: lexer.code.clone()},
		Token {typ: TokenType::And, value: "and".to_string(), lineno: 1, col: 45, line: lexer.code.clone()},
		Token {typ: TokenType::Or, value: "or".to_string(), lineno: 1, col: 49, line: lexer.code.clone()},
		Token {typ: TokenType::Not, value: "not".to_string(), lineno: 1, col: 52, line: lexer.code.clone()}
    ]);
}

#[test]
fn test_const_values() {
    let mut lexer = Lexer {code: "5 'a' 5.5 true false \"abc\"".to_string(), col: 0, pos: 0};
    assert_eq!(lexer.lex(), vec![
		Token {typ: TokenType::Int, value: "5".to_string(), lineno: 1, col: 0, line: lexer.code.clone()},
		Token {typ: TokenType::Char, value: "a".to_string(), lineno: 1, col: 2, line: lexer.code.clone()},
		Token {typ: TokenType::Dec, value: "5.5".to_string(), lineno: 1, col: 6, line: lexer.code.clone()},
		Token {typ: TokenType::Bool, value: "true".to_string(), lineno: 1, col: 10, line: lexer.code.clone()},
		Token {typ: TokenType::Bool, value: "false".to_string(), lineno: 1, col: 15, line: lexer.code.clone()},
		Token {typ: TokenType::Str, value: "3.abc".to_string(), lineno: 1, col: 21, line: lexer.code.clone()},
    ]);
}
