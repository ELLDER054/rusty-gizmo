pub mod lexer;
mod ast;
mod generator;
pub mod symbol;

use std::fs::File;
use std::io::Write;
use std::process::Command;
use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::Error;
use self::lexer::error::ErrorType;
use self::ast::Node;
use self::ast::Expression;
use self::generator::Generator;
use self::symbol::SymbolController;
use self::symbol::SymbolType;

// Stores information for a "Parser"
pub struct Parser {
    pub pos: usize, // Current position in tokens
    pub tokens: Vec<Token>, // Input list of tokens
    pub symtable: SymbolController,
    pub id_c: usize,
}

// Implement functions for a "Parser"
impl Parser {

    // Returns true if the type of the current token equals "t"
    fn expect_type(&mut self, t: TokenType) -> bool {
        // Returns false if the position is at the end of the input
        if self.pos >= self.tokens.len() {
            return false
        }
        if self.tokens[self.pos].typ == t {
            // Increment the position if we succeed
            self.pos += 1;
            return true;
        }
        return false;
    }

    // Returns a recursively parsed expression node
    fn expression(&mut self, start: usize) -> Expression {
        self.pos = start;
        return self.equality(start);
    }

    // Parse an equality operation (i.e., a == b, a != b)
    fn equality(&mut self, start: usize) -> Expression {
        self.pos = start;
        // Parse the left hand side of the expression
        let mut expr = self.comparison(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::EqualEqual) || self.expect_type(TokenType::NotEqual) {
            let save = self.pos - 1;
            let right = self.comparison(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr), right: Box::new(right)};
        }

        return expr;
    }

    fn comparison(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.term(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::GreaterThan) || self.expect_type(TokenType::LessThan) || self.expect_type(TokenType::GreaterEqual) || self.expect_type(TokenType::LessEqual) {
            let save = self.pos - 1;
            let right = self.term(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr), right: Box::new(right)};
        }

        return expr;
    }

    fn term(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.factor(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::Plus) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.factor(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr), right: Box::new(right)};
        }

        return expr;
    }

    fn factor(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.unary(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::Star) || self.expect_type(TokenType::Slash) {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr), right: Box::new(right)};
        }

        return expr;
    }

    fn unary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect a unary operator followed by an expression
        if self.expect_type(TokenType::Not) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            return Expression::UnaryOperator {oper: self.tokens[save].value.as_str().to_string(), child: Box::new(right)};
        }
        return self.primary(start);
    }

    fn primary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect an expression constant
        let int: bool = self.expect_type(TokenType::Int);
        if int {
            return Expression::Int(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let dec: bool = self.expect_type(TokenType::Dec);
        if dec {
            return Expression::Dec(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let string: bool = self.expect_type(TokenType::Str);
        if string {
            return Expression::Str(self.tokens[self.pos - 1].value.as_str().to_string());
        }
        let boolean: bool = self.expect_type(TokenType::Bool);
        if boolean {
            return Expression::Bool(if self.tokens[self.pos - 1].value == "true" {true} else {false});
        }
        let id: bool = self.expect_type(TokenType::Id);
        if id {
            let sym = self.symtable.find_error((&self.tokens[self.pos - 1].value).to_string(), SymbolType::Var, Vec::new());
            return Expression::Id((&self.tokens[self.pos - 1].value).to_string(), sym.typ.clone(), Vec::new(), sym.gen_id.clone());
        }

        // Nothing is found, return Non
        return Expression::Non;
    }

    // Parses a function call
    fn func_call(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect an identifier
        let id: bool = self.expect_type(TokenType::Id);
        let save = self.pos - 1;
        if !id {
            self.pos = start;
            return Node::Non;
        }
        
        // Expect a left parenthesis to follow the identifier
        let lp: bool = self.expect_type(TokenType::LeftParen);
        if !lp {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected left parenthesis after this identifier", helpers: "help: Insert a left parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        // Parse a series of arguments followed by commas. If the comma is
        // not found, stop parsing arguments
        let mut args: Vec<Box<Expression>> = Vec::new();
        loop {
            let expr = self.expression(self.pos);
            if expr == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected argument", helpers: "help: Insert an expression".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            
            // Give the type-checker a chance to find an error
            if expr.validate() == "error" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Mismatched types", helpers: "help: Mismatched types within this expression".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            args.push(Box::new(expr));
            let comma: bool = self.expect_type(TokenType::Comma);
            if !comma {
                break;
            }
        }

        // Expect a right parenthesis to follow the arguments
        let rp: bool = self.expect_type(TokenType::RightParen);
        if !rp {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected right parenthesis after this argument", helpers: "help: Insert a right parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        
        // Expect a semi-colon after the right parenthesis
        let semi: bool = self.expect_type(TokenType::SemiColon);
        if  !semi {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this parenthesis", helpers: "help: Insert a semi-colon after this parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        return Node::FuncCall {id: (&self.tokens[save].value).to_string(), args: args};
    }

    // Parses a let statement
    fn let_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect the "let" keyword
        let key: bool = self.expect_type(TokenType::Let);
        if !key {
            self.pos = start;
            return Node::Non;
        }
        
        // Expect an identifier after the "let" keyword
        let id: bool = self.expect_type(TokenType::Id);
        if !id {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected identifier after this let", helpers: "help: Insert an identifier".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let save: usize = self.pos - 1;

        // Expect an equals sign after the identifer
        let mut eq: bool = self.expect_type(TokenType::Equal);
        let mut save_type_pos = 0;
        if !eq {

            // If we don't find an equals sign, look for a colon
            eq = self.expect_type(TokenType::Colon);
            if !eq {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign or colon after this identifier", helpers: "help: Insert an equals sign or a colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }

            // Expect a type after the colon
            let typ: bool = self.expect_type(TokenType::Type);
            if !typ {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected type after this colon", helpers: "help: Insert a type after this colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            save_type_pos = self.pos - 1;

            // Once we find a type, look again for an equals sign
            eq = self.expect_type(TokenType::Equal);
            if !eq {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign after this colon", helpers: "help: Insert an equals sign here".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
        }

        // Look for an expression for the value of the let statement
        let expr: Expression = self.expression(self.pos);
        if expr == Expression::Non {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this equals sign", helpers: "help: Take away the equals sign or insert an expression after this equals sign".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        
        // If the type of the expression has a type-checker error, print error
        if expr.validate() == "error" {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "This type does not match the type of the expression", helpers: "".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        // If the type of the expression doesn't match the type given by programmer, print error
        if save_type_pos != 0 && self.tokens[save_type_pos].value != expr.validate() {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "The type of the variable and the type of the expression do not match", helpers: format!("help: Change this to {}", expr.validate())};
            err.emit_error(&self.tokens[save_type_pos]);
            std::process::exit(1);
        }

        // Expect a semi-colon after the expression
        let semi: bool = self.expect_type(TokenType::SemiColon);
        if  !semi {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this expression", helpers: "help: Insert a semi-colon after this expression".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        self.symtable.add_symbol((&self.tokens[save].value).to_string(), expr.validate().to_string(), SymbolType::Var, format!("%.{}", self.id_c), Vec::new());
        self.id_c += 1;
        return Node::Let {id: (&self.tokens[save].value).to_string(), expr: Box::new(expr), gen_id: format!("%.{}", self.id_c - 1)};
    }

    // Parses a series of statements based off of the input tokens
    fn program(&mut self, mut max_len: usize) {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
        let mut gen: Generator = Generator {name_num: 0, str_num: 0, code: "define i32 @main() {\nentry:\n".to_string(), ends: "".to_string(), begins: "".to_string()};

        // Stores each statement's node
        let mut nodes: Vec<Node> = Vec::new();

        // Loop through the tokens
        while self.pos < max_len {
            let let_stmt = self.let_statement(self.pos);

            // Check for a let statement
            if let_stmt != Node::Non {
                if let Node::Let {id, expr, gen_id} = let_stmt {
                    // Push the Node onto the nodes list
                    nodes.push(Node::Let {id, expr, gen_id});
                    continue;
                }
            }

            // Check for a function call
            let func_call = self.func_call(self.pos);
            if func_call != Node::Non {
                if let Node::FuncCall {id, args} = func_call {
                    // Push the Node onto the nodes list
                    nodes.push(Node::FuncCall {id, args});
                    continue;
                }
            }
            println!("Failure");
            break;
        }
        
        // Generate LLVM ir for each of the nodes
        gen.gen_all(nodes);
        gen.code.push_str("\tret i32 0\n}\n");
        gen.code = format!("{}{}{}", gen.begins, gen.code, gen.ends);

        // Open an output file and write to it
        let mut out_file = File::create("a.ll").expect("Couldn't create the output file");
        out_file.write_all(gen.code.as_bytes()).expect("Couldn't write to the output file");
        Command::new("lli a.ll");
    }

    // Resets the position and calls "program()"
    pub fn parse(&mut self) {
        self.pos = 0;
        self.program(0);
    }
}
