mod parser;

use std::env;
use std::fs;
use parser::lexer::Lexer;
use parser::lexer::token::Token;
use parser::Parser;
use parser::generator::Generator;
use parser::symbol::Scope;
use parser::symbol::SymbolController;
use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Gizmo v1.0");
        return;
    }
    let file: String = fs::read_to_string(&args[1]).unwrap();
    compile(file);
}

fn compile(code: String) {
    let mut lexer: Lexer = Lexer {pos: 0, code: code, col: 0};
    let tokens: Vec<Token> = lexer.lex();
    let sym_table = SymbolController {current: Scope {parent: None, children: Vec::new(), symbols: Vec::new()}};
    let mut parser: Parser = Parser {pos: 0, tokens: tokens, symtable: sym_table, id_c: 0};
    let ast = parser.parse();
    let mut generator = Generator::construct();
    generator.generate(ast);
    generator.destruct();

    // Open an output file and write to it
    let mut out_file = File::create("a.ll").expect("Couldn't create the output file");
    out_file.write_all(generator.ir_b.code.as_bytes()).expect("Couldn't write to the output file");
    Command::new("lli a.ll");
}

#[test]
fn test_compile() {
    assert_eq!(
    "int a = 5"
    )
}
