#[derive(Debug, PartialEq, Eq)]
pub struct Symbol {
    pub id: String,
    pub typ: String,
    pub arg_types: Vec<String>,
    pub sym_typ: String,
    pub gen_id: String,
    pub next: Option<Box<Symbol>>
}

#[derive(Debug, PartialEq, Eq)]
pub struct SymbolTable {
    pub parent: Option<Box<SymbolTable>>,
    pub child: Option<Box<SymbolTable>>,
    pub group: Vec<Symbol>
}

pub struct SymbolController {
    pub global: SymbolTable,
    pub current: SymbolTable,
}

impl SymbolController {
    pub fn add_symbol(&mut self, id: String, typ: String, sym_typ: String, arg_types: Vec<String>, gen_id: String) {
        self.current.group.push(Symbol {id: id, typ: typ, sym_typ: sym_typ, arg_types: arg_types, gen_id: gen_id, next: None});
    }

    pub fn add_global_symbol(&mut self, id: String, typ: String, sym_typ: String, arg_types: Vec<String>, gen_id: String) {
        self.global.group.push(Symbol {id: id, typ: typ, sym_typ: sym_typ, arg_types: arg_types, gen_id: gen_id, next: None});
    }

    pub fn find_symbol(&mut self, id: String, sym_typ: String, arg_types: Vec<String>) -> Option<Symbol> {
        for sym in self.current.group.iter() {
            if sym.id == id && sym.sym_typ == sym_typ && sym.arg_types == arg_types {
                return Some(Symbol {id: id, typ: sym.typ.clone(), sym_typ: sym_typ, gen_id: sym.gen_id.clone(), arg_types: arg_types, next: None});
            }
        }
        return None;
    }

    pub fn find_global_symbol(&mut self, id: String, sym_typ: String, arg_types: Vec<String>) -> Option<Symbol> {
        for sym in self.global.group.iter() {
            if sym.id == id && sym.sym_typ == sym_typ && sym.arg_types == arg_types {
                return Some(Symbol {id: id, typ: sym.typ.clone(), sym_typ: sym_typ, gen_id: sym.gen_id.clone(), arg_types: arg_types, next: None});
            }
        }
        return None;
    }

    /*pub fn pretty_print(&self) {
        for symtab in self.global.group.iter() {
            symtab.pretty_print();
        }
    }*/
}
