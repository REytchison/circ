//! Types for python variant
#![allow(warnings)]
use std::fmt::{self, Display, Formatter};
use super::term::{PyTerm, PyTermData};
use crate::circify::CirCtx;
use crate::ir::term::{Term, Sort};
use std::str::FromStr;

/*
Note: Python integers are more precisely represented with CirC's Sort::Int,
which has arbitrary precision like Python integers. However Sort::BitVector was
chosen because Sort::Int currently supports very few operations.
*/
pub const PY_INT_SIZE: usize = 32;

/// A type
#[derive(Clone, Eq)]
pub enum Ty {
    Int,
    Bool
}

impl Ty{
    pub fn sort(&self) -> Sort {
        match self {
            Self::Bool => Sort::Bool,
            Self::Int => Sort::BitVector(PY_INT_SIZE)
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
            Self::Int => PyTerm {
                term: PyTermData::Int(self.default_ir_term())
            }
        }
    }
}


impl FromStr for Ty {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>{
        match s {
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            _ => Err("ParsePythonTyError".to_string())
        }
    }
}


impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        use Ty::*;
        match (self, other) {
            (Int, Int) => true,
            (Bool, Bool) => true,
            (Int, Bool) => false,
            (Bool, Int) => false
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Int => {
                write!(f, "int")
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