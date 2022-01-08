pub mod lexer;

use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::Error;
use self::lexer::error::ErrorType;

trait Node {
    fn get_type(&self) -> &str;
}

struct Let {
    id: String,
    expr: Box<dyn Node>,
}

impl Node for Let {
    fn get_type(&self) -> &str {
        return "let";
    }
}

struct Int {
    value: i32,
}

impl Node for Int {
    fn get_type(&self) -> &str {
        return "int";
    }
}

struct Operator {
    left: Box<dyn Node>,
    oper: String,
    right: Box<dyn Node>,
}

impl Node for Operator {
    fn get_type(&self) -> &str {
        return "oper";
    }
}

struct Dec {
    value: f64,
}

impl Node for Dec {
    fn get_type(&self) -> &str {
        return "dec";
    }
}

struct Str {
    value: String,
}

impl Node for Str {
    fn get_type(&self) -> &str {
        return "str";
    }
}

struct Non {}

impl Node for Non {
    fn get_type(&self) -> &str {
        return "non";
    }
}

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

    fn expression(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        return self.equality(start);
    }

    fn equality(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let mut expr = self.comparison(self.pos);

        while self.expect_type(TokenType::EqualEqual) || self.expect_type(TokenType::NotEqual) {
            let right = self.comparison(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[self.pos - 1].value.as_str().to_string(), right: right});
        }

        return expr;
    }

    fn comparison(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let mut expr = self.term(self.pos);

        while self.expect_type(TokenType::GreaterThan) || self.expect_type(TokenType::LessThan) || self.expect_type(TokenType::GreaterEqual) || self.expect_type(TokenType::LessEqual) {
            let right = self.term(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[self.pos - 1].value.as_str().to_string(), right: right});
        }

        return expr;
    }

    fn term(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let mut expr = self.factor(self.pos);

        while self.expect_type(TokenType::Plus) || self.expect_type(TokenType::Dash) {
            let right = self.factor(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[self.pos - 1].value.as_str().to_string(), right: right});
        }

        return expr;
    }

    fn factor(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let mut expr = self.unary(self.pos);

        while self.expect_type(TokenType::Star) || self.expect_type(TokenType::Slash) {
            let right = self.unary(self.pos);
            if (*right).get_type() == "non" {
                let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this operator", helpers: "help: Take away the operator or insert an expression after this operator".to_string()};
                err.emit_error(&self.tokens[self.pos - 1]);
                std::process::exit(1);
            }
            expr = Box::new(Operator {left: expr, oper: self.tokens[self.pos - 1].value.as_str().to_string(), right: right});
        }

        return expr;
    }

    fn unary(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        return self.primary(start);
    }

    fn primary(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let int: bool = self.expect_type(TokenType::Int);
        if int {
            return Box::new(Int {value: self.tokens[self.pos - 1].value.parse().unwrap()});
        }
        let dec: bool = self.expect_type(TokenType::Dec);
        if dec {
            return Box::new(Dec {value: self.tokens[self.pos - 1].value.parse::<f64>().unwrap()});
        }
        let string: bool = self.expect_type(TokenType::Str);
        if string {
            return Box::new(Str {value: self.tokens[self.pos - 1].value.as_str().to_string()});
        }
        return Box::new(Non {});
    }

    fn let_statement(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let key: bool = self.expect_type(TokenType::Let);
        if !key {
            self.pos = start;
            return Box::new(Non {});
        }
        let id: bool = self.expect_type(TokenType::Id);
        if !id {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected identifier after this let", helpers: "help: Insert an identifier".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let save: usize = self.pos;
        let eq: bool = self.expect_type(TokenType::Equal);
        if !eq {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected equals sign after this identifier", helpers: "help: Insert an equals sign".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let expr: Box<dyn Node> = self.expression(self.pos);
        if (*expr).get_type() == "non" {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected expression after this equals sign", helpers: "help: Take away the equals sign or insert an expression after this equals sign".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        let semi: bool = self.expect_type(TokenType::SemiColon);
        if  !semi {
            let err: Error = Error {typ: ErrorType::ExpectedToken, msg: "Expected semi-colon after this expression", helpers: "help: Insert a semi-colon after this expression".to_string()};
            err.emit_error(&self.tokens[self.pos - 1]);
            std::process::exit(1);
        }
        return Box::new(Let {id: (&self.tokens[save - 1].value).to_string(), expr: expr});
    }

    fn statement(&mut self, start: usize) -> Box<dyn Node> {
        self.pos = start;
        let lets: Box<dyn Node> = self.let_statement(self.pos);
        if (*lets).get_type() != "non" {
            return lets;
        }
        return Box::new(Non {});
    }

    fn program(&mut self, mut max_len: usize) {
        if max_len == 0 {
            max_len = self.tokens.len();
        }
        while self.pos < max_len {
           let stmt: Box<dyn Node> = self.statement(self.pos);
           if (*stmt).get_type() == "non" {
               println!("error");
               break;
           }
           println!("Success");
        }
    }

    pub fn parse(&mut self) {
        self.program(0);
        self.pos = 0;
    }
}
