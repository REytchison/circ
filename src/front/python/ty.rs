//! Types for python variant
#![allow(warnings)]
use std::fmt::{self, Display, Formatter};

/// A type
#[derive(Clone, Eq)]
pub enum Ty {
    Int(usize)
}

impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        use Ty::*;
        match (self, other) {
            (Int(a_size), Int(b_size)) => a_size == b_size
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Int(w) => {
                write!(f, "{w}")
            },
        }
    }
}

impl fmt::Debug for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}