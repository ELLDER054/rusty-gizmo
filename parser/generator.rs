use super::ast::Node;
use super::ast::Expression;

fn type_of(t: String) -> &'static str {
    match t.as_str() {
        "int" => "i32",
        "dec" => "double",
        "bool" => "i1",
        "string" => "i8*",
        _ => ""
    }
}

fn type_of_oper(o: String) -> &'static str {
    match o.as_str() {
        "+" => "add",
        "-" => "sub",
        "*" => "mul",
        "/" => "sdiv",
        _ => ""
    }
}

pub struct Generator {
    pub name_num: usize,
    pub code: String,
    pub ends: String,
    pub begins: String,
}

impl Generator {
    pub fn gen_all(&mut self, ast: Vec<Node>) {
        for node in ast.iter() {
            match node {
                Node::Let {id: _, expr} => self.gen_let_stmt(&expr),
                Node::FuncCall {id, args} => self.gen_func_call(id.to_string(), args),
                Node::Non => {},
            };
        }
    }

    fn gen_expr(&mut self, expr: &Box<Expression>) -> String {
        let e = (**expr).clone();
        match e {
            Expression::Int(i) => format!("{}", i),
            Expression::Dec(d) => format!("{}", d),
            Expression::Bool(b) => if b {"1".to_string()} else {"0".to_string()},
            Expression::Str(s) => format!("{}", s),
            Expression::BinaryOperator {oper, left, right} => {
                let gen_left = self.gen_expr(&left);
                let gen_right = self.gen_expr(&right);
                let oper_typ = type_of_oper((&oper).to_string());
                self.code.push_str(format!("\t%{} = {} {} {}, {}\n", self.name_num, oper_typ, type_of(Expression::BinaryOperator {oper, left, right}.validate().to_string()), gen_left, gen_right).as_str());
                self.name_num += 1;
                format!("%{}", self.name_num - 1)
            },
            Expression::UnaryOperator {oper, child} => {
                let gen_child = self.gen_expr(&child);
                match oper.as_str() {
                    "-" => {
                        self.code.push_str(format!("\t%{} = mul {} {}, -1\n", self.name_num, type_of(child.validate().to_string()), gen_child).as_str());
                        self.name_num += 1;
                        format!("%{}", self.name_num - 1)
                    },
                    "not" => {
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

    pub fn gen_let_stmt(&mut self, expr: &Box<Expression>) {
        let gen_expr = self.gen_expr(expr);
        let typ = type_of(expr.validate().to_string());
        self.code.push_str(format!("\t%{} = alloca {}\n", self.name_num, typ).as_str());
        self.code.push_str(format!("\tstore {} {}, {0}* %{}\n", typ, gen_expr, self.name_num).as_str());
    }

    fn gen_func_call(&mut self, id: String, args: &Vec<Box<Expression>>) {
        let mut arg_names: Vec<(String, String)> = Vec::new();
        for expr in args {
            arg_names.push((type_of(expr.validate().to_string()).to_string(), self.gen_expr(&expr)));
        }
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
                }
            ).as_str());
            self.code.push_str(", i32 0, i32 0), ");
        } else {
            self.code.push_str(format!("\tcall void @{}(", id).as_str());
        }
        for name in arg_names {
            self.code.push_str(format!("{} {}", name.0, name.1).as_str());
        }
        self.code.push_str(")\n");
        self.ends.push_str("declare i32 @printf(i8*, ...)\n");
    }
}
