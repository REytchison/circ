//! The Python front-end

pub mod parser;

use crate::ir::term::{Computations, Computation};
use super::{FrontEnd};
use std::path::PathBuf;
use crate::ir::term::{Op, leaf_term, Value};
use crate::front::python::parser::PythonParser;
use python_parser::ast::{CompoundStatement, Funcdef, Statement};
use std::fs;
use std::collections::HashMap;


/// Inputs to Python compiler
pub struct Inputs {
    /// The file to look for `main` in.
    pub file: PathBuf
    // TODO MAYBE INCLUDE FIELD?
    // /// The mode to generate for (MPC or proof). Effects visibility.
    //pub mode: Mode
}

/// The Python front-end. Implements [FrontEnd]
pub struct Python;


impl FrontEnd for Python{
    type Inputs = Inputs;
    fn gen(i: Self::Inputs) -> Computations{
        let mut cs = Computations::new();
        let mut comp = Computation::new();
        // TODO error handling
        let code: String = fs::read_to_string(i.file).unwrap();
        let ast: Vec<Statement>  = PythonParser::parse_file(&code);
        let pygen = PyGen::new(ast);
        pygen.entry_fn("main");
        println!("{:?}", pygen);
        comp.outputs.push(leaf_term(Op::Const(Value::Bool(false))));
        cs.comps.insert("main".to_string(), comp);
        cs
    }
}

#[derive(Debug)]
struct PyGen {
    functions: HashMap<String, Funcdef>
}

impl PyGen {
    fn new(ast: Vec<Statement>) -> Self {
        let mut functions = HashMap::new();
        for stmt in ast{
            // TODO BETTER ERROR HANDLING
            match stmt {
                Statement::Compound(stmtbox) => {
                    let compound_stmt = *stmtbox;
                    if let CompoundStatement::Funcdef(funcdef) = compound_stmt{
                        functions.insert(funcdef.name.to_string(), funcdef);
                    } else {
                        panic!("Code is not only functions.")
                    }
                    //let CompoundStatement::Funcdef(funcdef)>
                },
                _ => panic!("Code is not only functions.")
            }
        }
        Self{functions}
    }

    fn entry_fn(&self, name: &str) {
        // TODO FINISH SETUP FOR ENTRY
        if self.functions.get(name).is_none(){
            panic!("Code does not have main function")
        }
    }
}