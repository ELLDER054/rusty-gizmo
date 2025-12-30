use super::ast::Node;
use super::ast::Expression;
use super::symbol::Argument;

/// Converts a Gizmo type to an llvm ir type
fn type_of(typ: String) -> String {
    let struct_type = format!("%{}", typ);
    match typ.as_str() {
        "int"    => "i32",
        "dec"    => "double",
        "bool"   => "i1",
        "char"   => "i8",
        "string" => "i8*",
        "void" => "void",
        arr if arr.ends_with(']') => "%.Arr",
        _ => struct_type.as_str()
    }.to_string()
}

/// Converts a Gizmo operator to an llvm ir operator
fn type_of_oper(oper: String) -> String {
    match oper.as_str() {
        "+"   => "add",
        "-"   => "sub",
        "*"   => "mul",
        "/"   => "sdiv",
        "=="  => "icmp eq",
        "!="  => "icmp ne",
        "<"   => "icmp slt",
        ">"   => "icmp sgt",
        "<="  => "icmp sle",
        ">="  => "icmp sge",
        "and" => "and",
        "or"  => "or",
        _ => ""
    }.to_string()
}

/// Stores information for an ir builder
pub struct IRBuilder {
    /// Contains the ir code
    pub code: String,

    /// String that will be added to the end
    pub ends: String,

    /// The number of ir variables created
    pub ssa_num: i32,

    /// Saves off the number of ir variables created
    pub save_ssa_num: i32,

    /// The number of strings created
    pub str_num: i32,
}

/// Implement functions for an ir builder
impl IRBuilder {
    /// Constructs a new ir builder
    fn construct() -> IRBuilder {
        IRBuilder {code: "define i32 @main() {\nentry:\n".to_string(), ends: "\tret i32 0\n}\n".to_string(), ssa_num: 0, str_num: 0, save_ssa_num: 0}
    }

    /// Creates an alloca statement
    /// # Example
    /// %0 = alloca i32
    fn create_alloca(&mut self, typ: String, name: Option<String>) -> String {
        self.code.push_str(format!("\t{} = alloca {}\n", name.clone().unwrap_or(format!("%{}", self.ssa_num)), typ).as_str());

        // If the caller didn't specify a name, use the ssa_num
        if name == None {
            self.ssa_num += 1;
        }
        format!("{}", name.clone().unwrap_or(format!("%{}", self.ssa_num - 1)))
    }
    
    /// Creates a store statement
    /// # Example
    /// %1 = store i32 5, i32 %0
    fn create_store(&mut self, src: String, dst: String, typ: String) {
        self.code.push_str(format!("\tstore {} {}, {0}* {}\n", typ, src, dst).as_str());
    }
    
    /// Creates a load statement
    /// # Example
    /// %1 = load i32, i32* %0
    fn create_load(&mut self, typ: String, src: String) -> String {
        self.code.push_str(format!("\t%{} = load {}, {1}* {}\n", self.ssa_num, typ, src).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }
    
    /// Creates a bitcast statement
    /// # Example
    /// %1 = bitcast i32* %0, i8*
    fn create_bitcast(&mut self, typ: String, name: String, typ2: String) -> String {
        self.code.push_str(format!("\t%{} = bitcast {} {} to {}\n", self.ssa_num, typ, name, typ2).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }

    /// Creates a getelementptr statement
    /// # Example
    /// %0 = getelementptr inbounds [3 x i8], [3 x i8]* @.str, i32 0, i32 0
    fn create_gep(&mut self, typ: String, name: String, indices: Vec<String>) -> String {
        self.code.push_str(format!("\t%{} = getelementptr inbounds {}, {1}* {}", self.ssa_num, typ, name).as_str());
        self.ssa_num += 1;

        // Add each of the indices
        for indice in indices {
            self.code.push_str(format!(", i32 {}", indice).as_str());
        }
        self.code.push('\n');
        format!("%{}", self.ssa_num - 1)
    }

    /// Creates a global statement
    /// # Example
    /// @.str = constant [4 x i8] c"abc\00"
    fn create_global(&mut self, id: String, value: String) -> String {
        self.code = format!("{} = constant {}\n\n{}", id, value, self.code);
        format!("{}", id)
    }

    /// Adds a string to the end of the code
    fn create_ends(&mut self, s: String) {
        self.ends.push_str(s.as_str());
    }

    /// Saves the current ssa_num into a save variable
    /// Resets the ssa_num to 0
    fn enter_function(&mut self) {
        self.save_ssa_num = self.ssa_num;
        self.ssa_num = 0;
    }

    /// Resets the ssa_num to the saved variable
    fn exit_function(&mut self) {
        self.ssa_num = self.save_ssa_num;
    }

    /// Creates a new struct
    /// # Example
    /// %.Arr = type {
    ///     i8*,
    ///     i32
    /// }
    fn create_new_struct(&mut self, id: String, fields: Vec<Argument>) -> String {
        // New string to store the fields
        let mut cat_fields = String::new();

        // Counts the number of fields
        let mut num_fields = 0;

        // Iterate of the the fields and add them to 'cat_fields'
        for field in fields.clone().iter() {
            cat_fields.push_str(format!("\t{}", type_of(field.clone().typ)).as_str());

            // If the last field is not reached, add a comma
            if num_fields + 1 < fields.clone().len() {
                cat_fields.push(',');
            }
            cat_fields.push('\n');

            // Increment the number of fields
            num_fields += 1;
        }
        self.code = format!("%{} = type {{\n{}}}\n\n{}", id, cat_fields, self.code);
        format!("@{}", id)
    }

    /// Creates an operation
    /// # Example
    /// %0 = add i32 5, 6
    fn create_operation(&mut self, oper: String, typ: String, left: String, right: String) -> String {
        self.code.push_str(format!("\t%{} = {} {} {}, {}\n", self.ssa_num, type_of_oper(oper), type_of(typ), left, right).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }
}

/// Stores information for a code generator
pub struct Generator {
    /// Stores an ir builder
    pub ir_b: IRBuilder,

    /// Number of formatted strings
    pub format_num: usize,

    /// Whether or %.Arr was declared
    pub has_array: bool,

    /// Whether or not @printf was declared
    pub dec_printf: bool,

    /// Whether or not @strlen was declared
    pub dec_strlen: bool,

    /// Whether or not @.int was declared
    pub dec_int: bool,

    /// Whether or not @.dec was declared
    pub dec_dec: bool,

    /// Whether or not @.char was declared
    pub dec_char: bool,

    /// Whether or not @.str was declared
    pub dec_str: bool,
}

impl Generator {
    /// Constructs a new code generator
    pub fn construct() -> Generator {
        Generator {ir_b: IRBuilder::construct(), has_array: false, dec_printf: false, dec_int: false, dec_dec: false, dec_char: false, dec_str: false, format_num: 0, dec_strlen: false}
    }

    /// Destructs the code generator
    pub fn destruct(&mut self) {
        self.ir_b.code.push_str(self.ir_b.ends.as_str());
    }

    /// Iterates through the nodes and generates ir for them
    pub fn generate(&mut self, nodes: Vec<Box<Node>>) {
        for node in nodes.iter() {
            match *node.clone() {
                Node::Let {id: _, expr, gen_id} => self.generate_let_stmt(expr.clone(), gen_id.clone()),
                Node::Ret {expr} => self.generate_ret_stmt(expr.clone()),
                Node::Pause {label} => {
                    self.ir_b.code.push_str(format!("\tbr label %l{}\n", label).as_str());
                    self.ir_b.ssa_num += 1;
                },
                Node::FuncDecl {id, typ, arguments, body} => self.generate_func_decl(id.clone(), typ.clone(), arguments.clone(), body.clone()),
                Node::While {cond, body, begin, end} => self.generate_while_loop(cond.clone(), body.clone(), begin, end),
                Node::If {cond, body, else_body, begin, else_, end} => self.generate_if_stmt(cond.clone(), body.clone(), else_body.clone(), begin, else_, end),
                Node::Assign {id, expr} => self.generate_assign_stmt(id.clone(), expr.clone()),
                Node::FuncCall {id, args} => {
                    self.generate_func_call(id.clone(), "void".to_string(), args.clone());
                },
                Node::Struct {id, fields} => self.generate_struct_decl(id.clone(), fields.clone()),
                Node::Block {statements} => {
                    self.generate(statements);
                }
                _ => {}
            }
        }
    }

    /// Gets the length of a string
    /// # Example
    /// abc\0Adef\00 -> 8
    fn get_str_length(&self, s: String) -> (i32, String) {
        let mut found = String::new();
        let mut pos = 0;
        for c in s.chars() {
            if c == '.' {
                return (found.parse::<i32>().unwrap_or(0) + 1, s[pos + 1..s.len()].to_string());
            }
            pos += 1;
            found.push(c);
        }
        return (0, String::new());
    }

    /// Generates an expression
    /// # Example
    /// 5 -> i32 5
    /// a -> %.1
    /// ...
    pub fn generate_expression(&mut self, expr: Expression, load_id: bool) -> String {
        match expr.clone() {
            Expression::Int(i) => i.to_string(),
            Expression::Chr(c) => (c as i32).to_string(),
            Expression::Dec(d) => d.to_string(),
            Expression::Bool(b) => b.to_string(),
            Expression::Str(s) => {
                // Get the length of the string
                let (length, rest) = self.get_str_length(s.clone());

                // Create a new global string
                let global = self.ir_b.create_global(format!("@.str.{}", self.ir_b.str_num), format!("[{} x i8] c\"{}\\00\"", length, rest));
               
                // Increment the number of strings created
                self.ir_b.str_num += 1;

                self.ir_b.create_gep(format!("[{} x i8]", length), global, vec!["0".to_string(), "0".to_string()])
            },
            Expression::Id(_id, typ, gen_id) => {
                // If the caller wants to load the identifiers into a true
                // value instead of keeping them pointers
                if load_id == true {
                    self.ir_b.create_load(type_of(typ), gen_id)
                } else {
                    gen_id
                }
            }
            Expression::NewStruct {id, fields} => {
                // Allocate a new struct
                let begin = self.ir_b.create_alloca(type_of(id.clone()), None);

                // Number of fields
                let mut field_num = 0;

                // Iterate through the fields
                for field in fields.iter() {
                    // Generate the expression
                    let gen_field = self.generate_expression(field.clone(), true);

                    // Find the correct place to store the expression
                    let gep = self.ir_b.create_gep(type_of(id.clone()), begin.clone(), vec!["0".to_string(), field_num.to_string()]);

                    // Store the expression in it's place
                    self.ir_b.create_store(gen_field, gep, type_of(field.validate().to_string()));

                    // Increment the number of fields found
                    field_num += 1;
                }
                self.ir_b.create_load(type_of(id), begin)
            }
            Expression::StructDot {id, typ, field_num, ..} => {
                // Generates the left side of the '.'
                let gen_begin = self.generate_expression(*id.clone(), false);

                // Index the left side with the field number
                let gep = self.ir_b.create_gep(type_of(id.validate().to_string()), gen_begin, vec!["0".to_string(), field_num.to_string()]);
                if load_id == true {
                    self.ir_b.create_load(type_of(typ.clone()), gep)
                } else {
                    gep
                }
            }
            Expression::FuncCall {id, typ, args} => {
                if typ.as_str() == "void" {
                    self.generate_func_call(id, "void".to_string(), args)
                } else {
                    self.generate_func_call(id, type_of(typ), args)
                }
            },
            Expression::Array {values, ..} => {
                // If %.Arr isn't already declared, declare it
                if !self.has_array {
                    self.ir_b.create_new_struct(".Arr".to_string(), vec![Argument {name: "".to_string(), typ: "string".to_string(), id_c: usize::MAX - 1}, Argument {name: "".to_string(), typ: "int".to_string(), id_c: usize::MAX}]);
                    self.has_array = true;
                }

                // Store the type of the first element
                let v_typ = type_of(values[0].clone().validate().to_string());

                // Allocate the array
                let alloca = self.ir_b.create_alloca("%.Arr".to_string(), None);

                // Allocate a temporary sized array
                let sized_alloca = self.ir_b.create_alloca(format!("[{} x {}]", values.len(), v_typ), None);

                // Number of elements
                let mut value_num = 0;
                
                // Iterate through the elements
                for value in values.iter() {
                    // Generate the element
                    let gen_value = self.generate_expression((*value).clone(), true);

                    // Get the pointer to the elements location
                    let gep = self.ir_b.create_gep(format!("[{} x {}]", values.len(), v_typ), sized_alloca.clone(), vec!["0".to_string(), value_num.to_string()]);

                    // Store the element in it's location
                    self.ir_b.create_store(gen_value, gep, type_of(value.clone().validate().to_string()));
                    value_num += 1;
                }

                // Bitcast the sized array to a pointer
                let bitcast = self.ir_b.create_bitcast(format!("[{} x {}]*", values.len(), v_typ), sized_alloca, "i8*".to_string());

                // Store the pointer in the array
                let gep2 = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "0".to_string()]);
                self.ir_b.create_store(bitcast, gep2, "i8*".to_string());

                // Store the length of the array in the array as well
                let gep3 = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "1".to_string()]);
                self.ir_b.create_store(values.len().to_string(), gep3, "i32".to_string());

                // Load the array
                self.ir_b.create_load("%.Arr".to_string(), alloca)
            }
            Expression::IndexedValue {src, index, new_typ} => {
                // Generate the value being indexed
                let gen_src = self.generate_expression(*src.clone(), true);

                // Generate the index
                let gen_index = self.generate_expression(*index.clone(), true);

                match src.validate() {
                    "string" => {
                        // Bitcast the i8* to a [0 x i8]*
                        let bitcast = self.ir_b.create_bitcast("i8*".to_string(), gen_src, "[0 x i8]*".to_string());

                        // Get and load the location of the index
                        let gep = self.ir_b.create_gep("[0 x i8]".to_string(), bitcast, vec!["0".to_string(), gen_index]);
                        self.ir_b.create_load("i8".to_string(), gep)
                    }
                    _ => {
                        // Allocate a temporary %.Arr* and store the %.Arr in
                        // it
                        let alloca = self.ir_b.create_alloca("%.Arr".to_string(), None);
                        self.ir_b.create_store(gen_src, alloca.clone(), "%.Arr".to_string());

                        // Get and load the i8* from the %.Arr*
                        let gep = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "0".to_string()]);
                        let load = self.ir_b.create_load("i8*".to_string(), gep);

                        // Bitcast the i8* to a [0 x i8]*
                        let bitcast = self.ir_b.create_bitcast("i8*".to_string(), load, format!("[0 x {}]*", type_of(new_typ.clone())));

                        // Get and load the location of the index
                        let gep2 = self.ir_b.create_gep(format!("[0 x {}]", type_of(new_typ.clone())), bitcast, vec!["0".to_string(), gen_index]);
                        if load_id == true {
                            self.ir_b.create_load(type_of(new_typ.clone()), gep2)
                        } else {
                            gep2
                        }
                    }
                }
            }
            Expression::BinaryOperator {oper, left, right} => {
                // Generate the left and right sides of the expression
                let gen_left = self.generate_expression((*left).clone(), true);
                let gen_right = self.generate_expression((*right).clone(), true);

                // Call the ir builder to create the operation
                self.ir_b.create_operation(oper, left.clone().validate().to_string(), gen_left, gen_right)
            }
            Expression::UnaryOperator {oper, child} => {
                let gen_child = self.generate_expression((*child).clone(), true);
                if oper == "-".to_string() {
                    // Having a negative value is the same as multiplying
                    // the value by -1
                    // -5 and 5 * -1 are equal
                    return self.ir_b.create_operation("*".to_string(), child.clone().validate().to_string(), gen_child.clone(), "-1".to_string());
                } else {
                    // Having a 'not' value is the same as subtracting
                    // 1 by the value
                    // not 0 and 1 - 0 are equal
                    return self.ir_b.create_operation("-".to_string(), child.clone().validate().to_string(), gen_child, "1".to_string());
                }
            }
            _ => "".to_string()
        }
    }

    /// Generates code for a 'let' statement
    fn generate_let_stmt(&mut self, expr: Expression, gen_id: String) {
        // Generate the value
        let gen_expr = self.generate_expression(expr.clone(), true);

        // Allocate a pointer of that type
        let var = self.ir_b.create_alloca(type_of(expr.clone().validate().to_string()), Some(gen_id));

        // Store the value into the pointer
        self.ir_b.create_store(gen_expr, var, type_of(expr.clone().validate().to_string()));
    }

    /// Generates code for a return statement
    fn generate_ret_stmt(&mut self, expr: Expression) {
        let gen_expr = self.generate_expression(expr.clone(), true);
        self.ir_b.code.push_str(format!("\tret {} {}\n", type_of(expr.validate().to_string()), gen_expr).as_str());
        self.ir_b.ssa_num += 1;
    }

    /// Generates code for a function declaration
    fn generate_func_decl(&mut self, id: String, typ: String, args: Vec<Argument>, body: Box<Node>) {
        // Save the current code
        let save = self.ir_b.code.clone();

        // New string to store the generated arguments
        let mut arg_code = String::new();

        // Number of arguments
        let mut arg_num = 0;

        // Iterate through the arguments
        for arg in args.iter() {
            // Add the argument to the code
            arg_code.push_str(format!("{}* %.{}", type_of(arg.typ.clone()), arg.id_c).as_str());

            // If the end isn't reached, add a comma
            if arg_num + 1 < args.len() {
                arg_code.push_str(", ");
            }

            // Increment the argument number
            arg_num += 1;
        }

        if typ.clone() == "void" {
            self.ir_b.code = format!("define void @{}({}) {{\nentry:\n", id, arg_code);
        } else {
            self.ir_b.code = format!("define {} @{}({}) {{\nentry:\n", type_of(typ.clone()), id, arg_code);
        }
        
        // Tell the ir builder to enter a function
        self.ir_b.enter_function();

        // Generate the body of the function
        self.generate(vec![body]);

        let mut _alloca: String = String::new();
        let base_type = match typ.as_str() { 
            "int"    => "0",
            "dec"    => "0.0",
            "char"   => "32",
            "bool"   => "false",
            "string" => {
                _alloca = self.ir_b.create_alloca("i8".to_string(), None);
                self.ir_b.create_store("32".to_string(), _alloca.clone(), "i8".to_string());
                &_alloca
            },
            _ => ""
        };

        self.ir_b.code.push_str(format!("\tret {} {}\n", type_of(typ.clone()), base_type).as_str());

        // Tell the ir builder to exit a function
        self.ir_b.exit_function();

        // Reset the code
        self.ir_b.code = format!("{}\n}}\n{}", self.ir_b.code, save.clone());
    }

    /// Generates code for a while-loop
    fn generate_while_loop(&mut self, cond: Expression, body: Box<Node>, begin: usize, end: usize) {
        // Generate the condition
        let gen_cond = self.generate_expression(cond.clone(), true);

        // Jump to the while-label to start the loop
        self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond, begin.clone(), end, begin.clone()).as_str());
        
        // Generate the body of the loop
        self.generate(vec![body]);

        // Depending on the condition, either jump back and start the loop
        // again, or exit the loop
        let gen_cond2 = self.generate_expression(cond.clone(), true);
        self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond2, begin, end.clone(), end).as_str());
    }
    
    /// Generates code for an if-statement
    fn generate_if_stmt(&mut self, cond: Expression, body: Box<Node>, else_body: Option<Box<Node>>, begin: i32, else_: i32, end: i32) {
        // Generate the condition
        let gen_cond = self.generate_expression(cond.clone(), true);

        // Jump to the if-label to start the loop
        if else_body == None {
            self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond, begin.clone(), end, begin.clone()).as_str());
        } else {
            self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond, begin.clone(), else_, begin.clone()).as_str());
        }
        
        // Generate the body of the loop
        self.generate(vec![body]);
        self.ir_b.code.push_str(format!("\tbr label %l{}\n", end.clone()).as_str());

        match else_body {
            Some(e) => {
                self.ir_b.code.push_str(format!("l{}:\n", else_).as_str());
                self.generate(vec![e]);
                self.ir_b.code.push_str(format!("\tbr label %l{}\n", end.clone()).as_str());
            },
            None => {}
        }
        self.ir_b.code.push_str(format!("l{}:\n", end).as_str());
    }

    /// Generates code for an assignment
    fn generate_assign_stmt(&mut self, id: Expression, expr: Expression) {
        // Generate the value
        let gen_expr = self.generate_expression(expr.clone(), true);

        // Generate the left side of the equals sign
        // Make sure not to load the end id       vvvvv
        let gen_id = self.generate_expression(id, false);

        // Store the value into the id
        self.ir_b.create_store(gen_expr, gen_id, type_of(expr.clone().validate().to_string()));
    }

    /// Generates code for a function call
    fn generate_func_call(&mut self, id: String, typ: String, args: Vec<Box<Expression>>) -> String {
        // New string to store the arguments
        let mut arg_values = String::new();

        // Number of arguments
        let mut arg_num = 0;

        // Iterate through the arguments
        for arg in args.iter() {
            // Generate the argument expression
            let gen_arg = self.generate_expression(*arg.clone(), true);

            // Find the type of the current argument
            let typ = type_of((*arg.clone().validate()).to_string());

            // If the function call is a built-in, don't make the argument
            // a pointer
            // Otherwise, make the argument a pointer
            if id.clone().as_str() != "write" && id.clone().as_str() != "len" {
                // Allocate space for the pointer
                let alloca = self.ir_b.create_alloca(typ.clone(), None);

                // Store the value in the pointer
                self.ir_b.create_store(gen_arg.clone(), alloca.clone(), typ.clone());

                // Use the pointer as the argument
                arg_values.push_str(format!("{}* {}", typ, alloca).as_str());
            } else {
                // Use the value as the argument
                arg_values.push_str(format!("{} {}", typ, gen_arg).as_str());
            }

            // If the end of the arguments isn't reached, add a comma
            if arg_num + 1 < args.len() {
                arg_values.push_str(", ");
            }

            // Increment the number of arguments
            arg_num += 1;
        }

        match id.clone().as_str() {
            "write" => {
                // New string to hold the formatted parts
                let mut fmt = String::new();

                // Length of the formatted parts
                let mut fmt_len = 1;
                
                // Iterate through the arguments
                
                for arg in args.iter() {
                    // Add the format type to the 'fmt' string
                    let c = match arg.validate() {
                        "int" | "bool" => "%d",
                        "dec" => "%f",
                        "string" => "%s",
                        "char" => "%c",
                        _ => ""
                    };
                    fmt.push_str(format!("{}", c).as_str());
                    
                    // Increment the length by 2 because %d is two characters
                    fmt_len += 2;
                }
                
                // Add the NUL terminator to the string
                fmt.push_str("\\00");

                // Create a global constant for the format
                self.ir_b.create_global(format!("@fmt{}", self.format_num), format!("[{} x i8] c\"{}\"", fmt_len, fmt));

                // If @printf is not declared, declare it
                if !self.dec_printf {
                    self.ir_b.create_ends(format!("declare i32 @printf(i8*, ...)\n"));
                    self.dec_printf = true;
                }

                // Generate the function call
                self.ir_b.code.push_str(format!("\tcall i32 (i8*, ...) @printf(i8* getelementptr inbounds ([{} x i8], [{0} x i8]* @fmt{}, i32 0, i32 0), {})\n", fmt_len, self.format_num, arg_values.clone()).as_str());
                self.format_num += 1;
                self.ir_b.ssa_num += 1;
            },
            "len" => {
                if (*args[0].clone()).validate() == "string" {
                    // Generate the function call
                    self.ir_b.code.push_str(format!("\t%{} = call i32 @strlen({})\n", self.ir_b.ssa_num, arg_values.clone()).as_str());
                    if !self.dec_strlen {
                        self.ir_b.create_ends(format!("declare i32 @strlen(i8*)\n"));
                        self.dec_strlen = true;
                    }
                    self.ir_b.ssa_num += 1;
                } else {
                    let gen_expr = self.generate_expression(*args[0].clone(), false);
                    let gep = self.ir_b.create_gep("%.Arr".to_string(), gen_expr, vec!["0".to_string(), "1".to_string()]);
                    self.ir_b.create_load("i32".to_string(), gep);
                }
            },
            _ => {
                // Generate the function call
                if typ.clone() == "void" {
                    self.ir_b.code.push_str(format!("\tcall void @{}({})\n", id.clone(), arg_values.clone()).as_str());
                } else {
                    self.ir_b.code.push_str(format!("\t%{} = call {} @{}({})\n", self.ir_b.ssa_num, typ.clone(), id.clone(), arg_values.clone()).as_str());
                }
                self.ir_b.ssa_num += 1;
            }
        };
        format!("%{}", self.ir_b.ssa_num - 1)
    }

    /// Generates code for a struct declaration
    fn generate_struct_decl(&mut self, id: String, fields: Vec<Argument>) {
        // Tell the ir builder to create a new struct
        self.ir_b.create_new_struct(id, fields);
    }
}
