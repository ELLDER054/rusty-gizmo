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

/// Returns the type of an operation "oper" "child" (i.e., -5 results in int)
fn unary_rules<'u>(oper: &'u String, child: &'u Box<Expression>) -> &'static str {
    match oper.as_str() {
        "-" => match (*child).validate() {
            "int" => "int",
            "dec" => "dec",
            _ => "error"
        },
        "not" => match (*child).validate() {
            _ => "bool",
        }
        _ => "error"
    }
}

/// Returns the type of an operation "left" "oper" "right" (i.e., 5 + 5 results in int)
fn binary_rules<'b>(oper: &'b String, left: &'b Box<Expression>, right: &'b Box<Expression>) -> &'static str {
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
        "-" | "*" => match (*left).validate() {
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
        "/" => match (*left).validate() {
            "int" => match (*right).validate() {
                "int" => "dec",
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
                "int" => "bool",
                _ => "error",
            },
            "dec" => match (*right).validate() {
                "dec" => "bool",
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

/// Implement functions for an expression node
impl Expression {
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
            Expression::BinaryOperator {oper, left, right} => binary_rules(oper, left, right),
            Expression::UnaryOperator {oper, child} => unary_rules(oper, child),
            Expression::NewStruct {id, fields: _} => id,
            Expression::StructDot {id: _, id2: _, typ, field_num: _} => typ,
            Expression::Non => "",
        }
    }
}

#[test]
fn test_validate() {
    assert_eq!(Expression::Int(5).validate(), "int");
    assert_eq!(Expression::Dec("16.788".to_string()).validate(), "dec");
    assert_eq!(Expression::Bool(true).validate(), "bool");
    assert_eq!(Expression::Str("Hello, World!".to_string()).validate(), "string");
    assert_eq!(Expression::Id("foo".to_string(), "int".to_string(), "%.0".to_string()).validate(), "int");
    assert_eq!(Expression::Array {values: vec![], typ: "int[]".to_string()}.validate(), "int[]");
    assert_eq!(Expression::IndexedValue {src: Box::new(Expression::Non), index: Box::new(Expression::Int(0)), new_typ: "int".to_string()}.validate(), "int");
    assert_eq!(Expression::NewStruct {id: "Foo".to_string(), fields: vec![]}.validate(), "Foo");
    assert_eq!(Expression::StructDot {id: Box::new(Expression::Non), id2: "def".to_string(), typ: "int".to_string(), field_num: 0}.validate(), "int");
}

#[test]
fn test_semantics() {
    let int =    Box::new(Expression::Int(5));
    let dec =    Box::new(Expression::Dec("5.5".to_string()));
    let boo =    Box::new(Expression::Bool(true));
    let string = Box::new(Expression::Str("test".to_string()));
    
    assert_eq!(unary_rules(&"-".to_string(), &int),        "int");
    assert_eq!(unary_rules(&"-".to_string(), &dec),        "dec");
    assert_eq!(unary_rules(&"-".to_string(), &string),     "error");

    assert_eq!(unary_rules(&"not".to_string(), &boo),      "bool");
    assert_eq!(unary_rules(&"not".to_string(), &dec),      "bool");


    assert_eq!(binary_rules(&"+".to_string(), &int, &int), "int");
    assert_eq!(binary_rules(&"-".to_string(), &int, &int), "int");
    assert_eq!(binary_rules(&"*".to_string(), &int, &int), "int");
    assert_eq!(binary_rules(&"/".to_string(), &int, &int), "dec");

    assert_eq!(binary_rules(&"+".to_string(), &dec, &dec), "dec");
    assert_eq!(binary_rules(&"-".to_string(), &dec, &dec), "dec");
    assert_eq!(binary_rules(&"*".to_string(), &dec, &dec), "dec");
    assert_eq!(binary_rules(&"/".to_string(), &dec, &dec), "dec");

    assert_eq!(binary_rules(&"<".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&"<".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&"<".to_string(), &string, &string), "error");

    assert_eq!(binary_rules(&">".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&">".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&">".to_string(), &string, &string), "error");

    assert_eq!(binary_rules(&"<=".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&"<=".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&"<=".to_string(), &string, &string), "error");

    assert_eq!(binary_rules(&">=".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&">=".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&">=".to_string(), &string, &string), "error");

    assert_eq!(binary_rules(&"==".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&"==".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&"==".to_string(), &string, &dec), "error");

    assert_eq!(binary_rules(&"!=".to_string(), &dec, &dec), "bool");
    assert_eq!(binary_rules(&"!=".to_string(), &int, &int), "bool");
    assert_eq!(binary_rules(&"!=".to_string(), &string, &dec), "error");

    assert_eq!(binary_rules(&"and".to_string(), &boo, &boo), "bool");
    assert_eq!(binary_rules(&"and".to_string(), &int, &int), "error");
    assert_eq!(binary_rules(&"and".to_string(), &string, &dec), "error");

    assert_eq!(binary_rules(&"or".to_string(), &boo, &boo), "bool");
    assert_eq!(binary_rules(&"or".to_string(), &int, &int), "error");
    assert_eq!(binary_rules(&"or".to_string(), &string, &dec), "error");
}
