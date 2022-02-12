/// An enum to store each possible Node

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    /// Let statement
    /// let a: int = 5;
    /// or
    /// let a = 5;
    Let {
        id: String,
        expr: Expr,
        gen_id: String,
    },

    /// Function declaration
    FuncDecl {
        id: String,
        typ: String,
        args: Vec<(String, String)>,
        body: Box<Node>
    },

    /// Return statement
    /// # Example
    /// ret 10;
    Ret {
        expr: Expr
    },

    /// Break or continue statement
    /// # Example
    /// break;
    /// or
    /// continue;
    Pause {
        label: usize,
    },

    /// Use statement
    /// # Example
    /// use "file2.gizmo";
    Use {
    },

    /// If statement
    /// if "abc" == "abc" {
    ///     write("Yay");
    /// }
    If {
        cond: Expr,
        body: Box<Node>,
        else_body: Option<Box<Node>>,
        begin: i32,
        else_: i32,
        end: i32
    },

    /// Assign statement
    /// let a: int = 5;
    /// or
    /// let a = 5;
    Assign {
        id: Expr,
        expr: Expr,
    },

    /// Function call
    /// write(5);
    FuncCall {
        id: String,
        args: Vec<Box<Expr>>,
    },

    /// Struct definition
    /// struct foo {
    ///     bar: int
    /// }
    Struct {
        id: String,
        fields: Vec<(String, String)>
    },

    /// Block
    /// {
    ///     // Statements
    /// }
    Block {
        statements: Vec<Box<Node>>
    },

    /// While loop
    /// while condition {
    ///     // Statements
    /// }
    While {
        cond: Expr,
        body: Box<Node>,
        begin: usize,
        end: usize
    },

    Non,
}

/// An enum to store each possible expression node
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    /// Integer
    /// # Example
    /// ```rust
    /// let my_int: int = 10;
    /// ```
    Int(String),

    /// Character
    /// # Example
    /// ```rust
    /// let my_chr: char = 'a';
    /// ```
    Chr(char),

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
        values: Vec<Expr>,
        typ: String
    },

    /// An index
    /// # Example
    /// `array[num]` or `string[num]`
    IndexedValue {
        src: Box<Expr>,
        index: Box<Expr>,
        new_typ: String
    },

    /// Binary operator
    /// # Example
    /// `5 + 6`
    BinaryOperator {
        oper: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Unary operator
    /// # Example
    /// `-6`
    UnaryOperator {
        oper: String,
        child: Box<Expr>,
    },

    /// New struct
    /// # Example
    /// let foo: Foo = new Foo(5, 6, 7);
    NewStruct {
        id: String,
        fields: Vec<Expr>
    },

    /// Struct dot identifier
    /// # Example
    /// let s: string = Foo.bar;
    StructDot {
        id: Box<Expr>,
        id2: String,
        typ: String,
        field_num: i32
    },

    /// Function call
    /// write(5);
    FuncCall {
        id: String,
        typ: String,
        args: Vec<Box<Expr>>
    },

    Non,
}

/// Returns the type of an operation "oper" "child" (i.e., -5 results in int)
fn unary_rules<'u>(oper: &'u String, child: &'u Box<Expr>) -> &'static str {
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
fn binary_rules<'b>(oper: &'b String, left: &'b Box<Expr>, right: &'b Box<Expr>) -> &'static str {
    match oper.as_str() {
        // Match the operator
        "+" => match (*left).validate() {
            // After matching the operator, match the left side
            "int" => match (*right).validate() {
                // Once the left side is known, match the right side
                "int" | "char" => "int",
                _ => "error",
            },
            "dec" => match (*right).validate() {
                "dec" => "dec",
                _ => "error",
            },
            "char" => match (*right).validate() {
                "int" | "char" => "char",
                _ => "error"
            },
            _ => "error",
        },
        "-" | "*" => match (*left).validate() {
            "int" => match (*right).validate() {
                "int" | "char" => "int",
                _ => "error",
            },
            "char" => match(*right).validate() {
                "int" | "char" => "char",
                _ => "error"
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
                "int" | "char" => "bool",
                _ => "error",
            },
            "char" => match (*right).validate() {
                "int" | "char" => "bool",
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
impl Expr {
    /// Validates the type of an expression
    pub fn validate(&self) -> &str {
        match self {
            // Match each kind of expression node to find it's type
            Expr::Int(_i) => "int",
            Expr::Chr(_c) => "char",
            Expr::Dec(_d) => "dec",
            Expr::Bool(_b) => "bool",
            Expr::Str(_s) => "string",
            Expr::Id(_i, t, _gen_id) => t,
            Expr::Array {typ, ..} => typ.as_str(),
            Expr::IndexedValue {new_typ, ..} => new_typ.as_str(),
            Expr::BinaryOperator {oper, left, right} => binary_rules(oper, left, right),
            Expr::UnaryOperator {oper, child} => unary_rules(oper, child),
            Expr::NewStruct {id, ..} => id,
            Expr::StructDot {typ, ..} => typ,
            Expr::FuncCall {typ, ..} => typ.as_str(),
            Expr::Non => "",
        }
    }
}

#[test]
fn test_validate() {
    assert_eq!(Expr::Int(5).validate(), "int");
    assert_eq!(Expr::Chr('a').validate(), "char");
    assert_eq!(Expr::Dec("16.788".to_string()).validate(), "dec");
    assert_eq!(Expr::Bool(true).validate(), "bool");
    assert_eq!(Expr::Str("Hello, World!".to_string()).validate(), "string");
    assert_eq!(Expr::Id("foo".to_string(), "int".to_string(), "%.0".to_string()).validate(), "int");
    assert_eq!(Expr::Array {values: vec![], typ: "int[]".to_string()}.validate(), "int[]");
    assert_eq!(Expr::IndexedValue {src: Box::new(Expr::Non), index: Box::new(Expr::Int(0)), new_typ: "int".to_string()}.validate(), "int");
    assert_eq!(Expr::NewStruct {id: "Foo".to_string(), fields: vec![]}.validate(), "Foo");
    assert_eq!(Expr::StructDot {id: Box::new(Expr::Non), id2: "def".to_string(), typ: "int".to_string(), field_num: 0}.validate(), "int");
}

#[test]
fn test_semantics() {
    let int =    Box::new(Expr::Int(5));
    let dec =    Box::new(Expr::Dec("5.5".to_string()));
    let boo =    Box::new(Expr::Bool(true));
    let string = Box::new(Expr::Str("test".to_string()));
    
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
