pub mod lexer;
mod ast;
mod generator;

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

pub struct Parser {
    pub pos: usize,
    pub tokens: Vec<Token>,
}

impl Parser {
    fn expect_type(&mut self, t: TokenType) -> bool {
        if self.pos >= self.tokens.len() {
            return false
        }
        if self.tokens[self.pos].typ == t {
            self.pos += 1;
            return true;
        }
        return false;
    }

    fn expression(&mut self, start: usize) -> Expression {
        self.pos = start;
        return self.equality(start);
    }

    fn equality(&mut self, start: usize) -> Expression {
        self.pos = start;
        let mut expr = self.comparison(self.pos);

        while self.expect_type(TokenType::EqualEqual) || self.expect_type(TokenType::NotEqual) {
            let save = self.pos - 1;
            let right = self.comparison(self.pos);
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
        let mut expr = self.term(self.pos);

        while self.expect_type(TokenType::GreaterThan) || self.expect_type(TokenType::LessThan) || self.expect_type(TokenType::GreaterEqual) || self.expect_type(TokenType::LessEqual) {
            let save = self.pos - 1;
            let right = self.term(self.pos);
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
        let mut expr = self.factor(self.pos);

        while self.expect_type(TokenType::Plus) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.factor(self.pos);
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
        let mut expr = self.unary(self.pos);

        while self.expect_type(TokenType::Star) || self.expect_type(TokenType::Slash) {
            let save = self.pos - 1;
            let right = self.unary(self.pos);
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
        if self.expect_type(TokenType::Not) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.unary(self.pos);
            return Expression::UnaryOperator {oper: self.tokens[save].value.as_str().to_string(), child: Box::new(right)};
        }
        return self.primary(start);
    }

    fn primary(&mut self, start: usize) -> Expression {
        self.pos = start;
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
        return Expression::Non;
    }

    fn func_call(&mut self, start: usize) -> Node {
        self.pos = start;
        let id: bool = self.expect_type(TokenType::Id);
        let save = self.pos - 1;
        if !id {
            self.pos = start;
            return Node::Non;
        }
        let lp: bool = self.expect_type(TokenType::LeftParen);
        if !lp {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected left parenthesis after this identifier", helpers: "help: Insert a left parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let mut args: Vec<Box<Expression>> = Vec::new();
        loop {
            let expr = self.expression(self.pos);
            if expr == Expression::Non {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected argument", helpers: "help: Insert an expression".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            args.push(Box::new(expr));
            let comma: bool = self.expect_type(TokenType::Comma);
            if !comma {
                break;
            }
        }
        let rp: bool = self.expect_type(TokenType::RightParen);
        if !rp {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected right parenthesis after this argument", helpers: "help: Insert a right parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let semi: bool = self.expect_type(TokenType::SemiColon);
        if  !semi {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this parenthesis", helpers: "help: Insert a semi-colon after this parenthesis".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        return Node::FuncCall {id: (&self.tokens[save].value).to_string(), args: args};
    }

    fn let_statement(&mut self, start: usize) -> Node {
        self.pos = start;
        let key: bool = self.expect_type(TokenType::Let);
        if !key {
            self.pos = start;
            return Node::Non;
        }
        let id: bool = self.expect_type(TokenType::Id);
        if !id {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected identifier after this let", helpers: "help: Insert an identifier".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let save: usize = self.pos - 1;
        let mut eq: bool = self.expect_type(TokenType::Equal);
        let mut save_type_pos = 0;
        if !eq {
            eq = self.expect_type(TokenType::Colon);
            if !eq {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign or colon after this identifier", helpers: "help: Insert an equals sign or a colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            let typ: bool = self.expect_type(TokenType::Type);
            if !typ {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected type after this colon", helpers: "help: Insert a type after this colon".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            save_type_pos = self.pos - 1;
            eq = self.expect_type(TokenType::Equal);
            if !eq {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign after this colon", helpers: "help: Insert an equals sign here".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
        }
        let expr: Expression = self.expression(self.pos);
        if expr == Expression::Non {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this equals sign", helpers: "help: Take away the equals sign or insert an expression after this equals sign".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        if expr.validate() == "error" {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "This type does not match the type of the expression", helpers: "".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        if save_type_pos != 0 && self.tokens[save_type_pos].value != expr.validate() {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "The type of the variable and the type of the expression do not match", helpers: format!("help: Change this to {}", expr.validate())};
            err.emit_error(&self.tokens[save_type_pos]);
            std::process::exit(1);
        }
        let semi: bool = self.expect_type(TokenType::SemiColon);
        if  !semi {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this expression", helpers: "help: Insert a semi-colon after this expression".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        return Node::Let {id: (&self.tokens[save].value).to_string(), expr: Box::new(expr)};
    }

    fn program(&mut self, mut max_len: usize) {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
        let mut gen: Generator = Generator {name_num: 0, code: "define i32 @main() {\nentry:\n".to_string(), ends: "".to_string(), begins: "".to_string()};
        let mut nodes: Vec<Node> = Vec::new();
        while self.pos < max_len {
            let let_stmt = self.let_statement(self.pos);
            if let_stmt != Node::Non {
                if let Node::Let {id, expr} = let_stmt {
                    nodes.push(Node::Let {id, expr});
                    continue;
                }
            }
            let func_call = self.func_call(self.pos);
            if func_call != Node::Non {
                if let Node::FuncCall {id, args} = func_call {
                    nodes.push(Node::FuncCall {id, args});
                    continue;
                }
            }
            println!("Failure");
            break;
        }
        gen.gen_all(nodes);
        gen.code.push_str("\tret i32 0\n}\n");
        gen.code = format!("{}{}{}", gen.begins, gen.code, gen.ends);
        println!("{}", gen.code);
        let mut out_file = File::create("a.ll").expect("Couldn't create the output file");
        out_file.write_all(gen.code.as_bytes()).expect("Couldn't write to the output file");
        Command::new("lli a.ll");
    }

    pub fn parse(&mut self) {
        self.pos = 0;
        self.program(0);
    }
}
