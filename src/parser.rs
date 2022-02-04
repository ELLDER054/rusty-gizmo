pub mod lexer;
mod ast;
pub mod generator;
pub mod symbol;

use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::ErrorType;
use self::lexer::error::emit_error;
use self::ast::Node;
use self::ast::Expression;
use self::symbol::SymbolController;
use self::symbol::SymbolType;
use self::symbol::VarSymbol;

/// Stores information for a "Parser"
pub struct Parser {
    /// Current position in tokens
    pub pos: usize,

    /// Input list of tokens
    pub tokens: Vec<Token>,

    /// Initialize a symbol table
    pub symtable: SymbolController,

    /// Stores the number of identifiers created
    pub id_c: usize,

    /// Whether or not the parser is currently in a function
    pub in_function: bool,

    /// The parser's current function type
    pub function_typ: String,

    /// Whether or not the parser is currently in an if
    pub in_if: bool,

    /// The parser's current if begin
    pub if_begin: String,

    /// The parser's current if else
    pub if_else: String,

    /// The parser's current if end
    pub if_end: String,

    /// Whether or not the parser is currently in a loop
    pub in_loop: bool,

    /// The parser's current loop begin
    pub loop_begin: String,

    /// The parser's current loop end
    pub loop_end: String,

    /// The number of labels created
    pub label_num: usize
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
        return self.boolean(start);
    }

    /// Parse a boolean operation
    /// # Example
    /// `a and b` or `a or b`
    fn boolean(&mut self, start: usize) -> Expression {
        self.pos = start;
        // Parse the left hand side of the expression
        let mut expr = self.equality(self.pos);

        // Continue matching an operator with another side of the expression
        // after it
        while self.match_t(TokenType::And) != None
        || self.match_t(TokenType::Or) != None {
            let save = self.pos - 1;
            let right = self.equality(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    "help: Insert an expression after this operator",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {
                oper:  self.tokens[save].value.clone(),
                left:  Box::new(expr.clone()),
                right: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes);
            }
        }

        return expr;
    }

    /// Parse an equality operation
    /// # Example
    /// `a == b` or `a != b`
    fn equality(&mut self, start: usize) -> Expression {
        self.pos = start;
        // Parse the left hand side of the expression
        let mut expr = self.comparison(self.pos);

        // Continue matching an operator with another side of the expression
        // after it
        while self.match_t(TokenType::EqualEqual) != None
        || self.match_t(TokenType::NotEqual) != None {
            let save = self.pos - 1;
            let right = self.comparison(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    "help: Insert an expression after this operator",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {
                oper:  self.tokens[save].value.clone(),
                left:  Box::new(expr.clone()),
                right: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes);
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

        // Continue matching a (OPER EXPR)
        while self.match_t(TokenType::GreaterThan) != None
        || self.match_t(TokenType::LessThan) != None
        || self.match_t(TokenType::GreaterEqual) != None
        || self.match_t(TokenType::LessEqual) != None {
            let save = self.pos - 1;
            let right = self.term(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    "help: Insert an expression after this operator",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {
                oper: self.tokens[save].value.as_str().to_string(),
                left: Box::new(expr.clone()),
                right: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes
                );
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

        // Continue matching a (OPER EXPR)
        while self.match_t(TokenType::Plus) != None
        || self.match_t(TokenType::Dash) != None {
            let save = self.pos - 1;
            let right = self.factor(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    "help: Insert an expression after this operator",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let left = expr.clone();
            expr = Expression::BinaryOperator {
                oper: self.tokens[save].value.as_str().to_string(),
                left: Box::new(expr.clone()),
                right: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left.validate(), right.clone().validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes
                );
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

        // Continue matching a (OPER EXPR)
        while self.match_t(TokenType::Star) != None
        || self.match_t(TokenType::Slash) != None {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    "help: Insert an expression after this operator",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let left_type = expr.clone();
            expr = Expression::BinaryOperator {
                oper: self.tokens[save].value.as_str().to_string(),
                left: Box::new(expr.clone()),
                right: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the types '{}' and '{}'", (&self.tokens[save]).value, left_type.validate(), right.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes
                );
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
        if self.match_t(TokenType::Not) != None
        || self.match_t(TokenType::Dash) != None {
            let save = self.pos - 1;
            let right = self.unary(self.pos);

            // If the right side isn't found, print an error
            if right == Expression::Non {
                emit_error(
                    "Expected an expression after this operator".to_string(),
                    // TOO LONG
                    "help: Take away the operator or insert an expression after this operator",
                    &self.tokens[self.pos - 1], 
                    ErrorType::ExpectedToken
                );
            }
            let oper = Expression::UnaryOperator {
                oper: (&self.tokens[save]).value.clone(),
                child: Box::new(right.clone())
            };

            // If the expression is semantically incorrect, print an error
            if oper.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to use '{}' with the type '{}'", (&self.tokens[save]).value, right.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes
                );
            }
            return oper;
        }
        return self.indexed(start);
    }

    /// Parse an indexed expression
    /// # Example
    /// `a.b` or `a[3]`
    fn indexed(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Parse the left hand side of the expression
        let mut expr = self.primary(self.pos);

        // Continue matching a (OPER EXPR)
        while self.match_t(TokenType::Dot) != None
        || self.match_t(TokenType::LeftBracket) != None {
            let save = self.pos - 1;

            // If the right isn't found, print an error
            let left_type = expr.clone();
            if (&self.tokens[save]).typ == TokenType::Dot {
                // If the right side isn't found, print an error
                let right = self.match_t(TokenType::Id);
                if right == None {
                    emit_error(
                        // TOO LONG
                        "Expected an expression after this operator".to_string(),
                        "help: Insert an expression after this operator",
                        &self.tokens[self.pos - 1],
                        ErrorType::ExpectedToken
                    );
                }

                // TOO LONG
                let sym = self.symtable.find_global_struct_error(expr.clone().validate().to_string());
                let mut field_num = 0;
                for field in sym.arg_types.iter() {
                    if field.clone() == right.clone().unwrap() {
                        expr = Expression::StructDot {
                            id: Box::new(expr.clone()),
                            id2: right.clone().unwrap(),
                            typ: field.clone(),
                            field_num: field_num
                        };
                        break;
                    }
                    field_num += 1;
                }
            } else {
                // If the right side isn't found, print an error
                let right = self.expression(self.pos);
                if right == Expression::Non {
                    emit_error(
                        // TOO LONG
                        "Expected an expression after this operator".to_string(),
                        "help: Insert an expression after this operator",
                        &self.tokens[self.pos - 1],
                        ErrorType::ExpectedToken
                    );
                }
                
                // If the index is not an integer, print an error
                if (&right).validate() != "int" {
                    emit_error(
                        "Mismatched types".to_string(),
                        "help: This must be an integer",
                        &self.tokens[save],
                        ErrorType::MismatchedTypes
                    );
                }
                self.match_t(TokenType::RightBracket);
                expr = Expression::IndexedValue {
                    src: Box::new(expr.clone()),
                    index: Box::new(right.clone()),
                    new_typ: self.strip_arr(expr.validate().to_string())
                };
            }
            // If the expression is semantically incorrect, print an error
            if expr.validate() == "error" {
                emit_error(
                    "Mismatched types within this expression".to_string(),
                    // TOO LONG
                    format!("Attempted to index with the type '{}'", left_type.validate()).as_str(),
                    &self.tokens[save],
                    ErrorType::MismatchedTypes
                );
            }
        }

        return expr;
    }

    /// Strips an array type down to the type of it's elements
    fn strip_arr(&self, s: String) -> String {
        match s.as_str() {
            "string" => "char",

            // Strips the last two characters of a type (i.e., int[] to int)
            _ => &s[0..s.len() - 2]
        }.to_string()
    }

    /// Parse a constant value
    fn primary(&mut self, start: usize) -> Expression {
        self.pos = start;

        // Match an expression constant
        let int = self.match_t(TokenType::Int);
        if int != None {
            return Expression::Int(int.unwrap().parse().unwrap());
        }
        let chr = self.match_t(TokenType::Char);
        if chr != None {
            return Expression::Chr(chr.unwrap().chars().collect::<Vec<char>>()[0]);
        }
        let dec = self.match_t(TokenType::Dec);
        if dec != None {
            return Expression::Dec(dec.unwrap().parse().unwrap());
        }
        let string = self.match_t(TokenType::Str);
        if string != None {
            return Expression::Str(string.unwrap().as_str().to_string());
        }
        let boolean = self.match_t(TokenType::Bool);
        if boolean != None {
            return Expression::Bool(if boolean.unwrap().as_str() == "true" {true} else {false});
        }
        let fc = self.func_call_no_semi(self.pos);
        if fc != Node::Non {
            if let Node::FuncCall {id, args} = fc {
                let sym = self.symtable.find_global_func_error(id.clone());
                if sym.typ == "void" {
                    emit_error(
                        "Mismatched types".to_string(),
                        "Cannot use a function that returns 'void' as an expression",
                        &self.tokens[start],
                        ErrorType::MismatchedTypes
                    );
                };
                return Expression::FuncCall {
                    id: id,
                    args: args,
                    typ: sym.typ
                };
            }
        }
        let lb = self.match_t(TokenType::LeftBracket);
        if lb != None {
            // Save the position of the beginning of the array
            let save = self.pos - 1;

            // Create a vector to store the elements
            let mut values: Vec<Expression> = Vec::new();
            loop {
                // Match an expression
                let array_value = self.expression(self.pos);
                if array_value == Expression::Non {
                    break;
                }

                // Append the element to the 'values' list
                values.push(array_value);

                // Match a comma
                let comma = self.match_t(TokenType::Comma);
                if comma == None {
                    break;
                }
            }

            // If the array is empty, it is impossible to infer it's type
            // TODO: Allow explicitly stating the array's type
            if values.len() == 0 {
                emit_error(
                    "This array does not have a type".to_string(),
                    "help: Consider explicitly stating the array's type",
                    &self.tokens[save],
                    ErrorType::UndefinedArray
                );
            }

            // Match a right bracket
            let rb = self.match_t(TokenType::RightBracket);
            if rb != None {
                // Store the type of the first element.
                let first_typ = (&values[0]).validate();

                // Loop through the rest of the elements
                for value in values[1..values.len()].iter() {
                    // If the type of the element does not match the first
                    // type, print an error
                    if value.validate() != first_typ {
                        emit_error(
                            "This array has mismatched types".to_string(),
                            "",
                            &self.tokens[save],
                            ErrorType::MismatchedTypes
                        );
                    }
                }
                let typ = format!("{}[]", &first_typ);
                return Expression::Array {values: values.clone(), typ: typ};
            }
            return Expression::Non;
        }
        let ns = self.new_struct(self.pos);
        if ns != Expression::Non {
            return ns;
        }
        let id = self.match_t(TokenType::Id);
        if id != None {
            let sym = self.symtable.find_global_var_error(id.clone().unwrap());
            // TOO LONG
            return Expression::Id(id.unwrap(), sym.typ.clone(), sym.gen_id.clone());
        }
        let lp = self.match_t(TokenType::LeftParen);
        if lp != None {
            let e = self.expression(self.pos);
            if e == Expression::Non {
                emit_error(
                    "Expected an expression".to_string(),
                    "Expected an expression after this left parenthesis",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            let rp = self.match_t(TokenType::RightParen);
            if rp == None {
                emit_error(
                    "Expected a right parenthesis".to_string(),
                    "Expected a right parenthesis after this expression",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            return e;
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
                emit_error(
                    "Expected a left parenthesis".to_string(),
                    "help: Insert a left parenthesis after this identifier",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }

            // Create a vector to store the fields
            let mut fields: Vec<Expression> = Vec::new();

            // Expect multiple fields followed by commas
            let mut field_num = 0;
            loop {
                // Match an expression
                let expr = self.expression(self.pos);
                if expr == Expression::Non {
                    break;
                }

                // Push the expression to fields
                fields.push(expr.clone());

                // Find the symbol in the symbol table
                // TOO LONG
                let sym = self.symtable.find_global_struct_error(id.clone().unwrap());
                
                if sym.arg_types[field_num] != expr.validate() {
                    // If the type of the expected field is not equal to the
                    // given field, print an error
                    emit_error(
                        // TOO LONG
                        "The type of this expression does not match the corresponding field".to_string(),
                        "",
                        &self.tokens[self.pos - 1],
                        ErrorType::MismatchedTypes
                    );
                }
                field_num += 1;

                // Match a comma
                // If the comma isn't there, stop looking for more fields
                let comma = self.match_t(TokenType::Comma);
                if comma == None {
                    break;
                }
            }
            
            // Match a right parenthesis
            let rp = self.match_t(TokenType::RightParen);
            if rp == None {
                emit_error(
                    "Expected a right parenthesis".to_string(),
                    "help: Insert a right parenthesis after this token",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            
            // Return the struct initialization
            return Expression::NewStruct {id: id.unwrap(), fields: fields};
        }
        return Expression::Non;
    }

    /// Parses a type
    /// # Example
    /// `struct Foo` or `int`
    fn rec_type(&mut self, start: usize) -> Option<String> {
        self.pos = start;

        // Match a basic type
        let typ = self.match_t(TokenType::Type);
        if typ != None {
            // Allow multiple pairs of brackets after that
            let mut brackets = String::new();

            // `int[]`, `int[][]` are valid types
            while self.match_t(TokenType::LeftBracket) != None {
                let rb = self.match_t(TokenType::RightBracket);
                if rb == None {
                    emit_error(
                        "Expected a right bracket".to_string(),
                        "help: Insert a right bracket after this left bracket",
                        &self.tokens[self.pos - 1],
                        ErrorType::ExpectedToken
                    );
                }
                brackets.push_str("[]");
            }
            return Some(format!("{}{}", typ.unwrap(), brackets));
        }

        // Match an identifier
        // The identifier must also be a struct
        let typ = self.match_t(TokenType::Id);
        if typ == None
        || self.symtable.find_global_struct(typ.clone().unwrap()) == None {
            // If there was not an identifier or the identifier was not a
            // struct, print an error
            emit_error(
                "Expected a struct type".to_string(),
                "help: Insert a struct type after this token",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }
        let mut brackets = String::new();

        // Again, allow multiple pairs of brackets after that
        while self.match_t(TokenType::LeftBracket) != None {
            let rb = self.match_t(TokenType::RightBracket);
            if rb == None {
                emit_error(
                    "Expected a right bracket".to_string(),
                    "help: Insert a right bracket after this left bracket",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
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
            emit_error(
                "Expected an identifier".to_string(),
                "help: Insert an identifier after the 'struct' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }
        
        // Match a left curly brace to follow the identifier
        let lb = self.match_t(TokenType::LeftBrace);
        if lb == None {
            emit_error(
                "Expected a left curly brace".to_string(),
                "help: Insert a left curly brace after this identifier",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Parse a series of struct fields followed by commas. If the comma is
        // not found, stop parsing fields
        let mut fields: Vec<(String, String)> = Vec::new();
        loop {
            // Match an identifier
            let id = self.match_t(TokenType::Id);
            if id == None {
                emit_error(
                    "Expected an identifier".to_string(),
                    "help: Insert an identifier after this token",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }

            // Match a colon after the identifier
            let colon = self.match_t(TokenType::Colon);
            if colon == None {
                emit_error(
                    "Expected a colon".to_string(),
                    "help: Insert a colon after this identifier",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }

            // Match a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                emit_error(
                    "Expected a type".to_string(),
                    "help: Insert a type after this colon",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
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
            emit_error(
                "Expected right curly brace".to_string(),
                "help: Insert a right curly brace after this struct field",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Add correct symbols
        let mut types: Vec<String> = Vec::new();
        for field in fields.iter() {
            types.push(field.clone().1);
        }
        // TOO LONG
        self.symtable.add_symbol(id.clone().unwrap(), id.clone().unwrap(), SymbolType::Struct, format!("%.{}", self.id_c), Some(types));
        
        // Return the struct node
        return Node::Struct {id: id.unwrap(), fields: fields};
    }

    /// Parse a function declaration
    /// # Example
    /// func a(args) : type {
    ///     // statements
    /// }
    fn func_declaration(&mut self, start: usize) -> Node {
        self.pos = start;

        // Match the 'func' keyword
        let key = self.match_t(TokenType::Func);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        // Match an identifier
        let id = self.match_t(TokenType::Id);
        if id == None {
            emit_error(
                "Expected an identifier".to_string(),
                "help: Insert an identifier after this 'func' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Match a left parenthesis
        let lp = self.match_t(TokenType::LeftParen);
        if lp == None {
            emit_error(
                "Expected a left parenthesis".to_string(),
                "help: Insert a left parenthesis after this identifier",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        let mut args: Vec<(String, String)> = Vec::new();
        let mut arg_types: Vec<String>      = Vec::new();
        let mut arg_symbols: Vec<VarSymbol>    = Vec::new();
        loop {
            let typ = self.rec_type(self.pos);
            if typ == None {
                break;
            }
            let id = self.match_t(TokenType::Id);
            if id == None {
                break;
            }
            args.push((typ.clone().unwrap(), format!("%.{}", self.id_c)));
            arg_types.push(typ.clone().unwrap());
            arg_symbols.push(VarSymbol {id: id.unwrap(), typ: typ.unwrap(), gen_id: format!("%.{}", self.id_c)});
            self.id_c += 1;
            let comma = self.match_t(TokenType::Comma);
            if comma == None {
                break;
            }
        }

        // Match a right parenthesis
        let rp = self.match_t(TokenType::RightParen);
        if rp == None {
            emit_error(
                "Expected a right parenthesis".to_string(),
                "help: Insert a right parenthesis after this token",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Match a colon
        let mut typ: Option<String> = Some(String::from("void"));
        if self.match_t(TokenType::Colon) != None {
            typ = self.rec_type(self.pos);
            if typ == None {
                emit_error(
                    "Expected a type".to_string(),
                    "help: Insert a type after this colon",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
        }

        if self.in_function {
            emit_error(
                "Attempted to nest functions".to_string(),
                "",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        self.in_function = true;
        self.function_typ = typ.clone().unwrap();
        let body = self.block_statement(self.pos, arg_symbols);
        self.in_function = false;

        self.symtable.add_symbol(id.clone().unwrap(), typ.clone().unwrap(), SymbolType::Func, id.clone().unwrap(), Some(arg_types));
        return Node::FuncDecl {id: id.unwrap(), typ: typ.unwrap(), args: args, body: Box::new(body)};
    }

    /// Parses a function call with no semi-colon
    /// # Example
    /// foo(5)
    fn func_call_no_semi(&mut self, start: usize) -> Node {
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
            self.pos = start;
            return Node::Non;
        }

        // Parse a series of arguments followed by commas. If the comma is
        // not found, stop parsing arguments
        let mut args: Vec<Box<Expression>> = Vec::new();
        loop {
            let expr = self.expression(self.pos);
            if expr == Expression::Non {
                break;
            }

            // Append the argument to the 'args' list
            args.push(Box::new(expr));

            let comma = self.match_t(TokenType::Comma);
            if comma == None {
                break;
            }
        }

        // Match a right parenthesis to follow the arguments
        let rp = self.match_t(TokenType::RightParen);
        if rp == None {
            emit_error(
                "Expected a right parenthesis".to_string(),
                "help: Insert a right parenthesis after this token",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }
        match id.clone().unwrap().as_str() {
            "write" => {},
            name => {self.symtable.find_global_func_error(name.to_string());}
        };
        return Node::FuncCall {id: id.unwrap(), args: args};
    }

    fn func_call(&mut self, start: usize) -> Node {
        self.pos = start;

        let fc = self.func_call_no_semi(self.pos);
        if fc == Node::Non {
            self.pos = start;
            return Node::Non;
        }

        // Match a semi-colon after the right parenthesis
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error(
                "Expected semi-colon".to_string(),
                "help: Insert a semi-colon after this parenthesis",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        return fc;
    }

    /// Parse an assignment
    /// # Example
    /// a = 5;
    fn assign_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Match an identifier after the 'let' keyword
        let id = self.indexed(self.pos);
        if id == Expression::Non {
            self.pos = start;
            return Node::Non;
        }

        // Match an equals sign after the
        let eq = self.match_t(TokenType::Equal);
        if eq == None {
            emit_error(
                "Expected an equals sign".to_string(),
                "help: Insert an equals sign after this identifier",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Match an expression for the value of the let statement
        let expr: Expression = self.expression(self.pos);
        if expr == Expression::Non {
            emit_error(
                "Expected an expression".to_string(),
                "help: Take away the equals sign or insert an expression after this equals sign",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }
        
        // If the type of the expression has a type-checker error, print an error
        if expr.clone().validate() == "error" {
            emit_error(
                "This type does not match the type of the expression".to_string(),
                "",
                &self.tokens[self.pos - 1],
                ErrorType::MismatchedTypes
            );
        }

        // Match an expression for the value of the let statement
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error(
                "Expected a semi-colon".to_string(),
                "help: Insert a semi-colon after this expression",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        return Node::Assign {id: id, expr: expr};
    }
    
    /// Parses a block
    /// # Example
    /// {
    ///     write(3);
    /// }
    fn block_statement(&mut self, start: usize, arguments: Vec<VarSymbol>) -> Node {
        self.pos = start;

        // Match a left brace
        let lb = self.match_t(TokenType::LeftBrace);
        if lb == None {
            self.pos = start;
            return Node::Non;
        }
        self.symtable.add_scope();
        for symbol in arguments.iter() {
            self.symtable.add_symbol(symbol.clone().id, symbol.clone().typ, SymbolType::Var, symbol.clone().gen_id, None);
        }
        let mut statements: Vec<Box<Node>> = Vec::new();
        while (&self.tokens[self.pos]).typ != TokenType::RightBrace {
            statements.push(Box::new(self.statement(self.pos)));
        }
        self.symtable.pop_scope();
        // Match a right brace
        let rb = self.match_t(TokenType::RightBrace);
        if rb == None {
            self.pos = start;
            return Node::Non;
        }

        return Node::Block {statements: statements};
    }
    
    /// Parses an if-statement
    /// # Example
    /// if "abc" == "abc" {
    ///     write("Yay!\n");
    /// }
    fn if_stmt(&mut self, start: usize) -> Node {
        self.pos = start;

        let key = self.match_t(TokenType::If);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        let cond = self.expression(self.pos);
        if cond == Expression::Non {
            emit_error(
                "Expected a conditional expression".to_string(),
                "help: Insert an conditional expression after this 'if' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        
        }
        self.in_if = true;
        let body = self.statement(self.pos);
        self.in_if = false;

        let mut else_body: Option<Box<Node>> = None;
        let e = self.match_t(TokenType::Else);
        if e != None {
            else_body = Some(Box::new(self.statement(self.pos)));
        }

        let used = if else_body == None {2} else {3};
        self.label_num += used;

        return Node::If {cond: cond, body: Box::new(body), else_body: else_body, begin: (self.label_num - used) as i32, else_: (self.label_num - (used - 1)) as i32, end: (self.label_num - 1) as i32};
    }

    /// Parses a while-loop
    /// # Example
    /// let i = 0;
    /// while i < 10 {
    ///     write(i);
    ///     i = i + 1;
    /// }
    fn while_loop(&mut self, start: usize) -> Node {
        self.pos = start;

        let key = self.match_t(TokenType::While);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        let cond = self.expression(self.pos);
        if cond == Expression::Non {
            emit_error(
                "Expected a conditional expression".to_string(),
                "help: Insert an conditional expression after this 'while' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        let save_in_loop = self.in_loop;
        self.in_loop = true;
        let save_b = self.loop_begin.clone();
        let save_e = self.loop_end.clone();
        self.loop_begin = format!("{}", self.label_num);
        self.loop_end = format!("{}", self.label_num + 1);
        let body = self.statement(self.pos);
        self.loop_begin = save_b;
        self.loop_end = save_e;
        self.in_loop = save_in_loop;
        self.label_num += 2;
        return Node::While {cond: cond, body: Box::new(body), begin: (self.label_num - 2) as i32, end: (self.label_num - 1) as i32};
    }

    /// Parses a break or continue statement
    fn ret_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Match the 'ret' keyword
        let mut key = self.match_t(TokenType::Break);
        if key == None {
            key = self.match_t(TokenType::Continue);
            if key == None {
                self.pos = start;
                return Node::Non;
            }
        }

        // Match a semi-colon after the expression
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error(
                "Expected a semi-colon".to_string(),
                "help: Insert a semi-colon after this expression",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        if !self.in_loop {
            emit_error(
                "Attempted to break outside of a loop".to_string(),
                "",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        return Node::Pause {label: if key.unwrap() == "break" {self.loop_end.clone()} else {self.loop_begin.clone()}};
    }

    /// Parses a pause statement
    /// # Example
    /// break 5;
    fn pause_statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Match the 'ret' keyword
        let key = self.match_t(TokenType::Ret);
        if key == None {
            self.pos = start;
            return Node::Non;
        }

        // Match an expression
        let expr = self.expression(self.pos);
        if expr == Expression::Non {
            emit_error(
                "Expected an expression".to_string(),
                "help: Insert an expression after this 'ret' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Match a semi-colon after the expression
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error(
                "Expected a semi-colon".to_string(),
                "help: Insert a semi-colon after this expression",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        if !self.in_function {
            emit_error(
                "Attempted to return from the main scope".to_string(),
                "",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        if self.function_typ.clone() != expr.clone().validate() {
            emit_error(
                "Attempted to return with the incorrect type".to_string(),
                format!("help: Expected {}, found {}", self.function_typ, expr.clone().validate()).as_str(),
                &self.tokens[self.pos - 1],
                ErrorType::MismatchedTypes
            );
        }

        return Node::Ret {expr: expr};
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
            emit_error(
                "Expected an identifier".to_string(),
                "help: Insert an identifier after this 'let' keyword",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        // Match an equals sign after the identifer
        let mut eq = self.match_t(TokenType::Equal);
        let mut check_type = false;
        let mut var_typ = String::new();
        if eq == None {
            // If we don't find an equals sign, look for a colon
            eq = self.match_t(TokenType::Colon);
            if eq == None {
                emit_error(
                    "Expected an equals sign or a colon".to_string(),
                    "help: Insert an equals sign or a colon after this identifier",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }

            // Match a type after the colon
            let typ = self.rec_type(self.pos);
            if typ == None {
                emit_error(
                    "Expected a type".to_string(),
                    "help: Insert a type after this colon",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
            check_type = true;
            var_typ = typ.unwrap();

            // Once we find a type, look again for an equals sign
            eq = self.match_t(TokenType::Equal);
            if eq == None {
                emit_error(
                    "Expected an equals sign".to_string(),
                    "help: Insert an equals sign after this type",
                    &self.tokens[self.pos - 1],
                    ErrorType::ExpectedToken
                );
            }
        }

        // Match an expression for the value of the let statement
        let expr: Expression = self.expression(self.pos);
        if expr == Expression::Non {
            emit_error(
                "Expected an expression".to_string(),
                "help: Insert an expression after this equals sign",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }
        
        // If the type of the expression is semantically incorrect, print an
        // error
        if expr.validate() == "error" {
            emit_error(
                "This type does not match the type of the expression".to_string(),
                "",
                &self.tokens[self.pos - 1],
                ErrorType::MismatchedTypes
            );
        }

        // If the type of the expression doesn't match the type given by
        // programmer, print error
        if check_type == true && var_typ.clone() != expr.validate() {
            emit_error(
                "The type of the variable and the type of the expression do not match".to_string(),
                format!("help: Change this to {}", expr.validate()).as_str(),
                &self.tokens[start + 3],
                ErrorType::MismatchedTypes
            );
        }

        // Match a semi-colon after the expression
        let semi = self.match_t(TokenType::SemiColon);
        if semi == None {
            emit_error(
                "Expected a semi-colon".to_string(),
                "help: Insert a semi-colon after this expression",
                &self.tokens[self.pos - 1],
                ErrorType::ExpectedToken
            );
        }

        self.symtable.add_symbol(id.clone().unwrap(), expr.validate().to_string(), SymbolType::Var, format!("%.{}", self.id_c), None);
        self.id_c += 1;
        return Node::Let {id: id.unwrap(), expr: expr, gen_id: format!("%.{}", self.id_c - 1)};
    }

    /// Parses a statement
    fn statement(&mut self, start: usize) -> Node {
        self.pos = start;

        // Check for a let statement
        let let_stmt = self.let_statement(self.pos);
        if let_stmt != Node::Non {
            return let_stmt;
        }

        // Check for a while-loop
        let wh = self.while_loop(self.pos);
        if wh != Node::Non {
            return wh;
        }

        // Check for a while-loop
        let i = self.if_stmt(self.pos);
        if i != Node::Non {
            return i;
        }

        // Check for a function call
        let func_call = self.func_call(self.pos);
        if func_call != Node::Non {
            return func_call;
        }

        // Check for a return statement
        let ret = self.ret_statement(self.pos);
        if ret != Node::Non {
            return ret;
        }

        // Check for a pause statement
        let pause = self.pause_statement(self.pos);
        if pause != Node::Non {
            return pause;
        }

        // Check for a function declaration
        let func_decl = self.func_declaration(self.pos);
        if func_decl != Node::Non {
            return func_decl;
        }

        // Check for an assignment statement
        let assign = self.assign_statement(self.pos);
        if assign != Node::Non {
            return assign;
        }

        // Check for a struct definition
        let struct_def = self.struct_def(self.pos);
        if struct_def != Node::Non {
            return struct_def;
        }

        // Check for a struct definition
        let block = self.block_statement(self.pos, vec![]);
        if block != Node::Non {
            return block;
        }

        return Node::Non;
    }

    /// Parses a series of statements based off of the input tokens
    fn program(&mut self, mut max_len: usize) -> Vec<Box<Node>> {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
        // Stores each statement's node
        let mut nodes: Vec<Box<Node>> = Vec::new();

        // Loop through the tokens
        while self.pos < max_len {
            // Check for a statement
            let stmt = self.statement(self.pos);
            if stmt != Node::Non {
                nodes.push(Box::new(stmt));
                continue;
            }
            emit_error(
                "Unexpected token or EOF".to_string(),
                "help: Failed to parse this statement",
                &self.tokens[self.pos],
                ErrorType::ExpectedToken
            );
            break;
        }
        nodes
    }

    /// Resets the position and calls "program()"
    pub fn parse(&mut self) -> Vec<Box<Node>> {
        self.pos = 0;
        self.symtable.add_symbol("write".to_string(), "void".to_string(), SymbolType::Func, "@printf".to_string(), None);
        self.symtable.add_symbol("len".to_string(), "int".to_string(), SymbolType::Func, "@strlen".to_string(), None);
        self.program(0)
    }
}
