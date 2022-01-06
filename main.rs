mod lexer;

use lexer::Lexer;
use lexer::token::Token;

fn main () {
    let mut lexer: Lexer = Lexer {pos: 0, code: "let abc = \"abc\";".to_string(), col: 0};
    let tokens: Vec<Token> = lexer.lex();
    for tok in tokens.iter() {
        println!("Value: {}, Lineno: {}, Col: {}, Line: {}", tok.value, tok.lineno, tok.col, tok.line);
    }
}
