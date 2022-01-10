pub mod lexer;
mod ast;

use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::Error;
use self::lexer::error::ErrorType;
use self::ast::Let;
use self::ast::Int;
use self::ast::Bool;
use self::ast::Str;
use self::ast::Dec;
use self::ast::Operator;
use self::ast::UnaryOperator;
use self::ast::Node;
use self::ast::Nodes;
use self::ast::Validated;

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

    fn expression(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        return self.equality(start);
    }

    fn equality(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        let mut expr = self.comparison(self.pos);

        while self.expect_type(TokenType::EqualEqual) || self.expect_type(TokenType::NotEqual) {
            let save = self.pos - 1;
            let right = self.comparison(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[save].value.as_str().to_string(), right: right, none: false});
        }

        return expr;
    }

    fn comparison(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        let mut expr = self.term(self.pos);

        while self.expect_type(TokenType::GreaterThan) || self.expect_type(TokenType::LessThan) || self.expect_type(TokenType::GreaterEqual) || self.expect_type(TokenType::LessEqual) {
            let save = self.pos - 1;
            let right = self.term(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[save].value.as_str().to_string(), right: right, none: false});
        }

        return expr;
    }

    fn term(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        let mut expr = self.factor(self.pos);

        while self.expect_type(TokenType::Plus) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.factor(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[save].value.as_str().to_string(), right: right, none:false});
        }

        return expr;
    }

    fn factor(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        let mut expr = self.unary(self.pos);

        while self.expect_type(TokenType::Star) || self.expect_type(TokenType::Slash) {
            let save = self.pos - 1;
            let right = self.unary(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[save].value.as_str().to_string(), right: right, none: false});
        }

        return expr;
    }

    fn unary(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        if self.expect_type(TokenType::Not) || self.expect_type(TokenType::Dash) {
            let save = self.pos - 1;
            let right = self.primary(self.pos);
            return Box::new(UnaryOperator {oper: self.tokens[save].value.as_str().to_string(), right: right, none: false});
        }
        return self.primary(start);
    }

    fn primary(&mut self, start: usize) -> Box<dyn Validated> {
        self.pos = start;
        let int: bool = self.expect_type(TokenType::Int);
        if int {
            return Box::new(Int {value: self.tokens[self.pos - 1].value.parse().unwrap(), none: false});
        }
        let dec: bool = self.expect_type(TokenType::Dec);
        if dec {
            return Box::new(Dec {value: self.tokens[self.pos - 1].value.parse::<f64>().unwrap(), none: false});
        }
        let string: bool = self.expect_type(TokenType::Str);
        if string {
            return Box::new(Str {value: self.tokens[self.pos - 1].value.as_str().to_string(), none: false});
        }
        let boolean: bool = self.expect_type(TokenType::Bool);
        if boolean {
            return Box::new(Bool {value: if self.tokens[self.pos - 1].value == "true" {true} else {false}, none: false});
        }
        return Box::new(Int {value: 0, none: true});
    }

    fn let_statement(&mut self, start: usize) -> Let {
        self.pos = start;
        let key: bool = self.expect_type(TokenType::Let);
        if !key {
            self.pos = start;
            return Let {id: "".to_string(), expr: Box::new(Int {value: 0, none: true}), none: true};
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
        let expr: Box<dyn Validated> = self.expression(self.pos);
        if (*expr).get_type() == "non" {
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
        return Let {id: (&self.tokens[save].value).to_string(), expr: expr, none: false};
    }

    fn program(&mut self, mut max_len: usize) {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
        let mut stmts: Vec<Nodes> = Vec::new();
        while self.pos < max_len {
            let let_stmt = self.let_statement(self.pos);
            if !let_stmt.is_none() {
                println!("Success");
                stmts.push(Nodes::Let(let_stmt));
                continue;
            }
            println!("Failure");
        }
        for stmt in stmts.iter() {
            let Nodes::Let(l) = stmt;
            println!("{}, {}", l.id, l.expr.validate());
        }
    }

    pub fn parse(&mut self) {
        self.program(0);
        self.pos = 0;
    }
}
