//! The Python front-end

pub mod parser;
pub mod term;
pub mod ty;

use crate::ir::term::{Computations, PartyId};
use crate::front::PUBLIC_VIS;
use super::{FrontEnd, Mode};
use std::path::PathBuf;
use crate::ir::term::{bv_lit, term, NOT, AND, OR, Term, Sort, check, bool_lit};
use parser::PythonParser;
use python_parser::ast::{CompoundStatement, Funcdef, Statement, Expression, IntegerType, Argument};
use term::{PyTerm, PyTermData, Pyt, cast_to_bool};
use std::fs;
use std::collections::HashMap;
use crate::circify::{CircError, Circify, Val, Loc};
use std::fmt::Display;
use std::str::FromStr;
use std::cell::RefCell;
use crate::front::python::ty::Ty;

/// Inputs to Python compiler
pub struct Inputs {
    /// The file to look for `main` in.
    pub file: PathBuf,
    /// enable SV competition builtin functions
    pub sv_functions: bool
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
        let code: String = fs::read_to_string(&i.file).unwrap();
        let ast: Vec<Statement>  = PythonParser::parse_file(&code);
        let mut pygen = PyGen::new(i, ast);
        pygen.entry_fn("main");
        let mut cs = Computations::new();
        //println!("{:?}", pygen);
        let main_comp = pygen.circify().consume().borrow().clone();
        //main_comp.outputs.push(leaf_term(Op::Const(Value::Bool(false))));
        println!("main: {:?}", main_comp.outputs);
        cs.comps.insert("main".to_string(), main_comp);
        cs
    }
}
/*
#[derive(Clone)]
enum PyLoc {
    Var(Loc)
}

impl PyLoc {
    fn loc(&self) -> &Loc {
        match self {
            PyLoc::Var(l) => l
        }
    }
}
*/

#[derive(Debug)]
struct PyGen {
    mode: Mode,
    circ: RefCell<Circify<Pyt>>,
    functions: HashMap<String, Funcdef>,
    /// Proof mode; find evaluations satisfying these.
    assumptions: Vec<Term>,
    /// Proof mode; find evaluations violating these.
    assertions: Vec<Term>,
    /// enable SV competition builtin functions
    sv_functions: bool
}

impl PyGen {
    fn new(config: Inputs, ast: Vec<Statement>) -> Self {
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
                },
                _ => panic!("Code is not only functions.")
            }
        }
        Self{
            mode: Mode::Proof,
            circ: RefCell::new(Circify::new(Pyt::new())),
            functions: functions,
            assumptions: vec![],
            assertions: vec![],
            sv_functions: config.sv_functions
        }
    }

    fn entry_fn(&mut self, name: &str) {
        // TODO FINISH SETUP FOR ENTRY
        let func = self.functions
            .get(name)
            .unwrap_or_else(|| panic!("Code does not have main function"))
            .clone();
        
        self.circ_enter_fn(name.to_owned(), Some(Ty::Int(32)));

        // TODO HANDLE OTHER KINDS OF ARGS AND ARG TYPES
        // TODO OTHER TYPES OF VISIBILITY
        for arg in func.parameters.args.iter() {
            let r = self.circ_declare_input(arg.0.clone(), &Ty::Int(32), PUBLIC_VIS, None, false);
            self.unwrap(r);
        }

        for ref stmt in func.code{
            self.gen_stmt(stmt);
        }
        // manually add calls to builtins for testing
        /*
        let assume_term = PyTerm{term: PyTermData::Bool(leaf_term(Op::Const(Value::Bool(false))))};
        let assert_term = PyTerm{term: PyTermData::Bool(leaf_term(Op::Const(Value::Bool(false))))};
        self.maybe_handle_builtins(&"__VERIFIER_assume".to_string(), &vec![assume_term]);
        self.maybe_handle_builtins(&"__VERIFIER_assert".to_string(), &vec![assert_term]);
        */
        if let Some(_r) = self.circ_exit_fn() {
            match self.mode {
                Mode::Proof => {
                    // Ensure non-empty
                    self.assumptions.push(bool_lit(true));
                    self.assertions.push(bool_lit(true));
                    let assumptions_hold = term(AND, self.assumptions.clone());
                    let an_assertion_doesnt = term(
                        OR,
                        self.assertions
                            .iter()
                            .map(|a| term![NOT; a.clone()])
                            .collect(),
                    );
                    let bug_if = term![AND; assumptions_hold, an_assertion_doesnt];
                    self.circ
                        .borrow()
                        .cir_ctx()
                        .cs
                        .borrow_mut()
                        .outputs
                        .push(bug_if);
                }
                _ => unimplemented!("Mode: {}", self.mode),
            }
        }
    }

    fn circ_declare_input(
        &self,
        name: String,
        ty: &Ty,
        vis: Option<PartyId>,
        precomputed_value: Option<PyTerm>,
        mangle_name: bool,
    ) -> Result<PyTerm, CircError> {
        self.circ
            .borrow_mut()
            .declare_input(name, ty, vis, precomputed_value, mangle_name)
    }

    fn gen_stmt(&mut self, stmt: &Statement){
        match stmt {
            Statement::Return(ret) => {
                let ret: PyTerm = self.gen_expr(&ret[0]);
                let ret_res = self.circ_return_(Some(ret));
                self.unwrap(ret_res);
            },
            Statement::Assignment(lhs, rhs) => {
                assert!(rhs.len() == 0); // can't handle real assignments yet
                assert!(lhs.len() == 1); // can only handle one expr on lhs for now
                self.gen_expr(&lhs[0]);
            },
            _ => unimplemented!("Statement {:#?} hasn't been implemented", stmt)
        }
    }

    fn gen_expr(&mut self, expr: &Expression) -> PyTerm {
        match expr {
            Expression::Int(int) => {
                self.integer(int)
                
            },
            Expression::False => {
                self.boolean(false)
            },
            Expression::True => {
                self.boolean(true)
            },
            Expression::Call(name_expr, arguments) => {
                // Get arguments
                let args = arguments
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => self.gen_expr(&expr),
                        _ => unimplemented!("Arg type not supported")
                    })
                    .collect::<Vec<_>>();
                let fname = match name_expr.as_ref() {
                    Expression::Name(string) => string,
                    _ => unimplemented!("Function name isn't string")
                };
                let maybe_return = self.maybe_handle_builtins(&fname, &args);
                if let Some(r) = maybe_return {
                    r
                } else {
                    unimplemented!("Can only handle builtin functions")
                }
            },
            Expression::Name(name) => {
                //PyLoc::Var(Loc::local(name))
                self
                .unwrap(self.circ_get_value(Loc::local(name.clone())))
                .unwrap_term()
            }
            _ => unimplemented!("Expr {:#?} hasn't been implemented", expr)
        }
    }

    /// Returns whether this was a builtin, and thus has been handled.
    fn maybe_handle_builtins(&mut self, name: &String, args: &Vec<PyTerm>) -> Option<PyTerm> {
        if self.sv_functions && (name == "__VERIFIER_assert" || name == "__VERIFIER_assume") {
            assert!(args.len() == 1);
            let bool_arg = cast_to_bool(args[0].clone());
            assert!(matches!(check(&bool_arg), Sort::Bool));
            if name == "__VERIFIER_assert" {
                self.assertions.push(bool_arg);
            } else {
                self.assumptions.push(bool_arg);
            }
            Some(Ty::Bool.default(self.circ.borrow().cir_ctx()))
        } else {
            None
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

    fn boolean(&self, b:bool) -> PyTerm{
        PyTerm{term: PyTermData::Bool(bool_lit(b))}
    }
    
    fn circify(&self) -> Circify<Pyt> {
        self.circ.replace(Circify::new(Pyt::new()))
    }

    fn circ_get_value(&self, loc: Loc) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow().get_value(loc)
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

    fn circ_exit_fn(&self) -> Option<Val<PyTerm>> {
        self.circ.borrow_mut().exit_fn()
    }
}
