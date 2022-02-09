use super::lexer::error::error;
use super::lexer::error::ErrorType;
use super::lexer::token::Token;

/// An enum to store each kind of symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolType {
    Var,
    Struct,
    Func
}

/// Stores information for each variable symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VarSymbol {
    /// Identifier of the symbol
    pub id: String,

    /// Type of the symbol (i.e., int or string)
    pub typ: String,

    /// Stores the id of the symbol in ir for code generation
    pub gen_id: String,
}

/// Stores information for each function symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FuncSymbol {
    /// Identifier of the symbol
    pub id: String,

    /// Type of the symbol (i.e., int or string)
    pub typ: String,

    /// Stores the id of the symbol in ir for code generation
    pub gen_id: String,

    /// Stores the types of the arguments
    pub arg_types: Vec<String>
}

/// Stores information for each struct symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructSymbol {
    /// Identifier of the symbol
    pub id: String,

    /// Stores the id of the symbol in ir for code generation
    pub gen_id: String,

    /// Stores the types of the arguments
    pub arg_types: Vec<String>
}

/// Stores information for each scope
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Scope {
    /// Parent of scope
    pub parent: Option<Box<Scope>>,

    /// Children of the scope
    pub children: Vec<Scope>,

    /// Symbols of the scope
    pub var_symbols:    Vec<VarSymbol>,
    pub func_symbols:   Vec<FuncSymbol>,
    pub struct_symbols: Vec<StructSymbol>
}

/// Stores information about the symbol table
pub struct SymbolController {
    /// The current scope in the symbol table
    pub current: Scope,
}

/// Implement functions for the symbol table
impl SymbolController {
    /// Adds a symbol to the current scope of the symbol table
    pub fn add_symbol(&mut self, id: String, typ: String, symtyp: SymbolType, gen_id: String, arg_types: Option<Vec<String>>) {
        // If the symbol already exists, print an error
        if self.find(id.clone(), symtyp.clone()) != false {
            eprintln!("Identifer '{}' already exists", id);
            std::process::exit(1);
        }

        // Add the new symbol to the current scope
        match symtyp.clone() {
            SymbolType::Var    => self.current.var_symbols.push(VarSymbol {id: id, typ: typ, gen_id: gen_id}),
            SymbolType::Func   => self.current.func_symbols.push(FuncSymbol {id: id, typ: typ, gen_id: gen_id, arg_types: arg_types.unwrap_or(Vec::new())}),
            SymbolType::Struct => self.current.struct_symbols.push(StructSymbol {id: id, gen_id: gen_id, arg_types: arg_types.unwrap_or(Vec::new())}),
        }
    }

    /// Adds a scope to the symbol table
    pub fn add_scope(&mut self) {
        let new = Scope {parent: Some(Box::new(self.current.clone())), children: Vec::new(), var_symbols: Vec::new(), func_symbols: Vec::new(), struct_symbols: Vec::new()};

        // Make the new scope the current scopes child
        self.current.children.push(new.clone());

        // Set the current to the new scope
        self.current = new.clone();
    }

    /// Pops a scope from the symbol table
    pub fn pop_scope(&mut self) {
        // Set the current to the parent of the current
        self.current = *self.current.parent.as_ref().unwrap().clone();

        // Pop the scope from the children
        self.current.children.pop();
    }

    /// Finds a symbol in the current scope
    /// Returns None if it doesn't exist
    pub fn find(&self, id: String, symtyp: SymbolType) -> bool {
        // Loop through the current symbols
        match symtyp {
            SymbolType::Var => {
                for sym in self.current.var_symbols.iter() {
                    if sym.id == id {
                        return true;
                    }
                }
            },
            SymbolType::Func => {
                for sym in self.current.func_symbols.iter() {
                    if sym.id == id {
                        return true;
                    }
                }
            },
            SymbolType::Struct => {
                for sym in self.current.struct_symbols.iter() {
                    if sym.id == id {
                        return true;
                    }
                }
            },
        };

        // The symbol wasn't found, return false
        return false;
    }

    /// Finds a variable identifier in the global scope
    /// Returns None if it doesn't exist
    pub fn find_global_var(&self, id: String) -> Option<VarSymbol> {
        // Loop through the current symbols
        let mut current: Option<Box<Scope>> = Some(Box::new(self.current.clone()));
        while current != None {
            let cur = *(current.clone().unwrap());
            for sym in cur.var_symbols.iter() {
                // If the symbol matches, return the symbol
                if sym.id == id {
                    return Some(sym.clone());
                }
            }
            current = current.clone().unwrap().parent.clone();
        }

        // The symbol wasn't found, return None
        return None;
    }

    /// Finds a function identifier in the global scope
    /// Returns None if it doesn't exist
    pub fn find_global_func(&self, id: String) -> Option<FuncSymbol> {
        // Loop through the current symbols
        let mut current: Option<Box<Scope>> = Some(Box::new(self.current.clone()));
        while current != None {
            let cur = *(current.clone().unwrap());
            for sym in cur.func_symbols.iter() {
                // If the symbol matches, return the symbol
                if sym.id == id {
                    return Some(sym.clone());
                }
            }
            current = current.clone().unwrap().parent.clone();
        }

        // The symbol wasn't found, return None
        return None;
    }

    /// Finds a struct identifier in the global scope
    /// Returns None if it doesn't exist
    pub fn find_global_struct(&self, id: String) -> Option<StructSymbol> {
        // Loop through the current symbols
        let mut current: Option<Box<Scope>> = Some(Box::new(self.current.clone()));
        while current != None {
            let cur = *(current.clone().unwrap());
            for sym in cur.struct_symbols.iter() {
                // If the symbol matches, return the symbol
                if sym.id == id {
                    return Some(sym.clone());
                }
            }
            current = current.clone().unwrap().parent.clone();
        }

        // The symbol wasn't found, return None
        return None;
    }

    /// Finds a variable identifier in the global scope
    /// Prints an error if it doesn't exist
    pub fn find_global_var_error(&self, id: String, token: &Token) -> VarSymbol {
        let sym = self.find_global_var(id.clone());
        if sym == None {
            // If the symbol isn't found, print an error
            let mut cur_var_ids: Vec<String> = Vec::new();
            for symbol in self.current.var_symbols.clone() {
                cur_var_ids.push(symbol.id.clone());
            }
            let helper = if autoc(id.clone(), cur_var_ids.clone()) == id.clone() {
                "Perhaps you spelled this identifier wrong".to_string()
            } else {
                format!("Perhaps you meant '{}'", autoc(id.clone(), cur_var_ids))
            };
            error(ErrorType::UndefinedSymbol, token)
                .note(format!("Undefined symbol '{}'", id).as_str())
                .help(helper.as_str())
                .emit();
            std::process::exit(1);
        } else {
            return sym.unwrap();
        }
    }

    /// Finds a function identifier in the global scope
    /// Prints an error if it doesn't exist
    pub fn find_global_func_error(&self, id: String, token: &Token) -> FuncSymbol {
        let sym = self.find_global_func(id.clone());
        if sym == None {
            // If the symbol isn't found, print an error
            let mut cur_func_ids: Vec<String> = Vec::new();
            for symbol in self.current.func_symbols.clone() {
                cur_func_ids.push(symbol.id.clone());
            }
            let helper = if autoc(id.clone(), cur_func_ids.clone()) == id.clone() {
                "Perhaps you spelled this identifier wrong".to_string()
            } else {
                format!("Perhaps you meant '{}'", autoc(id.clone(), cur_func_ids))
            };
            error(ErrorType::UndefinedSymbol, token)
                .note(format!("Undefined symbol '{}'", id).as_str())
                .help(helper.as_str())
                .emit();
            std::process::exit(1);
        } else {
            return sym.unwrap();
        }
    }

    /// Finds a struct symbol in the global scope
    /// Prints an error if it doesn't exist
    pub fn find_global_struct_error(&self, id: String, token: &Token) -> StructSymbol {
        let sym = self.find_global_struct(id.clone());
        if sym == None {
            // If the symbol isn't found, print an error
            let mut cur_struct_ids: Vec<String> = Vec::new();
            for symbol in self.current.struct_symbols.clone() {
                cur_struct_ids.push(symbol.id.clone());
            }
            let helper = if autoc(id.clone(), cur_struct_ids.clone()) == id.clone() {
                "Perhaps you spelled this identifier wrong".to_string()
            } else {
                format!("Perhaps you meant '{}'", autoc(id.clone(), cur_struct_ids))
            };
            error(ErrorType::UndefinedSymbol, token)
                .note(format!("'{}' is undefined", id).as_str())
                .help(helper.as_str())
                .emit();
            std::process::exit(1);
        } else {
            return sym.unwrap();
        }
    }
}

fn similarity(word: String, word2: String) -> f32 {
    let mut similar: Vec<char> = Vec::new();
    for c in word.chars() {
        if word2.contains(c) {
            similar.push(c);
        }
    }

    return similar.len() as f32 / std::cmp::max(word.len(), word2.len()) as f32;
}

fn autoc(word: String, names: Vec<String>) -> String {
    let mut max_sim = 0.0;
    let mut most_sim = word.clone();

    for name in names.iter() {
        let sim = similarity(name.clone(), word.clone());
        if sim > max_sim {
            max_sim = sim;
            most_sim = name.clone();
        }
    }

    return most_sim;
}
