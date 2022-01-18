/// An enum to store each kind of symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolType {
    Var,
    Struct,
    //Const,
    //Func
}

/// Stores information for each symbol
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    /// Identifier of the symbol
    pub id: String,

    /// Type of the symbol (i.e., int or string)
    pub typ: String,

    /// Symbol type of the symbol (i.e., var or func)
    pub symtyp: SymbolType,

    /// Stores the id of the symbol in ir for code generation
    pub gen_id: String,

    /// Stores the names and types of the arguments if there are any arguments
    pub arg_types: Vec<(String, String)>
}

/// Stores information for each scope
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Scope {
    /// Parent of scope
    pub parent: Option<Box<Scope>>,

    /// Children of the scope
    pub children: Vec<Scope>,

    /// Symbols of the scope
    pub symbols: Vec<Symbol>
}

/// Stores information about the symbol table
pub struct SymbolController {
    /// The current scope in the symbol table
    pub current: Scope,
}

/// Implement functions for the symbol table
impl SymbolController {
    /// Adds a symbol to the current scope of the symbol table
    pub fn add_symbol(&mut self, id: String, typ: String, symtyp: SymbolType, gen_id: String, arg_types: Vec<(String, String)>) {
        // If the symbol already exists, print an error
        if self.find(id.clone(), symtyp.clone(), Some(arg_types.clone())) != None {
            eprintln!("Identifer '{}' already exists", id);
            std::process::exit(1);
        }

        // Add the new symbol to the current scope
        self.current.symbols.push(Symbol {id: id, typ: typ, symtyp: symtyp, gen_id: gen_id, arg_types: arg_types});
    }

    /*
    /// Adds a scope to the symbol table
    pub fn add_scope(&mut self) {
        self.current.children.push(Scope {parent: Some(Box::new(self.current.clone())), children: Vec::new(), symbols: Vec::new()});
        self.current = self.current.children.last().unwrap().clone();
    }

    /// Pops a scope from the symbol table
    pub fn pop_scope(&mut self) {
        self.current = *self.current.parent.clone().unwrap();
        self.current.children.pop();
    }
    */

    /// Finds a symbol in the current scope
    /// Returns None if it doesn't exist
    pub fn find(&self, id: String, symtyp: SymbolType, arg_types: Option<Vec<(String, String)>>) -> Option<&Symbol> {
        // Loop through the current symbols
        for sym in self.current.symbols.iter() {
            // If the symbol matches, return the symbol
            if arg_types == None {
                if sym.id == id && sym.symtyp == symtyp {
                    return Some(sym);
                }
            } else {
                if sym.id == id && sym.symtyp == symtyp && Some(sym.arg_types.clone()) == arg_types {
                    return Some(sym);
                }
            }
        }

        // The symbol wasn't found, return None
        return None;
    }

    /// Finds a symbol in the current scope
    /// Prints an error if it doesn't exist
    pub fn find_error(&self, id: String, symtyp: SymbolType, arg_types: Option<Vec<(String, String)>>) -> &Symbol {
        // Loop through the current symbols
        for sym in self.current.symbols.iter() {
            println!("{:?}", sym);
            // If the symbol matches, return the symbol
            if arg_types == None {
                if sym.id == id && sym.symtyp == symtyp {
                    return sym;
                }
            } else {
                if sym.id == id && sym.symtyp == symtyp && Some(sym.arg_types.clone()) == arg_types {
                    return sym;
                }
            }
            println!("-----------------");
        }

        // The symbol wasn't found, print an error
        eprintln!("Identifier '{}' not found", id);
        std::process::exit(1);
    }
}
