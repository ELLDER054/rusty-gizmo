#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Let {
        id: String,
        expr: Box<Expression>,
    },
    FuncCall {
        id: String,
        args: Vec<Box<Expression>>,
    },
    Non,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Int(i32),
    Dec(String),
    Bool(bool),
    Str(String),
    BinaryOperator {
        oper: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOperator {
        oper: String,
        child: Box<Expression>,
    },
    Non,
}

impl Expression {
    fn binary_rules(&self, oper: &String, left: &Box<Expression>, right: &Box<Expression>) -> &str {
        match oper.as_str() {
            "+" => match (*left).validate() {
                "int" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                "dec" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                "string" => match (*right).validate() {
                    "string" => "string",
                    "char" => "string",
                    _ => "error",
                },
                "char" => match (*right).validate() {
                    "string" => "string",
                    "char" => "string",
                    _ => "error",
                },
                _ => "error",
            },
            "-" | "*" | "/" => match (*left).validate() {
                "int" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                "dec" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                _ => "error",
            },
            "==" | "!=" => match (*left).validate() {
                t if (*right).validate() == t => "bool",
                _ => "error",
            },
            ">=" | "<=" | ">" | "<" => match (*left).validate() {
                "int" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                "dec" => match (*right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                },
                _ => "error",
            },
            "and" | "or" => match (*left).validate() {
                "bool" => match (*right).validate() {
                    "bool" => "bool",
                    _ => "error",
                },
                _ => "error",
            },
            _ => "error",
        }
    }

    fn unary_rules(&self, oper: &String, child: &Box<Expression>) -> &str {
        return match oper.as_str() {
            "-" => match (*child).validate() {
                "int" => "int",
                "dec" => "dec",
                _ => "error",
            }
            "not" => match (*child).validate() {
                _ => "bool",
            }
            _ => "error",
        }
    }

    pub fn validate(&self) -> &str {
        match self {
            Expression::Int(_i) => "int",
            Expression::Dec(_d) => "dec",
            Expression::Bool(_b) => "bool",
            Expression::Str(_s) => "string",
            Expression::BinaryOperator {oper, left, right} => self.binary_rules(oper, left, right),
            Expression::UnaryOperator {oper, child} => self.unary_rules(oper, child),
            Expression::Non => "",
        }
    }
}
