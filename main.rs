mod parser;

use std::env;
use std::fs;
use parser::lexer::Lexer;
use parser::lexer::token::Token;
use parser::Parser;
use parser::symbol::SymbolController;
use parser::symbol::SymbolTable;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Gizmo v1.0");
        return;
    }
    let file: String = fs::read_to_string(&args[1]).unwrap();
    let mut lexer: Lexer = Lexer {pos: 0, code: file, col: 0};
    let tokens: Vec<Token> = lexer.lex();
    /*for token in tokens.iter() {
        println!("VALUE: {}, LINENO: {}, COL: {}, LINE: {}", token.value, token.lineno, token.col, token.line);
    }*/
    let mut parser: Parser = Parser {pos: 0, tokens: tokens, symtable: SymbolController {current: SymbolTable {parent: None, child: None, group: Vec::new()}, global: SymbolTable {parent: None, child: None, group: Vec::new()}}, id_c: 0};
    parser.parse();
}
