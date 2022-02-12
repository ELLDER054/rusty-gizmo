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
    let mut file_name:      String = String::new();
    let mut out_file_name:  String = String::from("a.out");
    let mut out_ir_name:    String = String::from("a.ll");
    let mut emit_llvm:      bool   = false;

    // Collect the command line arguments into a vector
    let args: Vec<String> = env::args().collect();

    // If there are no arguments, print the version
    if args.len() == 1 {
        println!("Gizmo v1.0");
        return;
    }

    // Parse the arguments
    let mut arg_num = 1;
    while arg_num < args.len() {
        if !args[arg_num].starts_with('-') {
            // An argument with no '-' is a file
            file_name = args[arg_num].clone();
            arg_num += 1;
        } else if args[arg_num] == "-o" {
            // An output file
            arg_num += 1;
            out_file_name = args[arg_num].clone();
            out_ir_name = args[arg_num].clone();
            arg_num += 1;
        } else if args[arg_num] == "-emit-llvm" {
            // Whether or not to emit llvm
            arg_num += 1;
            emit_llvm = true;
        }
    }

    // Open the input file
    let file = fs::read_to_string(file_name).unwrap();

    // Compile the input file and store the llvm ir in 'output'
    let output = compile(file);

    // Open an output file and write to it
    let mut out_file: File;
    if emit_llvm == true {
        out_file = File::create(out_ir_name.clone()).expect("Couldn't create the output file");
    } else {
        out_file = File::create("a.ll").expect("Couldn't create the output file");
    }
    out_file.write_all((&output).as_bytes()).expect("Couldn't write to the output file");
    
    if emit_llvm == false {
        // Call 'llc' on the created file
        Command::new("llc").args(&["a.ll", "--relocation-model=pic", "-filetype=obj"]).output().expect("Failed to call llc");
        Command::new("rm").arg("a.ll").output().expect("Failed to call rm1");
        // Call 'gcc' on 'a.o'
        Command::new("gcc").args(&["a.o", "-o", out_file_name.as_str()]).output().expect("Failed to call gcc");
        Command::new("rm").arg("a.o").output().expect("Failed to rm2");
    }
}

/// Compiles the given code
fn compile(code: String) -> String {
    // Create a lexer
    let mut lexer: Lexer = Lexer {pos: 0, code: code, col: 0};

    // Lex the input
    let tokens: Vec<Token> = lexer.lex();

    // Create a symbol-table and a parser
    let sym_table = SymbolController {current: Scope {parent: None, children: Vec::new(), var_symbols: Vec::new(), func_symbols: Vec::new(), struct_symbols: Vec::new()}};
    let mut parser: Parser = Parser {pos: 0, tokens: tokens, symtable: sym_table, id_c: 0};

    // Parse the tokens
    let ast = parser.parse();

    // Create a generator
    let mut generator = Generator::construct();
    
    // Generate llvm ir for the ast
    generator.generate(ast);
    generator.destruct();
    generator.ir_b.code
}
