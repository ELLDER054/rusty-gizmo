use colored::Colorize;

use super::token::Token;

pub fn emit_error(msg: String, helpers: &str, token: &Token, typ: ErrorType) {
    let spaces = " ".repeat(token.lineno.to_string().len());
    eprintln!("{}{}", "error: ".bright_red(), msg_for(typ.clone()).bright_white());
    eprintln!("{} abc.gizmo:{}:{}", "  -->".bright_blue(), token.lineno, token.col);
    eprintln!("{}{}", spaces, " | ".bright_blue());
    eprintln!("{}{}{}", token.lineno.to_string().bright_blue(), " | ".bright_blue(), token.line);
    eprintln!("{}{}{}{} {}", spaces, " |".bright_blue(), " ".repeat(token.lineno.to_string().len() + token.col), "^".repeat(token.value.len()).bright_blue(), msg.bright_white());

    for helper in helpers.to_string().split('\n') {
        eprintln!("   {}{}", " ".repeat(token.lineno.to_string().len() + token.col), helper.bright_white());
    }
    std::process::exit(1);
}

/// Returns the base error message for each error type
fn msg_for(typ: ErrorType) -> String {
    match typ {
        ErrorType::UnexpectedEOF   => "Unexpected EOF when parsing",
        ErrorType::UnknownChar     => "Unknown character",
        ErrorType::DecTooManyDots  => "Too many dots in floating point number",
        ErrorType::DecNotFound     => "Expected number after dot in floating point number",

        ErrorType::ExpectedToken   => "Expected token",
        ErrorType::MismatchedTypes => "Mismatched types",
        ErrorType::UndefinedArray  => "This array has no type"
    }.to_string()
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
    UndefinedArray
}
