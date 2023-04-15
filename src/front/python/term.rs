//! Terms in Python variant
#![allow(warnings)]
#![allow(unused)]
use crate::ir::term::{Term, Op, Sort, term, BvNaryOp};
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

impl Display for PyTermData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PyTermData::Bool(x) => write!(f, "Bool({x})"),
            PyTermData::Int(_, x) => write!(f, "Int({x})")
        }
    }
}

pub fn cast_to_bool(t: PyTerm) -> Term {
    cast(Some(Ty::Bool), t).term.simple_term()
}

pub fn cast(to_ty: Option<Ty>, t: PyTerm) -> PyTerm {
    // TODO ADD CASTS BETWEEN BOOLS AND INTS
    let ty = t.term.type_();
    match t.term {
        PyTermData::Bool(ref term) => match to_ty {
            Some(Ty::Bool) => t.clone(),
            Some(Ty::Int(_w)) => unimplemented!("Casting from bool to int not added yet"),
            None => panic!("Bad cast from {} to {:?}", ty, to_ty)
        },
        PyTermData::Int(w0, ref term) => match to_ty {
            Some(Ty::Bool) => PyTerm {
                term: PyTermData::Bool(term![Op::Not; term![Op::Eq; bv_lit(0, w0), term.clone()]])
            },
            Some(Ty::Int(w1)) => PyTerm {
                // Python ints can increase in size, so no reason to not pick larger size
                term: PyTermData::Int(if (w0 > w1) {w0} else {w1}, term.clone())
            },
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
            Ty::Bool => Self::T{
                term: PyTermData::Bool(
                    ctx.cs.borrow_mut().new_var(
                        &name,
                        Sort::Bool,
                        visibility,
                        precompute.map(|p| p.term.simple_term())
                    )
                )
            }
        }
    }

    /// Construct an it-then-else (ternary) langauge value.
    ///
    /// Conceptually, `(ite cond t f)`
    fn ite(&self, ctx: &mut CirCtx, cond: Term, t: Self::T, f: Self::T) -> Self::T{
        match (t.term, f.term) {
            (PyTermData::Bool(a), PyTermData::Bool(b)) => Self::T {
                term: PyTermData::Bool(term![Op::Ite; cond, a, b])
            },
            (PyTermData::Int(wa, a), PyTermData::Int(wb, b)) if wa == wb => Self::T {
                term: PyTermData::Int(wa, term![Op::Ite; cond, a, b])
            },
            (t, f) => panic!("Cannot ITE {} and {}", t, f)
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
        ty.default(ctx)
    }

    /// Create a new term for the default return value of a function returning type `ty`.
    /// The name `ssa_name` is globally unique, and can be used if needed.
    // Because the type alias may change.
    #[allow(clippy::ptr_arg)]
    fn initialize_return(&self, ty: &Self::Ty, ssa_name: &String) -> Self::T{
        // TODO UNCLEAR HOW THIS IS DIFFERENT FROM create_uninit
        match ty{
            Ty::Int(w) => {
                PyTerm{
                    term: PyTermData::Int(*w, Sort::BitVector(*w).default_term())
                }
            },
            Ty::Bool => {
                PyTerm {
                    term: PyTermData::Bool(Sort::Bool.default_term())
                }
            }
        }
    }
}


pub fn eq(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("==", eq_base, a, b)
}

pub fn neq(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("!=", neq_base, a, b)
}

// TODO IS BvNaryOp THE CORRECT PRIMITIVE (vs Integer type)
fn add_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Add); a, b]
}

pub fn add(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("+", add_uint, a, b)
}

fn bitxor_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Xor); a, b]
}

pub fn bitxor(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("^", bitxor_uint, a, b)
}

fn wrap_bin_arith(
    name: &str,
    func: fn(Term, Term) -> Term,
    a: PyTerm,
    b: PyTerm,
) -> Result<PyTerm, String> {
    // TODO CONVERSIONS
    match (&a.term, &b.term) {
        // TODO WIDENING SEMANTICS AND BOOL ARITHMETIC
        (PyTermData::Int(wx, x), PyTermData::Int(wy, y)) if wx == wy => {
            Ok(PyTerm {
                term: PyTermData::Int(*wx, func(x.clone(), y.clone()))
            })
        },
        (PyTermData::Bool(x), PyTermData::Bool(y)) => {
            Ok(PyTerm {
                term: PyTermData::Bool(func(x.clone(), y.clone()))
            })
        },
        (_, _) => Err(format!(" op '{name}' on {a} and {b}")),
    }
}

fn wrap_bin_cmp(
    // TODO HANDLE MORE DATATYPES AND CONVERSIONS
    name: &str,
    func: fn(Term, Term) -> Term,
    a: PyTerm,
    b: PyTerm,
) -> Result<PyTerm, String> {
    match (&a.term, &b.term) {
        (PyTermData::Int(_w0, t0), PyTermData::Int(_w1, t1)) => {
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0.clone(), t1.clone()))
            })
        },
        (PyTermData::Bool(t0), PyTermData::Bool(t1)) => {
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0.clone(), t1.clone()))
            })
        },
        (_, _) => Err(format!("Cannot perform op {name} on {a} and {b}"))
    }
}

fn eq_base(a: Term, b: Term) -> Term {
    term![Op::Eq; a, b]
}

fn neq_base(a: Term, b: Term) -> Term {
    term![Op::Not; term![Op::Eq; a, b]]
}
