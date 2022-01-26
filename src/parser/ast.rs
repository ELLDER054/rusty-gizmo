/// An enum to store each possible Node
#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    /// Let statement
    /// let a: int = 5;
    /// or
    /// let a = 5;
    Let {
        id: String,
        expr: Expression,
        gen_id: String,
    },

    /// Assign statement
    /// let a: int = 5;
    /// or
    /// let a = 5;
    Assign {
        id: Expression,
        expr: Expression,
    },

    /// Function call
    /// write(5);
    FuncCall {
        id: String,
        args: Vec<Box<Expression>>,
    },

    /// Struct definition
    /// struct foo {
    ///     bar: int
    /// }
    Struct {
        id: String,
        fields: Vec<(String, String)>
    },

    Non,
}

/// An enum to store each possible expression node
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    /// Integer
    /// # Example
    /// ```rust
    /// let my_int: int = 10;
    /// ```
    Int(i32),

    /// Decimal number
    /// # Example
    /// ```rust
    /// let my_dec: dec = 5.5;
    /// ```
    Dec(String),

    /// Boolean value (true or false)
    /// # Example
    /// ```rust
    /// let my_true: bool = true;
    /// let my_false: bool = false;
    /// ```
    Bool(bool),

    /// String value
    /// # Example
    /// ```rust
    /// let my_str: string = "This is my first string";
    Str(String),

    /// Identifier
    Id(String, String, String),

    /// Array
    Array {
        values: Vec<Expression>,
        typ: String
    },

    /// An index
    /// # Example
    /// `array[num]` or `string[num]`
    IndexedValue {
        src: Box<Expression>,
        index: Box<Expression>,
        new_typ: String
    },

    /// Binary operator
    /// # Example
    /// `5 + 6`
    BinaryOperator {
        oper: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Unary operator
    /// # Example
    /// `-6`
    UnaryOperator {
        oper: String,
        child: Box<Expression>,
    },

    /// New struct
    /// # Example
    /// let foo: Foo = new Foo(5, 6, 7);
    NewStruct {
        id: String,
        fields: Vec<Expression>
    },

    /// Struct dot identifier
    /// # Example
    /// let s: string = Foo.bar;
    StructDot {
        id: Box<Expression>,
        id2: String,
        typ: String,
        field_num: i32
    },

    Non,
}

/// Implement functions for an expression node
impl Expression {
    /// Returns the type of an operation "left" "oper" "right" (i.e., 5 + 5 results in int)
    fn binary_rules(&self, oper: &String, left: &Box<Expression>, right: &Box<Expression>) -> &str {
        match oper.as_str() {
            // Match the operator
            "+" => match (*left).validate() {
                // After matching the operator, match the left side
                "int" => match (*right).validate() {
                    // Once the left side is known, match the right side
                    "int" => "int",
                    _ => "error",
                },
                "dec" => match (*right).validate() {
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
                    _ => "error",
                },
                "dec" => match (*right).validate() {
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
                    _ => "error",
                },
                "dec" => match (*right).validate() {
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
                _ => "error"
            },
            "++" | "--" => match (*child).validate() {
                "int" => "int",
                _ => "error"
            },
            "not" => match (*child).validate() {
                _ => "bool",
            }
            _ => "error"
        }
    }

    /// Validates the type of an expression
    pub fn validate(&self) -> &str {
        match self {
            // Match each kind of expression node to find it's type
            Expression::Int(_i) => "int",
            Expression::Dec(_d) => "dec",
            Expression::Bool(_b) => "bool",
            Expression::Str(_s) => "string",
            Expression::Id(_i, t, _gen_id) => t,
            Expression::Array {values: _, typ} => typ.as_str(),
            Expression::IndexedValue {src: _, index: _, new_typ} => new_typ.as_str(),
            Expression::BinaryOperator {oper, left, right} => self.binary_rules(oper, left, right),
            Expression::UnaryOperator {oper, child} => self.unary_rules(oper, child),
            Expression::NewStruct {id, fields: _} => id,
            Expression::StructDot {id: _, id2: _, typ, field_num: _} => typ,
            Expression::Non => "",
        }
    }
}
