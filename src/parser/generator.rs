use super::ast::Node;
use super::ast::Expression;

fn type_of(typ: String) -> String {
    let struct_type = format!("%{}", typ);
    match typ.as_str() {
        "int" => "i32",
        "dec" => "double",
        "bool" => "i1",
        "char" => "i8",
        "string" => "i8*",
        arr if arr.ends_with(']') => "%.Arr",
        _ => struct_type.as_str()
    }.to_string()
}

fn type_of_oper(oper: String) -> String {
    match oper.as_str() {
        "+" => "add",
        "-" => "sub",
        "*" => "mul",
        "/" => "sdiv",
        "==" => "icmp eq",
        "!=" => "icmp ne",
        "<" => "icmp slt",
        ">" => "icmp sgt",
        "<=" => "icmp sle",
        ">=" => "icmp sge",
        _ => ""
    }.to_string()
}

pub struct IRBuilder {
    pub code: String,
    pub ends: String,
    pub ssa_num: i32,
    pub save: i32,
    pub tmp_num: i32,
    pub str_num: i32,
    pub label_num: i32,
}

impl IRBuilder {
    fn construct() -> IRBuilder {
        IRBuilder {code: "define i32 @main() {\nentry:\n".to_string(), ends: "\tret i32 0\n}\n".to_string(), ssa_num: 0, tmp_num: 0, str_num: 0, label_num: 0, save: 0}
    }

    fn create_alloca(&mut self, typ: String, name: Option<String>) -> String {
        self.code.push_str(format!("\t{} = alloca {}\n", name.clone().unwrap_or(format!("%{}", self.ssa_num)), typ).as_str());
        if name == None {
            self.ssa_num += 1;
        }
        format!("{}", name.clone().unwrap_or(format!("%{}", self.ssa_num - 1)))
    }
    
    fn create_store(&mut self, src: String, dst: String, typ: String) {
        self.code.push_str(format!("\tstore {} {}, {0}* {}\n", typ, src, dst).as_str());
    }
    
    fn create_load(&mut self, typ: String, src: String) -> String {
        self.code.push_str(format!("\t%{} = load {}, {1}* {}\n", self.ssa_num, typ, src).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }
    
    fn create_bitcast(&mut self, typ: String, name: String, typ2: String) -> String {
        self.code.push_str(format!("\t%{} = bitcast {} {} to {}\n", self.ssa_num, typ, name, typ2).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }

    fn create_gep(&mut self, typ: String, name: String, indices: Vec<String>) -> String {
        self.code.push_str(format!("\t%{} = getelementptr inbounds {}, {1}* {}", self.ssa_num, typ, name).as_str());
        self.ssa_num += 1;
        for indice in indices {
            self.code.push_str(format!(", i32 {}", indice).as_str());
        }
        self.code.push('\n');
        format!("%{}", self.ssa_num - 1)
    }

    fn create_global(&mut self, id: String, value: String) -> String {
        self.code = format!("{} = constant {}\n\n{}", id, value, self.code);
        format!("{}", id)
    }

    fn create_ends(&mut self, s: String) {
        self.ends.push_str(s.as_str());
    }

    fn enter_function(&mut self) {
        self.save = self.ssa_num;
        self.ssa_num = 0;
    }

    fn exit_function(&mut self) {
        self.ssa_num = self.save;
        self.save = 0;
    }

    fn create_new_struct(&mut self, id: String, fields: Vec<(String, String)>) -> String {
        let mut cat_fields = String::new();
        let mut num_fields = 0;
        for field in fields.clone().iter() {
            cat_fields.push_str(format!("\t{}", type_of(field.clone().1)).as_str());
            if num_fields + 1 < fields.clone().len() {
                cat_fields.push(',');
            }
            cat_fields.push('\n');
            num_fields += 1;
        }
        self.code = format!("%{} = type {{\n{}}}\n\n{}", id, cat_fields, self.code);
        format!("@{}", id)
    }

    fn create_operation(&mut self, oper: String, typ: String, left: String, right: String) -> String {
        self.code.push_str(format!("\t%{} = {} {} {}, {}\n", self.ssa_num, type_of_oper(oper), type_of(typ), left, right).as_str());
        self.ssa_num += 1;
        format!("%{}", self.ssa_num - 1)
    }
}

pub struct Generator {
    pub ir_b: IRBuilder,
    pub has_array: bool,
    pub dec_printf: bool,
    pub dec_int: bool,
    pub dec_dec: bool,
    pub dec_char: bool,
    pub dec_str: bool,
}

impl Generator {
    pub fn construct() -> Generator {
        Generator {ir_b: IRBuilder::construct(), has_array: false, dec_printf: false, dec_int: false, dec_dec: false, dec_char: false, dec_str: false}
    }

    pub fn destruct(&mut self) {
        self.ir_b.code.push_str(self.ir_b.ends.as_str());
    }

    pub fn generate(&mut self, nodes: Vec<Box<Node>>) {
        for node in nodes.iter() {
            match *node.clone() {
                Node::Let {id: _, expr, gen_id} => self.generate_let_stmt(expr.clone(), gen_id.clone()),
                Node::FuncDecl {id, args, body} => self.generate_func_decl(id.clone(), args.clone(), body.clone()),
                Node::While {cond, body} => self.generate_while_loop(cond.clone(), body.clone()),
                Node::Assign {id, expr} => self.generate_assign_stmt(id.clone(), expr.clone()),
                Node::FuncCall {id, args} => self.generate_func_call(id.clone(), args.clone()),
                Node::Struct {id, fields} => self.generate_struct_def(id.clone(), fields.clone()),
                Node::Block {statements} => {
                    self.generate(statements);
                }
                _ => {}
            }
        }
    }

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

    pub fn generate_expression(&mut self, expr: Expression, load_id: bool) -> String {
        match expr.clone() {
            Expression::Int(i) => i.to_string(),
            Expression::Chr(c) => (c as i32).to_string(),
            Expression::Dec(d) => d.to_string(),
            Expression::Bool(b) => b.to_string(),
            Expression::Str(s) => {
                let (length, rest) = self.get_str_length(s.clone());
                let global = self.ir_b.create_global(format!("@.str.{}", self.ir_b.str_num), format!("[{} x i8] c\"{}\\00\"", length, rest));
                self.ir_b.str_num += 1;
                self.ir_b.create_gep(format!("[{} x i8]", length), global, vec!["0".to_string(), "0".to_string()])
            },
            Expression::Id(_id, typ, gen_id) => {
                if load_id == true {
                    self.ir_b.create_load(type_of(typ), gen_id)
                } else {
                    gen_id
                }
            }
            Expression::NewStruct {id, fields} => {
                let begin = self.ir_b.create_alloca(type_of(id.clone()), None);
                let mut field_num = 0;
                for field in fields.iter() {
                    let gen_field = self.generate_expression(field.clone(), true);
                    let gep = self.ir_b.create_gep(type_of(id.clone()), begin.clone(), vec!["0".to_string(), field_num.to_string()]);
                    self.ir_b.create_store(gen_field, gep, type_of(field.validate().to_string()));
                    field_num += 1;
                }
                self.ir_b.create_load(type_of(id), begin)
            }
            Expression::StructDot {id, typ, field_num, ..} => {
                let gen_begin = self.generate_expression(*id.clone(), false);
                let gep = self.ir_b.create_gep(type_of(id.validate().to_string()), gen_begin, vec!["0".to_string(), field_num.to_string()]);
                if load_id == true {
                    self.ir_b.create_load(type_of(typ.clone()), gep)
                } else {
                    gep
                }
            }
            Expression::Array {values, ..} => {
                if !self.has_array {
                    self.ir_b.create_new_struct(".Arr".to_string(), vec![("".to_string(), "string".to_string()), ("".to_string(), "int".to_string())]);
                    self.has_array = true;
                }
                let v = values[0].clone();
                let v_typ = type_of(v.validate().to_string());
                let alloca = self.ir_b.create_alloca("%.Arr".to_string(), None);
                let sized_alloca = self.ir_b.create_alloca(format!("[{} x {}]", values.len(), v_typ), None);
                let mut value_num = 0;
                for value in values.iter() {
                    let gen_value = self.generate_expression((*value).clone(), true);
                    let gep = self.ir_b.create_gep(format!("[{} x {}]", values.len(), v_typ), sized_alloca.clone(), vec!["0".to_string(), value_num.to_string()]);
                    value_num += 1;
                    self.ir_b.create_store(gen_value, gep, type_of(value.clone().validate().to_string()));
                }
                let bitcast = self.ir_b.create_bitcast(format!("[{} x {}]*", values.len(), v_typ), sized_alloca, "i8*".to_string());
                let gep2 = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "0".to_string()]);
                self.ir_b.create_store(bitcast, gep2, "i8*".to_string());
                let gep3 = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "1".to_string()]);
                self.ir_b.create_store(values.len().to_string(), gep3, "i32".to_string());
                self.ir_b.create_load("%.Arr".to_string(), alloca)
            }
            Expression::IndexedValue {src, index, new_typ} => {
                let gen_src = self.generate_expression(*src.clone(), true);
                let gen_index = self.generate_expression(*index.clone(), true);
                match src.validate() {
                    "string" => {
                        let bitcast = self.ir_b.create_bitcast("i8*".to_string(), gen_src, "[0 x i8]*".to_string());
                        let gep = self.ir_b.create_gep("[0 x i8]".to_string(), bitcast, vec!["0".to_string(), gen_index]);
                        self.ir_b.create_load("i8".to_string(), gep)
                    }
                    _ => {
                        let alloca = self.ir_b.create_alloca("%.Arr".to_string(), None);
                        self.ir_b.create_store(gen_src, alloca.clone(), "%.Arr".to_string());
                        let gep = self.ir_b.create_gep("%.Arr".to_string(), alloca.clone(), vec!["0".to_string(), "0".to_string()]);
                        let load = self.ir_b.create_load("i8*".to_string(), gep);
                        let bitcast = self.ir_b.create_bitcast("i8*".to_string(), load, format!("[0 x {}]*", type_of(new_typ.clone())));
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
                let gen_left = self.generate_expression((*left).clone(), true);
                let gen_right = self.generate_expression((*right).clone(), true);
                self.ir_b.create_operation(oper, left.clone().validate().to_string(), gen_left, gen_right)
            }
            Expression::UnaryOperator {oper, child} => {
                let gen_child = self.generate_expression((*child).clone(), true);
                if oper == "-".to_string() {
                    return self.ir_b.create_operation("*".to_string(), child.clone().validate().to_string(), gen_child.clone(), "-1".to_string());
                } else if oper == "++".to_string() {
                    let oper = self.ir_b.create_operation("+".to_string(), "i32".to_string(), gen_child.clone(), "1".to_string());
                    self.ir_b.create_store(oper, gen_child.clone(), "i32".to_string());
                    gen_child
                } else {
                    return self.ir_b.create_operation("-".to_string(), child.clone().validate().to_string(), gen_child, "1".to_string());
                }
            }
            _ => "".to_string()
        }
    }

    fn generate_let_stmt(&mut self, expr: Expression, gen_id: String) {
        let gen_expr = self.generate_expression(expr.clone(), true);
        let var = self.ir_b.create_alloca(type_of(expr.clone().validate().to_string()), Some(gen_id));
        self.ir_b.create_store(gen_expr, var, type_of(expr.clone().validate().to_string()));
    }

    fn generate_func_decl(&mut self, id: String, args: Vec<(String, String)>, body: Box<Node>) {
        let save = self.ir_b.code.clone();
        let mut arg_code = String::new();
        let mut arg_num = 0;
        for arg in args.iter() {
            arg_code.push_str(format!("{}* {}", type_of(arg.0.clone()), arg.1).as_str());
            if arg_num + 1 < args.len() {
                arg_code.push_str(", ");
            }
            arg_num += 1;
        }
        self.ir_b.code = format!("define void @{}({}) {{\nentry:\n", id, arg_code);
        self.ir_b.enter_function();
        self.generate(vec![body]);
        self.ir_b.exit_function();
        self.ir_b.code.push_str("\tret void\n}\n");
        self.ir_b.code = format!("{}\n{}", self.ir_b.code, save.clone());
    }

    fn generate_while_loop(&mut self, cond: Expression, body: Box<Node>) {
        let gen_cond = self.generate_expression(cond.clone(), true);
        self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond, self.ir_b.label_num, self.ir_b.label_num + 1, self.ir_b.label_num).as_str());
        self.generate(vec![body]);
        let gen_cond2 = self.generate_expression(cond.clone(), true);
        self.ir_b.code.push_str(format!("\tbr i1 {}, label %l{}, label %l{}\nl{}:\n", gen_cond2, self.ir_b.label_num, self.ir_b.label_num + 1, self.ir_b.label_num + 1).as_str());
        self.ir_b.label_num += 2;
    }

    fn generate_assign_stmt(&mut self, id: Expression, expr: Expression) {
        let gen_expr = self.generate_expression(expr.clone(), true);
        let gen_id = self.generate_expression(id, false);
        self.ir_b.create_store(gen_expr, gen_id, type_of(expr.clone().validate().to_string()));
    }

    fn generate_func_call(&mut self, id: String, args: Vec<Box<Expression>>) {
        let mut arg_num = 0;
        let mut arg_values = String::new();
        for arg in args.iter() {
            let gen_arg = self.generate_expression(*arg.clone(), true);
            let typ = type_of((*arg.clone().validate()).to_string());
            if id.clone().as_str() != "write" {
                let alloca = self.ir_b.create_alloca(typ.clone(), None);
                self.ir_b.create_store(gen_arg.clone(), alloca.clone(), typ.clone());
                arg_values.push_str(format!("{}* {}", typ, alloca).as_str());
            } else {
                arg_values.push_str(format!("{} {}", typ, gen_arg).as_str());
            }
            if arg_num + 1 < args.len() {
                arg_values.push(',');
            }
            arg_num += 1;
        }
        self.ir_b.ssa_num += 1;
        match id.clone().as_str() {
            "write" => {
                match (*args[0].clone()).validate() {
                    "int" | "bool" => {
                        if self.dec_int == false {
                            self.ir_b.create_global(format!("@.{}", (*args[0].clone()).validate()), format!("[3 x i8] c\"%d\\00\""));
                            self.dec_int = true;
                        }
                    },
                    "dec" => {
                        if self.dec_dec == false {
                            self.ir_b.create_global(format!("@.{}", (*args[0].clone()).validate()), format!("[3 x i8] c\"%f\\00\""));
                            self.dec_dec = true;
                        }
                    },
                    "string" => {
                        if self.dec_str == false {
                            self.ir_b.create_global(format!("@.{}", (*args[0].clone()).validate()), format!("[3 x i8] c\"%s\\00\""));
                            self.dec_str = true;
                        }
                    },
                    "char" => {
                        if self.dec_char == false {
                            self.ir_b.create_global(format!("@.{}", (*args[0].clone()).validate()), format!("[3 x i8] c\"%c\\00\""));
                            self.dec_char = true;
                        }
                    },
                    _ => {}
                };
                if !self.dec_printf {
                    self.ir_b.create_ends(format!("declare i32 @printf(i8*, ...)\n"));
                    self.dec_printf = true;
                }
                self.ir_b.code.push_str(format!("\tcall i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.{}, i32 0, i32 0), {})\n", (*args[0]).clone().validate(), arg_values.clone()).as_str());
            },
            _ => {
                self.ir_b.code.push_str(format!("\tcall void @{}({})\n", id.clone(), arg_values.clone()).as_str());
            }
        };
    }

    fn generate_struct_def(&mut self, id: String, fields: Vec<(String, String)>) {
        self.ir_b.create_new_struct(id, fields);
    }
}
