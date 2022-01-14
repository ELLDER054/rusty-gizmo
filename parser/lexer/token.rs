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
    PlusEqual,
    PlusPlus,
    Plus,
    DashEqual,
    DashDash,
    Dash,
    StarEqual,
    StarStar,
    Star,
    SlashEqual,
    SlashSlash,
    Slash,
    NotEqual,
    SemiColon,
    LeftParen,
    RightParen,
    Let,
    If,
    While,
    For,
    Id,
    Int,
    Dec,
    Str,
    Equal,
    EqualEqual,
    Not,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Type,
    And,
    Or,
    Bool,
    Colon,
    Comma,
    Error,
}
