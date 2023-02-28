//! The Python front-end
use crate::ir::term::{Computations, Computation};
use super::{FrontEnd};
use std::path::PathBuf;
use crate::ir::term::{Op, leaf_term, Value};
//use ty::Ty;
//se crate::ir::term::{term, Term};


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
        let _j = i;
        let mut cs = Computations::new();
        let mut comp = Computation::new();
        comp.outputs.push(leaf_term(Op::Const(Value::Bool(false))));
        cs.comps.insert("main".to_string(), comp);
        cs
    }
}
