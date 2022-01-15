// An enum to store each possible Node
#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Let {
        id: String,
        expr: Box<Expression>,
        gen_id: String,
    },
    FuncCall {
        id: String,
        args: Vec<Box<Expression>>,
    },
    Non,
}

// An enum to store each possible expression node
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Int(i32),
    Dec(String),
    Bool(bool),
    Str(String),
    Id(String, String, Vec<String>, String),
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

// Implement functions for an expression node
impl Expression {
    // Returns the type of an operation "left" "oper" "right" (i.e., 5 + 5 results in int)
    fn binary_rules(&self, oper: &String, left: &Box<Expression>, right: &Box<Expression>) -> &str {
        match oper.as_str() {
            // Match the operator
            "+" => match (*left).validate() {
                // After matching the operator, match the left side
                "int" => match (*right).validate() {
                    // Once the left side is known, match the right side
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
            // Match each kind of expression node to find it's type
            Expression::Int(_i) => "int",
            Expression::Dec(_d) => "dec",
            Expression::Bool(_b) => "bool",
            Expression::Str(_s) => "string",
            Expression::Id(_i, t, _a, gen_id) => t,
            Expression::BinaryOperator {oper, left, right} => self.binary_rules(oper, left, right),
            Expression::UnaryOperator {oper, child} => self.unary_rules(oper, child),
            Expression::Non => "",
        }
    }
}
