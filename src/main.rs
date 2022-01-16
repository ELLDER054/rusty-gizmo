mod parser;

use std::env;
use std::fs;
use parser::lexer::Lexer;
use parser::lexer::token::Token;
use parser::Parser;
use parser::symbol::Scope;
use parser::symbol::SymbolController;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Gizmo v1.0");
        return;
    }
    let file: String = fs::read_to_string(&args[1]).unwrap();
    let mut lexer: Lexer = Lexer {pos: 0, code: file, col: 0};
    let tokens: Vec<Token> = lexer.lex();
    for token in tokens.iter() {
        println!("{:?}", token);
    }
    let sym_table = SymbolController {current: Scope {parent: None, children: Vec::new(), symbols: Vec::new()}};
    let mut parser: Parser = Parser {pos: 0, tokens: tokens, symtable: sym_table, id_c: 0};
    parser.parse();
}
