pub mod lexer;
mod ast;
mod sgenerator;
pub mod symbol;

use std::fs::File;
use std::io::Write;
use std::process::Command;
use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::ErrorType;
use self::lexer::error::emit_error;
use self::ast::Node;
use self::ast::Expression;
use self::sgenerator::Generator;
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
    /// Returns the current token if the type of the current token equals 't'
    fn match_t(&mut self, t: TokenType) -> Option<String> {
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

        // Continue matching an operator with another side of the expression after it
        while self.match_t(TokenType::EqualEqual) != None || self.match_t(TokenType::NotEqual) != None {
            let save = self.pos - 1;
            let right = self.comparison(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                emit_error("Expected an expression after this operator".to_string(), "help: Take away the operator or insert an expression after this operator".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr.clone()), right: Box::new(right.clone())};
            if expr.validate() == "error" {
                emit_error("Mismatched types within this expression".to_string(), format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.validate()), &self.tokens[save], ErrorType::MismatchedTypes);
            }
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

        // Continue matching an operator with another side of the expression after it
        while self.match_t(TokenType::GreaterThan) != None || self.match_t(TokenType::LessThan) != None || self.match_t(TokenType::GreaterEqual) != None || self.match_t(TokenType::LessEqual) != None {
            let save = self.pos - 1;
            let right = self.term(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                emit_error("Expected an expression after this operator".to_string(), "help: Take away the operator or insert an expression after this operator".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr.clone()), right: Box::new(right.clone())};
            if expr.validate() == "error" {
                emit_error("Mismatched types within this expression".to_string(), format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.validate()), &self.tokens[save], ErrorType::MismatchedTypes);
            }
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

        // Continue matching an operator with another side of the expression after it
        while self.match_t(TokenType::Plus) != None || self.match_t(TokenType::Dash) != None {
            let save = self.pos - 1;
            let right = self.factor(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                emit_error("Expected an expression after this operator".to_string(), "help: Take away the operator or insert an expression after this operator".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr.clone()), right: Box::new(right.clone())};
            if expr.validate() == "error" {
                emit_error("Mismatched types within this expression".to_string(), format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.clone().validate()), &self.tokens[save], ErrorType::MismatchedTypes);
            }
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

        // Continue matching an operator with another side of the expression after it
        while self.match_t(TokenType::Star) != None || self.match_t(TokenType::Slash) != None {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                emit_error("Expected an expression after this operator".to_string(), "help: Take away the operator or insert an expression after this operator".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            let left_type = expr.clone();
            expr = Expression::BinaryOperator {oper: self.tokens[save].value.as_str().to_string(), left: Box::new(expr.clone()), right: Box::new(right.clone())};
            if expr.validate() == "error" {
                emit_error("Mismatched types within this expression".to_string(), format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left_type.validate(), right.validate()), &self.tokens[save], ErrorType::MismatchedTypes);
            }
        }

        return expr;
    }

    /// Parse a unary operation
    /// # Example
    /// `-a` or `not a`
    fn unary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Match a unary operator followed by an expression
        if self.match_t(TokenType::Not) != None || self.match_t(TokenType::Dash) != None {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right isn't found, print an error
            if right == Expression::Non {
                emit_error("Expected an expression after this operator".to_string(), "help: Take away the operator or insert an expression after this operator".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            let oper = Expression::UnaryOperator {oper: (&self.tokens[save]).value.clone(), child: Box::new(right.clone())};
            if oper.validate() == "error" {
                emit_error("Mismatched types within expression".to_string(), format!("Attempted to use '{}' with type '{}'", (&self.tokens[save]).value, right.validate()), &self.tokens[save], ErrorType::MismatchedTypes);
            }
            return oper;
        }
        return self.primary(start);
    }

    fn strip_arr(&self, s: String) -> String {
        match s.as_str() {
            "string" => "char",
            _ => &s[0..s.len() - 2]
        }.to_string()
    }

    /// Parse a constant value
    fn primary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Match an expression constant
        let int = self.match_t(TokenType::Int);
        if int != None {
            return Expression::Int(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let dec = self.match_t(TokenType::Dec);
        if dec != None {
            return Expression::Dec(self.tokens[self.pos - 1].value.parse().unwrap());
        }
        let string = self.match_t(TokenType::Str);
        if string != None {
            if self.match_t(TokenType::LeftBracket) == None {
                return Expression::Str(self.tokens[self.pos - 1].value.as_str().to_string());
            } else {
                let e = self.expression(self.pos);
                self.match_t(TokenType::RightBracket);
                return Expression::IndexedValue {src: Box::new(Expression::Str(string.unwrap())), index: Box::new(e), new_typ: "char".to_string()};
            }
        }
        let boolean = self.match_t(TokenType::Bool);
        if boolean != None {
            return Expression::Bool(if self.tokens[self.pos - 1].value == "true" {true} else {false});
        }
        let lb = self.match_t(TokenType::LeftBracket);
        if lb != None {
            let save = self.pos - 1;
            let mut values: Vec<Expression> = Vec::new();
            loop {
                let array_value = self.expression(self.pos);
                if array_value == Expression::Non {
                    break;
                }
                values.push(array_value);
                let comma = self.match_t(TokenType::Comma);
                if comma == None {
                    break;
                }
            }
            if values.len() == 0 {
                emit_error("This array does not have a type".to_string(), "help: Consider explicitly stating the array's type".to_string(), &self.tokens[save], ErrorType::UndefinedArray);
            }
            let rb = self.match_t(TokenType::RightBracket);
            if rb != None {
                let first_typ = (&values[0]).validate();
                for value in values[1..values.len()].iter() {
                    if value.validate() != first_typ {
                        emit_error("This array has mismatched types".to_string(), "".to_string(), &self.tokens[save], ErrorType::MismatchedTypes);
                    }
                }
                let typ = format!("{}[]", &first_typ);
                if self.match_t(TokenType::LeftBracket) == None {
                    return Expression::Array {values: values.clone(), typ: typ};
                } else {
                    let e = self.expression(self.pos);
                    self.match_t(TokenType::RightBracket);
                    return Expression::IndexedValue {src: Box::new(Expression::Array {values: values.clone(), typ: typ.clone()}), index: Box::new(e), new_typ: self.strip_arr(typ).to_string()};
                }
            }
            return Expression::Non;
        }
        let ns = self.new_struct(self.pos);
        if ns != Expression::Non {
            return ns;
        }
        let rec_id = self.rec_identifier(self.pos);
        if rec_id != Expression::Non {
            if self.match_t(TokenType::LeftBracket) == None {
                return rec_id;
            } else {
                let e = self.expression(self.pos);
                self.match_t(TokenType::RightBracket);
                return Expression::IndexedValue {src: Box::new(rec_id.clone()), index: Box::new(e), new_typ: self.strip_arr(rec_id.validate().to_string())};
            }
        }
        let id = self.match_t(TokenType::Id);
        if id != None {
            if self.match_t(TokenType::LeftBracket) == None {
                let sym = self.symtable.find_error((&self.tokens[self.pos - 1].value).to_string(), SymbolType::Var, None);
                return Expression::Id((&self.tokens[self.pos - 1].value).to_string(), sym.typ.clone(), sym.gen_id.clone());
            } else {
                let e = self.expression(self.pos);
                self.match_t(TokenType::RightBracket);
                let sym = self.symtable.find_error(id.clone().unwrap(), SymbolType::Var, None);
                let sym_arr = self.strip_arr(sym.typ.clone());
                return Expression::IndexedValue {src: Box::new(Expression::Id(id.unwrap(), sym.typ.clone(), sym.gen_id.clone())), index: Box::new(e), new_typ: sym_arr.to_string()};
            }
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
        let key = self.match_t(TokenType::New);
        if key != None {
            // If we found the 'new' keyword, match an identifier
            let id = self.match_t(TokenType::Id);
            if id == None {
                self.pos = start;
                return Expression::Non;
            }

            // Match a left parenthesis
            let lp = self.match_t(TokenType::LeftParen);
            if lp == None {
                emit_error("Expected a left parenthesis".to_string(), "help: Insert a left parenthesis after this identifier".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            // Create a vector to store the fields
            let mut fields: Vec<Expression> = Vec::new();

            // Expect multiple fields followed by commas
            loop {
                // Match an expression
                let expr = self.expression(self.pos);
                if expr == Expression::Non {
                    self.pos = start;
                    return Expression::Non;
                }

                // Push the expression to fields
                fields.push(expr);

                // Match a comma
                // Not finding a comma tells the compiler to stop parsing fields
                let comma = self.match_t(TokenType::Comma);
                if comma == None {
                    break;
                }
            }
            
            // Match a right parenthesis
            let rp = self.match_t(TokenType::RightParen);
            if rp == None {
                emit_error("Expected a right parenthesis".to_string(), "help: Insert a right parenthesis after this token".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
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

        // Match a struct initialization
        // This is because it is possible to have something like:
        // let bar = new Foo(5, 6, 7).bar
        let mut begin = self.new_struct(self.pos);
        if begin == Expression::Non {
            // If we didn't find the struct initialization, we can just match an identifier
            let id = self.match_t(TokenType::Id);
            if id == None {
                self.pos = start;
                return Expression::Non;
            }
            if self.match_t(TokenType::LeftBracket) == None {
                let mut id_sym = self.symtable.find(id.clone().unwrap(), SymbolType::Var, None);
                if id_sym == None {
                    id_sym = Some(self.symtable.find_error(id.clone().unwrap(), SymbolType::Struct, None));
                }
                begin = Expression::Id(id.unwrap(), id_sym.unwrap().typ.clone(), id_sym.unwrap().gen_id.clone());
            } else {
                let e = self.expression(self.pos);
                self.match_t(TokenType::RightBracket);
                let sym = self.symtable.find_error(id.clone().unwrap(), SymbolType::Var, None);
                let sym_arr = self.strip_arr(sym.typ.clone());
                begin = Expression::IndexedValue {src: Box::new(Expression::Id(id.unwrap(), sym.typ.clone(), sym.gen_id.clone())), index: Box::new(e), new_typ: sym_arr.to_string()};
            }
<<<<<<< HEAD
=======
            begin = Expression::Id(id.unwrap(), id_sym.unwrap().typ.clone(), id_sym.unwrap().gen_id.clone());
>>>>>>> cf018bafddcc009ee31174cac1b1d298c540b735
        }

        // Match a dot
        let dot = self.match_t(TokenType::Dot);
        if dot == None {
            self.pos = start;
            return Expression::Non;
        }

        // Match another identifier
        let id2 = self.match_t(TokenType::Id);
        if id2 == None {
            emit_error("Expected an identifier".to_string(), "help: Insert an identifier after this dot".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }

        // Find the field number of the field
        let mut field_num = 0;
        let sym = self.symtable.find_error(begin.clone().validate().to_string(), SymbolType::Struct, None);
        for field in sym.arg_types.iter() {
            if field.0 == id2.clone().unwrap() {
                return Expression::StructDot {id: Box::new(begin), id2: id2.unwrap(), typ: sym.arg_types[field_num as usize].clone().1, field_num: field_num};
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

        let typ = self.match_t(TokenType::Type);
        if typ != None {
            let mut brackets = String::new();
            while self.match_t(TokenType::LeftBracket) != None {
                let rb = self.match_t(TokenType::RightBracket);
                if rb == None {
                    emit_error("Expected a right bracket".to_string(), "help: Insert a right bracket after this left bracket".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
                }
                brackets.push_str("[]");
            }
            return Some(format!("{}{}", typ.unwrap(), brackets));
        }

        // Match an identifier
        // The identifier must also be a struct
        let typ = self.match_t(TokenType::Id);
        if typ == None || self.symtable.find(typ.clone().unwrap(), SymbolType::Struct, None) == None {
            emit_error("Expected a struct type".to_string(), "help: Insert a struct type after this token".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }
        let mut brackets = String::new();
        while self.match_t(TokenType::LeftBracket) != None {
            let rb = self.match_t(TokenType::RightBracket);
            if rb == None {
                emit_error("Expected a right bracket".to_string(), "help: Insert a right bracket after this left bracket".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            brackets.push_str("[]");
        }
        return Some(format!("{}{}", typ.unwrap(), brackets));
    }

    /// Parses a struct definition
    /// # Example
    /// struct Foo {
    ///     bar: int
    /// }
    fn struct_def(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect the 'struct' keyword
        let key = self.match_t(TokenType::Struct);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        // Match an identifier to follow the keyword
        let id = self.match_t(TokenType::Id);
        if id == None {
            emit_error("Expected an identifier".to_string(), "help: Insert an identifier after the 'struct' keyword".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }
        
        // Match a left curly brace to follow the identifier
        let lb = self.match_t(TokenType::LeftBrace);
        if lb == None {
            emit_error("Expected a left curly brace".to_string(), "help: Insert a left curly brace after this identifier".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }

        // Parse a series of struct fields followed by commas. If the comma is
        // not found, stop parsing fields
        let mut fields: Vec<(String, String)> = Vec::new();
        loop {
            // Match an identifier
            let id = self.match_t(TokenType::Id);
            if id == None {
                emit_error("Expected an identifier".to_string(), "help: Insert an identifier after this token".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            // Match a colon after the identifier
            let colon = self.match_t(TokenType::Colon);
            if colon == None {
                emit_error("Expected a colon".to_string(), "help: Insert a colon after this identifier".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            // Match a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                emit_error("Expected a type".to_string(), "help: Insert a type after this colon".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            // Push the field onto the list of fields
            fields.push((id.unwrap(), typ.unwrap()));

            // Match a comma
            // If the comma isn't there, stop looking for more fields
            let comma = self.match_t(TokenType::Comma);
            if comma == None {
                break;
            }
        }
        
        // Match a left curly brace to follow the identifier
        let rb = self.match_t(TokenType::RightBrace);
        if rb == None {
            emit_error("Expected right curly brace".to_string(), "help: Insert a right curly brace after this struct field".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
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

        // Match an identifier
        let id = self.match_t(TokenType::Id);
        if id == None {
            self.pos = start;
            return Node::Non;
        }
        
        // Match a left parenthesis to follow the identifier
        let lp = self.match_t(TokenType::LeftParen);
        if lp == None {
            emit_error("Expected a left parenthesis".to_string(), "help: Insert a left parenthesis after this identifier".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }

        // Parse a series of arguments followed by commas. If the comma is
        // not found, stop parsing arguments
        let mut args: Vec<Box<Expression>> = Vec::new();
        loop {
            let expr = self.expression(self.pos);
            if expr == Expression::Non {
                emit_error("Expected an argument".to_string(), "help: Insert an expression after this token".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            args.push(Box::new(expr));
            let comma = self.match_t(TokenType::Comma);
            if comma == None {
                break;
            }
        }

        // Match a right parenthesis to follow the arguments
        let rp = self.match_t(TokenType::RightParen);
        if rp == None {
            emit_error("Expected a right parenthesis".to_string(), "help: Insert a right parenthesis after this token".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }
        
        // Match a semi-colon after the right parenthesis
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error("Expected semi-colon".to_string(), "help: Insert a semi-colon after this parenthesis".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }
        match id.clone().unwrap().as_str() {
            "write" => {},
            name => {self.symtable.find_error(name.to_string(), SymbolType::Func, None);}
        };
        return Node::FuncCall {id: id.unwrap(), args: args};
    }

    /// Parses a let statement
    /// # Example
    /// let a = 5;
    /// or
    /// let a: int = 5;
    fn let_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Expect the 'let' keyword
        let key = self.match_t(TokenType::Let);
        if key == None {
            self.pos = start;
            return Node::Non;
        }
        
        // Match an identifier after the 'let' keyword
        let id = self.match_t(TokenType::Id);
        if id == None {
            emit_error("Expected an identifier".to_string(), "help: Insert an identifier after this 'let' keyword".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }

        // Match an equals sign after the identifer
        let mut eq = self.match_t(TokenType::Equal);
        let mut check_type = false;
        let mut var_typ = String::new();
        if eq == None {
            // If we don't find an equals sign, look for a colon
            eq = self.match_t(TokenType::Colon);
            if eq == None {
                emit_error("Expected an equals sign or a colon".to_string(), "help: Insert an equals sign or a colon after this identifier".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }

            // Match a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                emit_error("Expected a type".to_string(), "help: Insert a type after this colon".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
            check_type = true;
            var_typ = typ.unwrap();

            // Once we find a type, look again for an equals sign
            eq = self.match_t(TokenType::Equal);
            if eq == None {
                emit_error("Expected an equals sign".to_string(), "help: Insert an equals sign after this type".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
            }
        }

        // Look for an expression for the value of the let statement
        let expr: Expression = self.expression(self.pos);
        if expr == Expression::Non {
            emit_error("Expected an expression".to_string(), "help: Take away the equals sign or insert an expression after this equals sign".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }
        
        // If the type of the expression has a type-checker error, print an error
        if expr.validate() == "error" {
            emit_error("This type does not match the type of the expression".to_string(), "".to_string(), &self.tokens[self.pos - 1], ErrorType::MismatchedTypes);
        }

        // If the type of the expression doesn't match the type given by programmer, print error
        if check_type == true && var_typ.clone() != expr.validate() {
            emit_error("The type of the variable and the type of the expression do not match".to_string(), format!("help: Change this to {}", expr.validate()), &self.tokens[start + 3], ErrorType::MismatchedTypes);
        }

        // Match a semi-colon after the expression
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error("Expected a semi-colon".to_string(), "help: Insert a semi-colon after this expression".to_string(), &self.tokens[self.pos - 1], ErrorType::ExpectedToken);
        }

        self.symtable.add_symbol(id.clone().unwrap(), expr.validate().to_string(), SymbolType::Var, format!("%.{}", self.id_c), Vec::new());
        self.id_c += 1;
        return Node::Let {id: id.unwrap(), expr: expr, gen_id: format!("%.{}", self.id_c - 1)};
    }

    /// Parses a series of statements based off of the input tokens
    fn program(&mut self, mut max_len: usize) {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
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
            emit_error("Unexpected token or EOF".to_string(), "help: Failed to parse this statement".to_string(), &self.tokens[self.pos], ErrorType::ExpectedToken);
            break;
        }
       
        let mut gen: Generator = Generator::construct();
        gen.generate(nodes);

        // Open an output file and write to it
        let mut out_file = File::create("a.ll").expect("Couldn't create the output file");
        out_file.write_all(gen.ir_b.code.as_bytes()).expect("Couldn't write to the output file");
        Command::new("lli a.ll");
    }

    /// Resets the position and calls "program()"
    pub fn parse(&mut self) {
        self.pos = 0;
        self.program(0);
    }
}
