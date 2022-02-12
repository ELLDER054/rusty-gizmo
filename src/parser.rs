pub mod lexer;
mod ast;
pub mod symbol;
pub mod generator;

use self::lexer::token::Token;
use self::lexer::token::TokenType;
use self::lexer::error::error;
use self::lexer::error::ErrorType;
use self::ast::Node;
use self::ast::Expr;
use self::symbol::SymbolController;
use self::symbol::SymbolType;

pub struct Parser {
    pub pos: usize,
    pub tokens: Vec<Token>,
    pub symtable: SymbolController,
    pub id_c: i32
}

impl Parser {
    fn matches(&mut self, types: Vec<TokenType>) -> bool {
        for t in types {
            if self.check().typ == t {
                self.pos += 1;
                return true;
            }
        }

        return false;
    }

    fn consume(&mut self, t: TokenType, msg: &str) -> String {
        let found = self.matches(vec![t]);
        if !found {
            error(ErrorType::ExpectedToken, &self.previous())
                .note(msg)
                .emit();
        }
        return self.previous().value;
    }

    fn check(&mut self) -> Token {
        if self.eof() {
            return self.previous();
        }

        return self.peek();
    }

    fn peek(&mut self) -> Token {
        return self.tokens[self.pos].clone();
    }

    fn previous(&mut self) -> Token {
        return self.tokens[self.pos - 1].clone();
    }

    fn eof(&mut self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expression(&mut self) -> Expr {
        self.boolean()
    }

    fn boolean(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.matches(vec![TokenType::And, TokenType::Or]) {
            let oper = self.previous().value;
            let right = self.comparison();
            expr = Expr::BinaryOperator {
                left: Box::new(expr),
                oper: oper,
                right: Box::new(right)
            };
        }
        
        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.matches(vec![TokenType::Equal, TokenType::NotEqual]) {
            let oper = self.previous().value;
            let right = self.comparison();
            expr = Expr::BinaryOperator {
                left: Box::new(expr),
                oper: oper,
                right: Box::new(right)
            };
        }
        
        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.matches(vec![TokenType::GreaterThan, TokenType::GreaterEqual, TokenType::LessThan, TokenType::LessEqual]) {
            let oper = self.previous().value;
            let right = self.term();
            expr = Expr::BinaryOperator {
                left: Box::new(expr),
                oper: oper,
                right: Box::new(right)
            };
        }
        
        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.matches(vec![TokenType::Star, TokenType::Slash]) {
            let oper = self.previous().value;
            let right = self.factor();
            expr = Expr::BinaryOperator {
                left: Box::new(expr),
                oper: oper,
                right: Box::new(right)
            };
        }
        
        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.matches(vec![TokenType::Plus, TokenType::Dash]) {
            let oper = self.previous().value;
            let right = self.unary();
            expr = Expr::BinaryOperator {
                left: Box::new(expr),
                oper: oper,
                right: Box::new(right)
            };
        }
        
        return expr;
    }

    fn unary(&mut self) -> Expr {
        if self.matches(vec![TokenType::Not, TokenType::Dash]) {
            let oper = self.previous().value;
            let right = self.unary();
            return Expr::UnaryOperator {
                oper: oper,
                child: Box::new(right)
            };
        }
        
        return self.primary();
    }

    fn primary(&mut self) -> Expr {
        if self.matches(vec![TokenType::Int]) {return Expr::Int(self.previous().value);}
        if self.matches(vec![TokenType::Str]) {return Expr::Str(self.previous().value);}
        if self.matches(vec![TokenType::Id]) {
            let prev = self.previous();
            let symbol = self.symtable.find_global_var_error(prev.value.clone(), &prev);
            return Expr::Id(self.previous().value, symbol.typ, symbol.gen_id);
        }
        if self.matches(vec![TokenType::New]) {
            let id = self.consume(TokenType::Id, "Expect an identifier after this 'new'");
            self.consume(TokenType::LeftParen, "Expect an '(' after this identifier");

            let mut fields: Vec<Expr> = Vec::new();
            while self.peek().typ != TokenType::RightParen {
                let expr = self.expression();
                fields.push(expr.clone());
                let comma = self.matches(vec![TokenType::Comma]);
                if !comma {
                    break
                }
            }
            self.consume(TokenType::RightParen, "Expect an ')' after this expression");
            return Expr::NewStruct {id: id, fields: fields};
        }
        
        error(ErrorType::ExpectedToken, &self.previous())
            .note("Expect a constant expression")
            .emit();
        return Expr::Non;
    }

    fn let_statement(&mut self) -> Node {
        let id = self.consume(TokenType::Id, "Expect an identifier after this 'let'");
        self.consume(TokenType::Equal, "Expect an '=' after this identifier");
        let expr = self.expression();
        self.consume(TokenType::SemiColon, "Expect an ';' after this expression");

        self.id_c += 1;
        self.symtable.add_symbol(id.clone(), expr.validate().to_string(), SymbolType::Var, format!("%.{}", self.id_c - 1), None);
        return Node::Let {id: id, expr: expr, gen_id: format!("%.{}", self.id_c - 1)};
    }

    fn function_call(&mut self) -> Node {
        let id = self.previous().value;
        self.consume(TokenType::LeftParen, "Expect an '(' after this identifier");
        let mut args: Vec<Box<Expr>> = Vec::new();
        let mut arg_types: Vec<String> = Vec::new();
        while self.peek().typ != TokenType::RightParen {
            let expr = self.expression();
            args.push(Box::new(expr.clone()));
            arg_types.push(expr.validate().into());
            let comma = self.matches(vec![TokenType::Comma]);
            if !comma {
                break
            }
        }
        self.consume(TokenType::RightParen, "Expect an ')' after this expression");
        self.consume(TokenType::SemiColon, "Expect an ';' after this ')'");

        self.symtable.add_symbol(id.clone(), "".into(), SymbolType::Func, id.clone(), Some(arg_types));
        return Node::FuncCall {id: id, args: args};
    }

    fn parse_type(&mut self) -> Option<String> {
        if self.matches(vec![TokenType::Type]) {
            return Some(self.previous().value);
        }
        if self.matches(vec![TokenType::Id]) {
            let prev = self.previous();
            let symbol = self.symtable.find_global_struct_error(prev.value.clone(), &prev);
            return Some(symbol.id.clone());
        }

        return None;
    }

    fn struct_definition(&mut self) -> Node {
        let id = self.consume(TokenType::Id, "Expect an identifier after this 'struct'");
        self.consume(TokenType::LeftBrace, "Expect an '{' after this identifier");

        let mut fields: Vec<(String, String)> = Vec::new();
        while self.peek().typ != TokenType::RightBrace {
            let id = self.consume(TokenType::Id, "Expect an identifier after this ','");
            self.consume(TokenType::Colon, "Expect ':' after this identifier");
            let typ = self.parse_type();
            if typ == None {
                error(ErrorType::ExpectedToken, &self.previous())
                    .note("Expect a type after this ':'")
                    .emit();
            }
            fields.push((id, typ.unwrap()));
            let comma = self.matches(vec![TokenType::Comma]);
            if !comma {
                break
            }
        }
        self.consume(TokenType::RightBrace, "Expect an '}' after this type");
        return Node::Struct {id: id, fields: fields};
    }

    fn statement(&mut self) -> Node {
        if self.matches(vec![TokenType::Let]) {return self.let_statement();}
        if self.matches(vec![TokenType::Id]) {return self.function_call();}
        if self.matches(vec![TokenType::Struct]) {return self.struct_definition();}

        return Node::Non;
    }

    pub fn parse(&mut self) -> Vec<Box<Node>> {
        let mut stmts: Vec<Box<Node>> = Vec::new();
        while !self.eof() {
            let stmt = self.statement();
            stmts.push(Box::new(stmt));
        }
        return stmts;
    }
}
