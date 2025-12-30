extern crate colorz;
use self::colorz::Colorize;

use super::token::Token;

/// Stores information for an error
pub struct Error {
    /// Type of the error
    typ: ErrorType,

    /// Location of the error
    /// line number, column, line, value
    location: (usize, usize, String, String),

    /// Side notes and suggestions
    helpers: String
}

/// Implement functions for an error
impl Error {
    /// Emits an error
    pub fn emit(&mut self) {
        eprintln!("{}: {}", "Error".bright_red(), message_for(self.typ.clone()).bright_white());
        eprintln!("  {} {}", "-->".bright_blue(), format!("In elliott.gizmo:{}:{}", self.location.0, self.location.1).bright_white());
        eprintln!("{} {}", " ".repeat(self.location.0.to_string().len()), "|".bright_blue());
        eprintln!("{} {} {}", self.location.0.to_string().bright_blue(), "|".bright_blue(), self.location.2);
        eprint!("{} {} ", " ".repeat(self.location.0.to_string().len()), "|".bright_blue());
        eprintln!("{}{}", " ".repeat(self.location.1), "^".repeat(self.location.3.len()));
        for h in self.helpers.split('\n') {
            eprintln!("{}{}", " ".repeat((self.location.0 as i32).to_string().len() + 3), h.bright_white());
        }
        std::process::exit(1);
    }

    /// Adds a suggestion to the error
    pub fn help(&mut self, s: &str) -> &mut Self {
        self.helpers.push_str("help: ");
        self.helpers.push_str(s);
        self.helpers.push('\n');
        self
    }

    /// Adds a side note to the error
    pub fn note(&mut self, s: &str) -> &mut Self {
        self.helpers.push_str("note: ");
        self.helpers.push_str(s);
        self.helpers.push('\n');
        self
    }
}

/// Creates an error
pub fn error(t: ErrorType, token: &Token) -> Error {
    return Error {
        typ: t,
        location: (token.lineno, token.col, token.line.clone(), token.value.clone()),
        helpers: String::new()
    }
}

/// An enum with all the possible error types 
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorType {
    /// Lexer errors
    UnexpectedEOF,
    UnknownChar,
    DecTooManyDots,
    DecNotFound,

    /// Parser errors
    ExpectedToken,
    MismatchedTypes,
    UndefinedArray,
    UndefinedSymbol
}

/// finds the correct error message for a given ErrorType
fn message_for(e: ErrorType) -> String {
    match e {
        ErrorType::UnexpectedEOF   => "Unexpected end of input",
        ErrorType::UnknownChar     => "Unexpected character",
        ErrorType::DecTooManyDots  => "Floating point number has multiple dots",
        ErrorType::DecNotFound     => "Expected digits after dot",

        ErrorType::ExpectedToken   => "Expected token",
        ErrorType::MismatchedTypes => "Mismatched types",
        ErrorType::UndefinedArray  => "This array has no explicit type",
        ErrorType::UndefinedSymbol => "This symbol is undefined"
    }.to_string()
}
