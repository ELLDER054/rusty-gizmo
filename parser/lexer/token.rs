// Stores each token's information
pub struct Token {
    pub typ: TokenType,
    pub value: String,
    pub lineno: usize,
    pub col: usize,
    pub line: String,
}

// An enum with all the tokens for the language
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    PlusEqual,    // +=
    PlusPlus,     // ++
    Plus,         // +
    DashEqual,    // -=
    DashDash,     // --
    Dash,         // -
    StarEqual,    // *=
    StarStar,     // **
    Star,         // *
    SlashEqual,   // /=
    SlashSlash,   // //
    Slash,        // /
    NotEqual,     // !=
    SemiColon,    // ;
    LeftParen,    // (
    RightParen,   // )
    Equal,        // =
    EqualEqual,   // ==
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Colon,        // :
    Comma,        // ,
    Let,
    If,
    While,
    For,
    Id,
    Int,
    Dec,
    Bool,
    Str,
    Type,
    And,
    Or,
    Not,
    Error,
}
