use super::token::Token;
pub fn emit_error(msg: String, helpers: String, token: &Token, typ: ErrorType) {
    let spaces = " ".repeat(token.lineno.to_string().len());
    eprintln!("\x1b[91merror\x1b[0m: \x1b[97m{}\x1b[0m", msg_for(typ));
    eprintln!("  \x1b[94m-->\x1b[0m abc.gizmo:{}:{}", token.lineno, token.col);
    eprintln!("\x1b[94m{} | \x1b[0m", spaces);
    eprintln!("\x1b[94m{} | {}\x1b[0m", token.lineno, token.line);
    eprintln!("\x1b[94m{} |\x1b[0m{}{} {}", spaces, " ".repeat(token.lineno.to_string().len() + token.col), "\x1b[91m^\x1b[0m".repeat(token.value.len()), msg);
    for helper in helpers.split('\n') {
        eprintln!("   {}{}", " ".repeat(token.lineno.to_string().len() + token.col), helper);
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
#[derive(Debug, PartialEq, Eq)]
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
