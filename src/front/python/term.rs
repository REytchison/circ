//! Terms in Python variant
#![allow(warnings)]
#![allow(unused)]
use crate::ir::term::{Term, Op, Sort, term};
use std::fmt::{self, Display, Formatter};
use crate::circify::{CirCtx, Embeddable};
use crate::ir::term::{bv_lit};
use crate::front::PartyId;
use crate::front::python::ty::Ty;
use crate::circify::Typed;

#[derive(Clone, Debug)]
pub enum PyTermData {
    Bool(Term),
    Int(usize, Term)
}

impl PyTermData {
    pub fn type_(&self) -> Ty {
        match self {
            Self::Bool(_) => Ty::Bool,
            Self::Int(w, _) => Ty::Int(*w)
        }
    }
    
    pub fn simple_term(&self) -> Term {
        match self {
            PyTermData::Bool(b) => b.clone(),
            PyTermData::Int(_, b) => b.clone(),
            _ => panic!(),
        }
    }
}

pub fn cast_to_bool(t: PyTerm) -> Term {
    cast(Some(Ty::Bool), t).term.simple_term()
}

pub fn cast(to_ty: Option<Ty>, t: PyTerm) -> PyTerm {
    let ty = t.term.type_();
    match t.term {
        PyTermData::Bool(ref term) => match to_ty {
            Some(Ty::Bool) => t.clone(),
            Some(Ty::Int(_w)) => unimplemented!("Casting from bool to int not added yet"),
            None => panic!("Bad cast from {} to {:?}", ty, to_ty)
        },
        PyTermData::Int(w, ref term) => match to_ty {
            Some(Ty::Bool) => PyTerm {
                term: PyTermData::Bool(term![Op::Not; term![Op::Eq; bv_lit(0, w), term.clone()]])
            },
            Some(Ty::Int(w)) => unimplemented!("Casting from int to int not added yet"),
            None => panic!("Bad cast from {} to {:?}", ty, to_ty)
        }
    }
}
#[derive(Clone, Debug)]
pub struct PyTerm {
    pub term: PyTermData,
    // add whether udef?
}

impl Display for PyTerm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Term: {:#?}", self.term)
    }
}

impl Typed<Ty> for PyTerm {
    // get type from internal PyTermData
    fn type_(&self) -> Ty {
        self.term.type_()
    }
}

/// Python language definition (values, )

pub struct Pyt {}

impl Pyt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Embeddable for Pyt{
    // /// Type for this language
    // type Ty: Display + Clone + Debug + PartialEq + Eq;
    // /// Terms for this language
    //type T: Display + Clone + Debug + Typed<Self::Ty>;
    type T = PyTerm;
    type Ty = Ty;

    /// Declare a language-level *input* to the computation.
    ///
    /// ## Arguments
    ///
    ///    * `ctx`: circuit context: you must add the circuit-level *input*
    ///    * `ty`: the type
    ///    * `name`: the name
    ///    * `visibility`: who knows it
    ///    * `precompute`: an optional term for pre-computing the values of this input. If a party
    ///    knows the inputs to the precomputation, they can use the precomputation.
    fn declare_input(
        &self,
        ctx: &mut CirCtx,
        ty: &Self::Ty,
        name: String,
        visibility: Option<PartyId>,
        precompute: Option<Self::T>,
    ) -> Self::T{
        match ty {
            Ty::Int(w) => Self::T{
                term: PyTermData::Int(
                    *w,
                    ctx.cs.borrow_mut().new_var(
                        &name,
                        Sort::BitVector(*w),
                        visibility,
                        precompute.map(|p| p.term.simple_term())
                    )
                )
            },
            _ => unimplemented!("input type not supported yet")
        }
    }

    /// Create a new uninitialized value of the given term in your language.
    ///
    /// For most languages, this should just be a kind of default value.
    ///
    /// ## Arguments
    ///
    ///    * `ctx`: circuit context: you must add the circuit-level *input*
    ///    * `ty`: the type
    fn create_uninit(&self, ctx: &mut CirCtx, ty: &Self::Ty) -> Self::T{
        // TODO ADD REAL UNITIALIZED
        let size:usize = 32;
        let num:i32 = 0;
        Self::T {
            term: PyTermData::Int(32,bv_lit(num, size))
        }
    }

    /// Construct an it-then-else (ternary) langauge value.
    ///
    /// Conceptually, `(ite cond t f)`
    fn ite(&self, ctx: &mut CirCtx, cond: Term, t: Self::T, f: Self::T) -> Self::T{
        // TODO ADD REAL ITE
        let size:usize = 32;
        let num:i32 = 0;
        Self::T {
            term: PyTermData::Int(32,bv_lit(num, size))
        }
    }

    /// Create a new term for the default return value of a function returning type `ty`.
    /// The name `ssa_name` is globally unique, and can be used if needed.
    // Because the type alias may change.
    #[allow(clippy::ptr_arg)]
    fn initialize_return(&self, ty: &Self::Ty, ssa_name: &String) -> Self::T{
        // TODO ADD REAL DEFAULT RETURN VAL
        let size:usize = 32;
        let num:i32 = 0;
        Self::T {
            term: PyTermData::Int(32,bv_lit(num, size))
        }
    }
}