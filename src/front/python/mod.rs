//! The Python front-end

pub mod parser;
pub mod term;
pub mod ty;

use crate::ir::term::{Computations};
use super::{FrontEnd};
use std::path::PathBuf;
use crate::ir::term::{bv_lit};
use parser::PythonParser;
use python_parser::ast::{CompoundStatement, Funcdef, Statement, Expression, IntegerType};
use term::{PyTerm, PyTermData, Pyt};
use std::fs;
use std::collections::HashMap;
use crate::circify::{CircError, Circify};
use std::fmt::Display;
use std::str::FromStr;
use std::cell::RefCell;
use crate::front::python::ty::Ty;

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
        // TODO error handling
        let code: String = fs::read_to_string(i.file).unwrap();
        let ast: Vec<Statement>  = PythonParser::parse_file(&code);
        let mut pygen = PyGen::new(ast);
        pygen.entry_fn("main");
        let mut cs = Computations::new();
        //println!("{:?}", pygen);
        //comp.outputs.push(leaf_term(Op::Const(Value::Bool(false))));
        let main_comp = pygen.circify().consume().borrow().clone();
        println!("main: {:?}", main_comp);
        cs.comps.insert("main".to_string(), main_comp);
        cs
    }
}

#[derive(Debug)]
struct PyGen {
    circ: RefCell<Circify<Pyt>>,
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
        Self{
            circ: RefCell::new(Circify::new(Pyt::new())),
            functions: functions}
    }

    fn entry_fn(&mut self, name: &str) {
        // TODO FINISH SETUP FOR ENTRY
        let func = self.functions
            .get(name)
            .unwrap_or_else(|| panic!("Code does not have main function"))
            .clone();
        
        self.circ_enter_fn(name.to_owned(), Some(Ty::Int(32)));
        // TODO handle more than one statement
        self.gen_stmt(&func.code[0]);
    }

    fn gen_stmt(&mut self, stmt: &Statement){
        match stmt {
            Statement::Return(ret) => {
                let ret: PyTerm = self.gen_expr(&ret[0]);
                let ret_res = self.circ_return_(Some(ret));
                self.unwrap(ret_res);
            }
            _ => unimplemented!("Statement {:#?} hasn't been implemented", stmt)
        }
    }

    fn gen_expr(&mut self, expr: &Expression) -> PyTerm {
        match expr {
            Expression::Int(int) => {
                self.integer(int)
            }
            _ => unimplemented!("Expr {:#?} hasn't been implemented", expr)
        }
    }

    fn integer(&self, int: &IntegerType) -> PyTerm {
        let radix:u32 = 10;
        // TODO handling int size?
        let size = 32;
        let int_str: String = int.to_str_radix(radix);
        let num = i32::from_str(&int_str).unwrap();
        PyTerm{term: PyTermData::Int(size, bv_lit(num, size))}
    }
    
    fn circify(&self) -> Circify<Pyt> {
        self.circ.replace(Circify::new(Pyt::new()))
    }

    fn circ_return_(&self, ret: Option<PyTerm>) -> Result<(), CircError> {
        self.circ.borrow_mut().return_(ret)
    }
    
    fn circ_enter_fn(&self, f_name: String, ret_ty: Option<Ty>) {
        self.circ.borrow_mut().enter_fn(f_name, ret_ty)
    }

    /// Unwrap a result of an error and abort
    fn err<E: Display>(&self, e: E) -> ! {
        println!("Error: {e}");
        std::process::exit(1)
    }

    /// Unwrap result of a computation
    /// TODO: Add span for debugging
    fn unwrap<PyTerm, E: Display>(&self, r: Result<PyTerm, E>) -> PyTerm {
        r.unwrap_or_else(|e| self.err(e))
    }
}
