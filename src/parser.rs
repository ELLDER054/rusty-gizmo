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

/// Stores information for a "Parser"
pub struct Parser {
    /// Current position in tokens
    pub pos: usize,

    /// Input list of tokens
    pub tokens: Vec<Token>,

    /// Initialize a symbol table
    pub symtable: SymbolController,

    /// Counts the identifiers created
    pub id_c: usize,
}

/// Implement functions for a "Parser"
impl Parser {
    /// Returns the current token if the type of the current token equals "t"
    fn expect_type(&mut self, t: TokenType) -> Option<String> {
        // Returns None if the position is at the end of the input
        if self.pos >= self.tokens.len() {
            return None
        }
        if self.tokens[self.pos].typ == t {
            // Increment the position if we succeed
            self.pos += 1;
            return Some((&self.tokens[self.pos - 1].value).to_string());
        }
        return None;
    }

    /// Returns a recursively parsed expression node
    fn expression(&mut self, start: usize) -> Expression {
        self.pos = start;
        return self.equality(start);
    }

    /// Parse an equality operation
    /// # Example
    /// `a == b` or `a != b`
    fn equality(&mut self, start: usize) -> Expression {
        self.pos = start;
        // Parse the left hand side of the expression
        let mut expr = self.comparison(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::EqualEqual) != None || self.expect_type(TokenType::NotEqual) != None {
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

    /// Parse a comparison operation
    /// # Example
    /// `a < b`, `a > b`, `a <= b` or `a >= b`
    fn comparison(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.term(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::GreaterThan) != None || self.expect_type(TokenType::LessThan) != None || self.expect_type(TokenType::GreaterEqual) != None || self.expect_type(TokenType::LessEqual) != None {
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

    /// Parse a term
    /// # Example
    /// `a + b` or `a - b`
    fn term(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.factor(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::Plus) != None || self.expect_type(TokenType::Dash) != None {
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

    /// Parse a factor
    /// # Example
    /// `a * b` or `a / b`
    fn factor(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.unary(self.pos);

        // Continue expecting an operator with another side of the expression after it
        while self.expect_type(TokenType::Star) != None || self.expect_type(TokenType::Slash) != None {
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

    /// Parse a unary operation
    /// # Example
    /// `-a` or `not a`
    fn unary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect a unary operator followed by an expression
        if self.expect_type(TokenType::Not) != None || self.expect_type(TokenType::Dash) != None {
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

    /// Parse a constant value
    fn primary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect an expression constant
        let int = self.expect_type(TokenType::Int);
        if int != None {
            return Expression::Int(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let dec = self.expect_type(TokenType::Dec);
        if dec != None {
            return Expression::Dec(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let string = self.expect_type(TokenType::Str);
        if string != None {
            return Expression::Str(self.tokens[self.pos - 1].value.as_str().to_string());
        }
        let boolean = self.expect_type(TokenType::Bool);
        if boolean != None {
            return Expression::Bool(if self.tokens[self.pos - 1].value == "true" {true} else {false});
        }
        let ns = self.new_struct(self.pos);
        if ns != Expression::Non {
            return ns;
        }
        let rec_id = self.rec_identifier(self.pos);
        if rec_id != Expression::Non {
            return rec_id;
        }
        let id = self.expect_type(TokenType::Id);
        if id != None {
            let sym = self.symtable.find_error((&self.tokens[self.pos - 1].value).to_string(), SymbolType::Var, None);
            return Expression::Id((&self.tokens[self.pos - 1].value).to_string(), sym.typ.clone(), Vec::new(), sym.gen_id.clone());
        }

        // Nothing is found, return Non
        return Expression::Non;
    }

    /// Parse a struct initialization
    /// # Example
    /// new Foo(5, 6, 7)
    fn new_struct(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect the 'new' keyword
        let key = self.expect_type(TokenType::New);
        if key != None {
            // If we found the 'new' keyword, expect an identifier
            let id = self.expect_type(TokenType::Id);
            if id == None {
                self.pos = start;
                return Expression::Non;
            }

            // Expect a left parenthesis
            let lp = self.expect_type(TokenType::LeftParen);
            if lp == None {
                self.pos = start;
                return Expression::Non;
            }

            // Create a vector to store the fields
            let mut fields: Vec<Expression> = Vec::new();

            // Expect multiple fields followed by commas
            loop {
                // Expect an expression
                let expr = self.expression(self.pos);
                if expr == Expression::Non {
                    self.pos = start;
                    return Expression::Non;
                }

                // Push the expression to fields
                fields.push(expr);

                // Expect a comma
                // Not finding a comma tells the compiler to stop parsing fields
                let comma = self.expect_type(TokenType::Comma);
                if comma == None {
                    break;
                }
            }
            
            // Expect a right parenthesis
            let rp = self.expect_type(TokenType::RightParen);
            if rp == None {
                self.pos = start;
                return Expression::Non;
            }
            
            // Return the struct initialization
            return Expression::NewStruct {id: id.unwrap(), fields: fields};
        }
        return Expression::Non;
    }

    /// Parses an identifier
    /// # Example
    /// `a`, `a.b`, `a.b.c`, etc
    fn rec_identifier(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Expect a struct initialization
        // This is because it is possible to have something like:
        // let bar = new Foo(5, 6, 7).bar
        let mut begin = self.new_struct(self.pos);
        if begin == Expression::Non {
            // If we didn't find the struct initialization, we can just expect an identifier
            let id = self.expect_type(TokenType::Id);
            if id == None {
                self.pos = start;
                return Expression::Non;
            }

            // Find the symbol in the symbol table, printing an error if it was not found
            let mut id_sym = self.symtable.find(id.clone().unwrap(), SymbolType::Var, None);
            if id_sym == None {
                id_sym = Some(self.symtable.find_error(id.clone().unwrap(), SymbolType::Struct, None));
            }
            begin = Expression::Id(id.clone().unwrap(), id_sym.unwrap().typ.clone(), Vec::new(), id_sym.unwrap().gen_id.clone());
        }

        // Expect a dot
        let dot = self.expect_type(TokenType::Dot);
        if dot == None {
            self.pos = start;
            return Expression::Non;
        }

        // Expect another identifier
        let id2 = self.expect_type(TokenType::Id);
        if id2 == None {
            self.pos = start;
            return Expression::Non;
        }

        // Find the field number of the field
        let mut field_num = 0;
        println!("{}", begin.clone().validate());
        let sym = self.symtable.find(begin.clone().validate().to_string(), SymbolType::Struct, None);
        if sym == None {
            return Expression::Non;
        }
        for field in sym.unwrap().arg_types.iter() {
            if field.0 == id2.clone().unwrap() {
                println!("327");
                return Expression::StructDot {id: Box::new(begin.clone()), id2: id2.unwrap(), typ: sym.unwrap().arg_types[0].clone().1, field_num: field_num};
            }
            field_num += 1;
        }
        self.pos = start;
        return Expression::Non;
    }

    /// Parses a type
    /// # Example
    /// `struct Foo` or `int`
    fn rec_type(&mut self, start: usize) -> Option<String> {
        self.pos = start;

        // Expect the 'struct' keyword
        let key = self.expect_type(TokenType::Struct);
        if key == None {
            // If the struct keyword was not found, find a basic type and return
            let typ = self.expect_type(TokenType::Type);
            if typ == None {
                self.pos = start;
                return None;
            }
            return Some(typ.clone().unwrap());
        }

        // Expect an identifier
        // The identifier must also be a struct
        let typ = self.expect_type(TokenType::Id);
        if typ == None || self.symtable.find(typ.clone().unwrap(), SymbolType::Struct, None) == None {
            self.pos = start;
            return None;
        }
        return Some(format!("{}", typ.unwrap()).clone());
    }

    /// Parses a struct definition
    /// # Example
    /// struct Foo {
    ///     bar: int
    /// }
    fn struct_def(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect the 'struct' keyword
        let key = self.expect_type(TokenType::Struct);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        // Expect an identifier to follow the keyword
        let id = self.expect_type(TokenType::Id);
        if id == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected an identifier", helpers: "help: Insert an identifier".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        
        // Expect a left curly brace to follow the identifier
        let lb = self.expect_type(TokenType::LeftBrace);
        if lb == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected left curly brace after this identifier", helpers: "help: Insert a left curly brace".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        // Parse a series of struct fields followed by commas. If the comma is
        // not found, stop parsing fields
        let mut fields: Vec<(String, String)> = Vec::new();
        loop {
            // Expect an identifier
            let id = self.expect_type(TokenType::Id);
            if id == None {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected an identifier", helpers: "help: Insert an identifier after this token".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }

            // Expect a colon after the identifier
            let colon = self.expect_type(TokenType::Colon);
            if colon == None {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected a colon", helpers: "help: Insert a colon after this identifier".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }

            // Expect a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected a type", helpers: "help: Insert a type after this colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }

            // Push the field onto the list of fields
            fields.push((id.unwrap(), typ.unwrap()));

            // Expect a comma
            // If the comma isn't there, stop looking for more fields
            let comma = self.expect_type(TokenType::Comma);
            if comma == None {
                break;
            }
        }
        
        // Expect a left curly brace to follow the identifier
        let rb = self.expect_type(TokenType::RightBrace);
        if rb == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected right curly brace after this struct field", helpers: "help: Insert a right curly brace".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        // Add correct symbols
        let mut types: Vec<(String, String)> = Vec::new();
        for field in fields.iter() {
            types.push(field.clone());
        }
        self.symtable.add_symbol(id.clone().unwrap(), id.clone().unwrap(), SymbolType::Struct, format!("%.{}", self.id_c), types);
        
        // Return the struct node
        return Node::Struct {id: id.unwrap(), fields: fields};
    }

    /// Parses a function call
    /// # Example
    /// write(5);
    fn func_call(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect an identifier
        let id = self.expect_type(TokenType::Id);
        if id == None {
            self.pos = start;
            return Node::Non;
        }
        
        // Expect a left parenthesis to follow the identifier
        let lp = self.expect_type(TokenType::LeftParen);
        if lp == None {
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
            let comma = self.expect_type(TokenType::Comma);
            if comma == None {
                break;
            }
        }

        // Expect a right parenthesis to follow the arguments
        let rp = self.expect_type(TokenType::RightParen);
        if rp == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected right parenthesis after this argument", helpers: "help: Insert a right parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        
        // Expect a semi-colon after the right parenthesis
        let semi = self.expect_type(TokenType::SemiColon);
        if semi == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this parenthesis", helpers: "help: Insert a semi-colon after this parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        return Node::FuncCall {id: id.unwrap(), args: args};
    }

    /// Parses a let statement
    /// # Example
    /// let a = 5;
    /// or
    /// let a: int = 5;
    fn let_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect the "let" keyword
        let key = self.expect_type(TokenType::Let);
        if key == None {
            self.pos = start;
            return Node::Non;
        }
        
        // Expect an identifier after the "let" keyword
        let id = self.expect_type(TokenType::Id);
        if id == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected identifier after this let", helpers: "help: Insert an identifier".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        // Expect an equals sign after the identifer
        let mut eq = self.expect_type(TokenType::Equal);
        let mut save_type_pos = 0;
        if eq == None {
            // If we don't find an equals sign, look for a colon
            eq = self.expect_type(TokenType::Colon);
            if eq == None {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign or colon after this identifier", helpers: "help: Insert an equals sign or a colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }

            // Expect a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected type after this colon", helpers: "help: Insert a type after this colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            save_type_pos = self.pos - 1;

            // Once we find a type, look again for an equals sign
            eq = self.expect_type(TokenType::Equal);
            if eq == None {
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
        let semi = self.expect_type(TokenType::SemiColon);
        if semi == None {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this expression", helpers: "help: Insert a semi-colon after this expression".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }

        self.symtable.add_symbol(id.clone().unwrap(), expr.validate().to_string(), SymbolType::Var, format!("%.{}", self.id_c), Vec::new());
        self.id_c += 1;
        return Node::Let {id: id.unwrap(), expr: Box::new(expr), gen_id: format!("%.{}", self.id_c - 1)};
    }

    /// Parses a series of statements based off of the input tokens
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

            // Check for a struct definition
            let struct_def = self.struct_def(self.pos);
            if struct_def != Node::Non {
                if let Node::Struct {id, fields} = struct_def {
                    // Push the Node onto the nodes list
                    nodes.push(Node::Struct {id: id, fields: fields});
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

    /// Resets the position and calls "program()"
    pub fn parse(&mut self) {
        self.pos = 0;
        self.program(0);
    }
}
