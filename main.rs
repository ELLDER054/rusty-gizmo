mod lexer;

use std::env;
use std::fs;
use lexer::Lexer;
use lexer::token::Token;

fn main () {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Gizmo v1.0");
        return;
    }
	let file: String = fs::read_to_string(&args[1]).unwrap();
    let mut lexer: Lexer = Lexer {pos: 0, code: file, col: 0};
    let tokens: Vec<Token> = lexer.lex();
    for token in tokens.iter() {
        println!("VALUE: {}, LINENO: {}, COL: {}, LINE: {}", token.value, token.lineno, token.col, token.line);
    }
}
