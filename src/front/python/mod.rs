//! The Python front-end

pub mod parser;
pub mod term;
pub mod ty;
mod builtins;

use crate::ir::term::{Computations, PartyId};
use crate::front::PUBLIC_VIS;
use super::{FrontEnd, Mode};
use std::path::PathBuf;
use crate::ir::term::{bv_lit, term, NOT, AND, OR, Term, Sort, check, bool_lit};
use parser::PythonParser;
use python_parser::ast::{
    CompoundStatement, Funcdef, Statement, Expression, IntegerType, Argument,
    Bop, Uop
};
use term::{
    PyTerm, PyTermData, Pyt, cast_to_bool, eq, ne, add, bitxor, cast, 
    floor_div, bitand, bitor, lt, gt, le, ge, sub, mult, minus
};
use std::fs;
use std::collections::HashMap;
use crate::circify::{CircError, Circify, Val, Loc};
use std::fmt::Display;
use std::str::FromStr;
use std::cell::RefCell;
use crate::front::python::ty::{Ty, PY_INT_SIZE};
use crate::front::python::builtins::range;

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
        let ast: Vec<Statement>  = PythonParser::parse_code(&code);
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

#[derive(Clone)]
enum PyLoc {
    Var(Loc)
}
/*
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
            // TODO ROLL INTO gen_compound_stmt
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
        
        self.circ_enter_fn(name.to_owned(), Some(Ty::Int));

        // TODO OTHER TYPES OF VISIBILITY
        for arg in func.parameters.args.iter() {
            let arg_name = &arg.0;
            let ty_expr = arg.1.clone().unwrap_or_else(|| panic!("Argument does not have type hint"));
            let ty_string = match ty_expr{
                Expression::Name(string) => string,
                _ => panic!("Type hint is not Name")
            };
            let ty: Ty = Ty::from_str(&ty_string).unwrap_or_else(|e| self.err(e));
            let r = self.circ_declare_input(arg_name.clone(), &ty, PUBLIC_VIS, None, false);
            self.unwrap(r);
        }

        for ref stmt in func.code{
            self.gen_stmt(stmt);
        }
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

    fn gen_decl(&mut self, name: &str, ty_str: &str, term: PyTerm){
        let ty: Ty = Ty::from_str(ty_str).unwrap_or_else(|e| self.err(e));
        let res = self.circ_declare_init(
            name.to_string(),
            ty.clone(),
            Val::Term(cast(Some(ty.clone()), term.clone())),
        );
        self.unwrap(res);
    }

    fn gen_stmt(&mut self, stmt: &Statement){
        match stmt {
            Statement::Return(ret) => {
                let ret: PyTerm = self.gen_expr(&ret[0]);
                let ret_res = self.circ_return_(Some(ret));
                self.unwrap(ret_res);
            },
            Statement::Assignment(lhs, rhs) => {
                assert!(lhs.len() == 1); // can only handle one expr on lhs for now
                if rhs.len() > 0 {
                    assert!(rhs[0].len() == 1); // can only handle one or less expr on rhs for now
                    // real assignment
                    let loc = self.gen_lval(&lhs[0]);
                    let val = self.gen_expr(&rhs[0][0]);
                    let already_declared = match &lhs[0] {
                        Expression::Name(name) => self.circ_already_declared(name),
                        _ => unimplemented!("Invalid left value")
                    };
                    if !already_declared{
                        unimplemented!("Declarations are only implemented for typed assignments");
                    }
                    let assign_res = self.gen_assign(loc, val);
                    self.unwrap(assign_res);
                } else {
                    // just expression(s)
                    self.gen_expr(&lhs[0]);
                }
            },
            Statement::Compound(stmt) => self.gen_compound_stmt(stmt),
            Statement::TypedAssignment(lhs, ty, rhs) =>{
                assert!(lhs.len() == 1); // can only handle one expr on lhs for now
                assert!(rhs.len() == 1); // can only handle one expr on rhs for now
                let var_name = match &lhs[0] {
                    Expression::Name(name) => name,
                    _ => panic!("Left value is not Name")
                };
                let ty_str = match ty {
                    Expression::Name(name) => name,
                    _ => panic!("Type annotation is not Name")
                };
                let loc = self.gen_lval(&lhs[0]);
                let val = self.gen_expr(&rhs[0]);
                if self.circ_already_declared(var_name){
                    // ignore type hint for non-declaration assignments
                    let assign_res = self.gen_assign(loc, val);
                    self.unwrap(assign_res);
                } else {
                    self.gen_decl(&var_name, ty_str, val);
                }
            }
            _ => unimplemented!("Statement {:#?} hasn't been implemented", stmt)
        }
    }

    fn gen_compound_stmt(&mut self, stmt: &CompoundStatement){
        match stmt {
            CompoundStatement::If(cond_stmts, else_stmt_opt) => {
                assert!(cond_stmts.len() == 1); // can't handle elif yet
                let cond_stmt = &cond_stmts[0];
                let cond = self.gen_expr(&cond_stmt.0);
                let cond_term = cond.term.simple_term();
                self.circ_enter_condition(cond_term.clone());
                for inner_stmt in cond_stmt.1.iter(){
                    self.gen_stmt(inner_stmt);
                }
                self.circ_exit_condition();
                if let Some(else_stmt) = else_stmt_opt {
                    self.circ_enter_condition(term!(NOT; cond_term));
                    for inner_stmt in else_stmt.iter(){
                        self.gen_stmt(inner_stmt);
                    }
                    self.circ_exit_condition();
                }
            },
            CompoundStatement::For{r#async, item, iterator, for_block, else_block} => {
                assert!(!r#async); // can't handle async for loops
                assert!(else_block.is_none()); // can't handle else block yet
                // Get loop bounds
                match &iterator[..] {
                    [Expression::Call(box_expr, args)] if **box_expr == Expression::Name("range".to_string()) => {
                        let range = range(&args).unwrap_or_else(|e| self.err(e));
                        let _placeholder_to_compile = item;
                        for _ in range{
                            self.circ_enter_scope();
                            for for_stmt in for_block{
                                self.gen_stmt(for_stmt);
                            }
                            self.circ_exit_scope();
                        }
                    },
                    _ => unimplemented!("Only supports range for iter for now")
                }
                // TODO USE item
            }
            _ => unimplemented!("Compound Statement {:#?} hasn't been implemented", stmt)
        }
    }

    fn gen_expr(&mut self, expr: &Expression) -> PyTerm {
        match expr {
            Expression::Int(int) => {
                Self::integer(int)
                
            },
            Expression::False => {
                Self::boolean(false)
            },
            Expression::True => {
                Self::boolean(true)
            },
            Expression::Bop(bop, expr0, expr1) => {
                let t0 = self.gen_expr(expr0);
                let t1 = self.gen_expr(expr1);
                let f = self.get_bin_op(bop);
                f(t0, t1).unwrap()
            },
            Expression::Uop(uop, expr) => {
                let t = self.gen_expr(expr);
                match uop {
                    Uop::Minus => minus(t).unwrap(),
                    _ => unimplemented!("Unary op not implemented yet")
                }
            }
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
                self
                .unwrap(self.circ_get_value(Loc::local(name.clone())))
                .unwrap_term()
            }
            _ => unimplemented!("Expr {:#?} hasn't been implemented", expr)
        }
    }

    fn get_bin_op(&self, op: &Bop) -> fn(PyTerm, PyTerm) -> Result<PyTerm, String>{
        match op {
            Bop::Eq => eq,
            Bop::Neq => ne,
            Bop::Add => add,
            Bop::BitXor => bitxor,
            Bop::Sub => sub,
            Bop::Mult => mult,
            Bop::Floordiv => floor_div,
            Bop::BitAnd => bitand,
            Bop::BitOr => bitor,
            Bop::Lt => lt,
            Bop::Gt => gt,
            Bop::Leq => le,
            Bop::Geq => ge,
            Bop::And => bitand,
            Bop::Or => bitor, 
            _ => unimplemented!("Binary op not implemented yet")
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

    fn gen_lval(&mut self, expr: &Expression) -> PyLoc {
        match &expr {
            Expression::Name(string) => {
                PyLoc::Var(Loc::local(string.clone()))
            },
            _ => unimplemented!("Invalid left value")
        }
    }

    fn gen_assign(&mut self, loc: PyLoc, val: PyTerm) -> Result<PyTerm, String> {
        match loc {
            PyLoc::Var(l) => {
                Ok(self
                    .circ_assign(l, Val::Term(val))
                    .map_err(|e| format!("{e}"))?
                    .unwrap_term())
            }
        }
    }

    fn pyint_to_i32(int: &IntegerType) -> i32{
        let radix:u32 = 10;
        // TODO handling int size?
        let int_str: String = int.to_str_radix(radix);
        return i32::from_str(&int_str).unwrap();
    }

    fn integer(int: &IntegerType) -> PyTerm {
        let num = Self::pyint_to_i32(int);
        PyTerm{term: PyTermData::Int(bv_lit(num, PY_INT_SIZE))}
    }

    fn boolean(b:bool) -> PyTerm{
        PyTerm{term: PyTermData::Bool(bool_lit(b))}
    }
    
    fn circify(&self) -> Circify<Pyt> {
        self.circ.replace(Circify::new(Pyt::new()))
    }

    fn circ_already_declared(&self, name: &str) -> bool{
        self.circ.borrow().already_declared(name)
    }

    fn circ_assign(&self, loc: Loc, val: Val<PyTerm>) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow_mut().assign(loc, val)
    }

    fn circ_get_value(&self, loc: Loc) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow().get_value(loc)
    }

    fn circ_enter_condition(&self, cond: Term) {
        self.circ.borrow_mut().enter_condition(cond).unwrap();
    }

    fn circ_exit_condition(&self) {
        self.circ.borrow_mut().exit_condition()
    }

    fn circ_return_(&self, ret: Option<PyTerm>) -> Result<(), CircError> {
        self.circ.borrow_mut().return_(ret)
    }
    
    fn circ_enter_fn(&self, f_name: String, ret_ty: Option<Ty>) {
        self.circ.borrow_mut().enter_fn(f_name, ret_ty)
    }
    
    fn circ_declare_init(
        &self,
        name: String,
        ty: Ty,
        val: Val<PyTerm>,
    ) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow_mut().declare_init(name, ty, val)
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

    fn circ_enter_scope(&self) {
        self.circ.borrow_mut().enter_scope()
    }

    fn circ_exit_scope(&self) {
        self.circ.borrow_mut().exit_scope()
    }
}
