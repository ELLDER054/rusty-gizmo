use super::token::Token;

// TODO: Change the way we call an error in other files
// It is currently too tedious

/// Stores the information for each error
pub struct Error<'a> {
    /// Type of the error
    pub typ: ErrorType,

    /// Error message
    pub msg: &'a str,

    /// Helper messages
    pub helpers: String,
}

/// Implement functions to pretty-print the error's information
impl<'a> Error<'_> {
    /// Emits an error with colors and all information
    pub fn emit_error(&self, token: &Token) {
        let spaces = " ".repeat(token.lineno.to_string().len());
        eprintln!("\x1b[91merror\x1b[0m: \x1b[97m{}\x1b[0m", self.msg_for());
        eprintln!("  \x1b[94m-->\x1b[0m abc.gizmo:{}:{}", token.lineno, token.col);
        eprintln!("\x1b[94m{} | \x1b[0m", spaces);
        eprintln!("\x1b[94m{} | {}\x1b[0m", token.lineno, token.line);
        eprintln!("\x1b[94m{} |\x1b[0m{}{} {}", spaces, " ".repeat(token.lineno.to_string().len() + token.col), "\x1b[91m^\x1b[0m".repeat(token.value.len()), self.msg);
        for helper in self.helpers.split('\n') {
            eprintln!("   {}{}", " ".repeat(token.lineno.to_string().len() + token.col), helper);
        }
    }
    
    /// Returns the base error message for each error type
    fn msg_for(&self) -> &str {
        return match self.typ {
            ErrorType::StringWithoutEnd => "String never ends",
            ErrorType::UnknownChar => "Unknown character",
            ErrorType::DecTooManyDots => "Too many dots in floating point number",
            ErrorType::DecNotFound => "Expected number after dot in floating point number",
            ErrorType::ExpectedToken => "Expected token",
        }
    }
    
    /// Adds another help message to an error
    pub fn note(&mut self, msg: &str) {
        self.helpers.push_str(format!("\n{}", msg).as_str());
    }
}

/// An enum with all the possible error types 
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    /// Lexer errors
    StringWithoutEnd,
    UnknownChar,
    DecTooManyDots,
    DecNotFound,

    /// Parser errors
    ExpectedToken,
}
