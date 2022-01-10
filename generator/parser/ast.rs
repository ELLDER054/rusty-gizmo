pub trait Node {
    fn get_type(&self) -> &str;
    fn is_none(&self) -> bool;
}

pub enum Nodes {
    Let(Let),
}

impl Node for Nodes {
    fn get_type(&self) -> &str {
        return match self {
            Nodes::Let(l) => l.get_type()
        }
    }

    fn is_none(&self) -> bool {
        return match self {
            Nodes::Let(l) => l.is_none()
        }
    }
}

pub trait Validated {
    fn validate(&self) -> &str;
    fn get_type(&self) -> &str;
    fn is_none(&self) -> bool;
}

pub struct Let {
    pub id: String,
    pub expr: Box<dyn Validated>,
    pub none: bool,
}

impl Node for Let {
    fn get_type(&self) -> &str {
        return "let";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}

pub struct Int {
    pub value: i32,
    pub none: bool,
}

impl Validated for Int {
    fn validate(&self) -> &str {
        return "int";
    }

    fn get_type(&self) -> &str {
        return "int";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}

pub struct Operator {
    pub left: Box<dyn Validated>,
    pub oper: String,
    pub right: Box<dyn Validated>,
    pub none: bool,
}

impl Validated for Operator {
    fn validate(&self) -> &str {
        return match self.oper.as_str() {
            "+" => {match (*self.left).validate() {
                "int" => {match (*self.right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                }},
                "dec" => {match (*self.right).validate() {
                    "int" | "dec" => "dec",
                    _ => "error"
                }},
                "string" => {match (*self.right).validate() {
                    "string" => "string",
                    _ => "error"
                }},
                _ => "non",
            }},
            "-" | "*" | "/" => {match (*self.left).validate() {
                "int" => {match (*self.right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                }},
                "dec" => {match (*self.right).validate() {
                    "int" | "dec" => "dec",
                    _ => "error"
                }},
                "" => {match (*self.right).validate() {
                    "int" => "int",
                    "dec" => "dec",
                    _ => "error",
                }},
                _ => "non",
            }},
            ">" | "<" | ">=" | "<=" => {match (*self.left).validate() {
                "int" => {match (*self.right).validate() {
                    "int" | "dec" => "bool",
                    _ => "error",
                }},
                "dec" => {match (*self.right).validate() {
                    "int" | "dec" => "bool",
                    _ => "error"
                }},
                _ => "non",
            }},
            "and" | "or" => {match (*self.left).validate() {
                "bool" => {match (*self.right).validate() {
                    "bool" => "bool",
                    _ => "error",
                }},
                _ => "non",
            }},
            "==" | "!=" => {match (*self.left).validate() {
                typ if (*self.right).validate() == typ => "bool",
                _ => "error",
            }},
            _ => "non",
        };
    }

    fn get_type(&self) -> &str {
        return "oper";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}

pub struct UnaryOperator {
    pub oper: String,
    pub right: Box<dyn Validated>,
    pub none: bool,
}

impl Validated for UnaryOperator {
    fn validate(&self) -> &str {
        return match self.oper.as_str() {
            "not" => {match (*self.right).validate() {
                "bool" => "bool",
                _ => "error",
            }},
            "-" => {match (*self.right).validate() {
                "int" => "int",
                "dec" => "dec",
                _ => "error",
            }},
            _ => "non",
        };
    }

    fn get_type(&self) -> &str {
        return "oper";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}


pub struct Dec {
    pub value: f64,
    pub none: bool,
}

impl Validated for Dec {
    fn validate(&self) -> &str {
        return "dec";
    }

    fn get_type(&self) -> &str {
        return "dec";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}

pub struct Bool {
    pub value: bool,
    pub none: bool,
}

impl Validated for Bool {
    fn validate(&self) -> &str {
        return "bool";
    }

    fn get_type(&self) -> &str {
        return "bool";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}

pub struct Str {
    pub value: String,
    pub none: bool,
}

impl Validated for Str {
    fn validate(&self) -> &str {
        return "string";
    }

    fn get_type(&self) -> &str {
        return "str";
    }

    fn is_none(&self) -> bool {
        self.none
    }
}
