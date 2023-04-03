//! Types for python variant
#![allow(warnings)]
use std::fmt::{self, Display, Formatter};
use super::term::{PyTerm, PyTermData};
use crate::circify::CirCtx;
use crate::ir::term::{Term, Sort};
/// A type
#[derive(Clone, Eq)]
pub enum Ty {
    Int(usize),
    Bool
}

impl Ty{
    pub fn sort(&self) -> Sort {
        match self {
            Self::Bool => Sort::Bool,
            Self::Int(w) => Sort::BitVector(*w)
        }
    }

    fn default_ir_term(&self) -> Term {
        self.sort().default_term()
    }
    pub fn default(&self, ctx: &CirCtx) -> PyTerm {
        match self {
            Self::Bool => PyTerm {
                term: PyTermData::Bool(self.default_ir_term())
            },
            Self::Int(w) => PyTerm {
                term: PyTermData::Int(*w, self.default_ir_term())
            }
        }
    }
}

impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        use Ty::*;
        match (self, other) {
            (Int(a_size), Int(b_size)) => a_size == b_size,
            (Bool, Bool) => true,
            (Int(_w), Bool) => false,
            (Bool, Int(_w)) => false
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Int(w) => {
                write!(f, "s{w}")
            },
            Ty::Bool => {
                write!(f, "bool")
            },
        }
    }
}

impl fmt::Debug for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}