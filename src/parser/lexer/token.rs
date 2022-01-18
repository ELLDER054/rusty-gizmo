/// Stores each token's information
#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    /// Type of the token
    pub typ: TokenType,

    /// Value that is stored in the token
    pub value: String,

    /// Line number that the token is on
    pub lineno: usize,

    /// Column that the token is on
    pub col: usize,

    /// Line that the token is on (for printing errors)
    pub line: String,
}

/// An enum with all the tokens for the language
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
    LeftBrace,    // {
    RightBrace,   // }
    Equal,        // =
    EqualEqual,   // ==
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Colon,        // :
    Comma,        // ,
    Dot,          // .
    Let,
    New,
    Struct,
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
