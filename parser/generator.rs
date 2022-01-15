use super::ast::Node;
use super::ast::Expression;

// Returns the Gizmo type converted to an LLVM ir type
fn type_of(t: String) -> &'static str {
    match t.as_str() {
        "int" => "i32",
        "dec" => "double",
        "bool" => "i1",
        "string" => "i8*",
        _ => ""
    }
}

// Returns the Gizmo operator converted to an LLVM ir instruction
fn type_of_oper(o: String) -> &'static str {
    match o.as_str() {
        "+" => "add",
        "-" => "sub",
        "*" => "mul",
        "/" => "sdiv",
        _ => ""
    }
}

// Stores information for a "Generator"
pub struct Generator {
    pub name_num: usize, // Number of variables created
    pub code: String, // Output code
    pub ends: String, // Will be added to the end of "code" when done generating
    pub begins: String, // Will be added to the beginning of "code" when done generating
}

// Implement functions for a "Generator"
impl Generator {
    // Loops through the ast and calls recursive functions to go deeper
    pub fn gen_all(&mut self, ast: Vec<Node>) {
        for node in ast.iter() {
            match node {
                // Match node for each type of Node
                Node::Let {id: _, expr, gen_id} => self.gen_let_stmt(&expr, gen_id.to_string()),
                Node::FuncCall {id, args} => self.gen_func_call(id.to_string(), args),
                Node::Non => {},
            };
        }
    }

    // Generates LLVM ir for an expression (recursively)
    fn gen_expr(&mut self, expr: &Box<Expression>) -> String {
        // Clone the expression
        // TODO: Change this to use less memory
        let e = (**expr).clone();
        match e {
            // Match the cloned expression against each type of Expression
            Expression::Int(i) => i.to_string(),
            Expression::Dec(d) => d.to_string(),
            Expression::Bool(b) => if b {"1".to_string()} else {"0".to_string()},
            Expression::Str(s) => s,
            Expression::Id(_i, t, _a, gen_id) => {
                let typ = type_of(t);
                self.code.push_str(format!("\t%{} = load {}, {}* {}\n", self.name_num, typ, typ, gen_id).as_str());
                self.name_num += 1;
                format!("%{}", self.name_num - 1)
            },
            // When a binary operator is found, generate ir for it
            Expression::BinaryOperator {oper, left, right} => {
                // Recursively generate ir for the left and right side
                let gen_left = self.gen_expr(&left);
                let gen_right = self.gen_expr(&right);

                // Get the operator converted to an LLVM instruction
                let oper_typ = type_of_oper((&oper).to_string());

                // Add ir to the "code" variable in a "Generator"
                self.code.push_str(format!("\t%{} = {} {} {}, {}\n", self.name_num, oper_typ, type_of(Expression::BinaryOperator {oper, left, right}.validate().to_string()), gen_left, gen_right).as_str());

                // Increment the number of variables
                self.name_num += 1;

                // Return the variable number we just used
                format!("%{}", self.name_num - 1)
            },
            // When a unary operator is found, generate ir for it
            Expression::UnaryOperator {oper, child} => {
                // Recursively generate ir for the child of the operator
                let gen_child = self.gen_expr(&child);
                match oper.as_str() {
                    // Match the operator
                    "-" => {
                        // Generate ir for a negative constant
                        self.code.push_str(format!("\t%{} = mul {} {}, -1\n", self.name_num, type_of(child.validate().to_string()), gen_child).as_str());
                        self.name_num += 1;
                        format!("%{}", self.name_num - 1)
                    },
                    "not" => {
                        // Generate ir for a not statement
                        self.code.push_str(format!("\t%{} = sub i1 1, {}\n", self.name_num, gen_child).as_str());
                        self.name_num += 1;
                        format!("%{}", self.name_num - 1)
                    },
                    _ => "".to_string(),
                }
            },
            Expression::Non => "".to_string(),
        }
    }

    // Generate ir for a let statement
    pub fn gen_let_stmt(&mut self, expr: &Box<Expression>, gen_id: String) {
        // Generate ir for the expression of the let statement
        let gen_expr = self.gen_expr(expr);

        // Store the type of the expression
        let typ = type_of(expr.validate().to_string());
        
        // Add the ir to the code for the "Generator"
        self.code.push_str(format!("\t{} = alloca {}\n", gen_id, typ).as_str());
        self.code.push_str(format!("\tstore {} {}, {0}* {}\n", typ, gen_expr, gen_id).as_str());
    }

    // Generate ir for a function call
    fn gen_func_call(&mut self, id: String, args: &Vec<Box<Expression>>) {
        // Create a vector of tuples to store the types and names of each argument
        let mut arg_names: Vec<(String, String)> = Vec::new();
        // This also provides a chance for the "gen_expr()" function to set up
        // the variables it uses
        for expr in args {
            arg_names.push((type_of(expr.validate().to_string()).to_string(), self.gen_expr(&expr)));
        }

        // Match against built-in functions, so that we can accurately generate
        // the correct ir for it
        if id == "write" {
            self.code.push_str("\tcall i32 (i8*, ...) @printf(i8* getelementptr inbounds (");
            match type_of(args[0].validate().to_string()) {
                "i32" => self.code.push_str("[3 x i8], [3 x i8]* @.int"),
                "double" => self.code.push_str("[3 x i8], [3 x i8]* @.dec"),
                "i1" => self.code.push_str("[3 x i8], [3 x i8]* @.bool"),
                "i8*" => self.code.push_str("[{} x i8], [{} x i8]* @.str"),
                _ => {},
            }
            self.begins.push_str(format!("@.{} = constant [3 x i8] c\"%{}\\00\"\n\n", args[0].validate(), match args[0].validate() {
                    "int" => "d",
                    "dec" => "f",
                    "bool" => "d",
                    "string" => "s",
                    _ => "",
            }).as_str());
            self.ends.push_str("declare i32 @printf(i8*, ...)\n");
            self.code.push_str(", i32 0, i32 0), ");
        } else {
            self.code.push_str(format!("\tcall void @{}(", id).as_str());
        }

        // Loop through the arguments and add them too
        for name in arg_names {
            self.code.push_str(format!("{} {}", name.0, name.1).as_str());
        }
        self.code.push_str(")\n");
    }
}
