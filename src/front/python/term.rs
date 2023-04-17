//! Terms in Python variant
#![allow(warnings)]
#![allow(unused)]
use crate::ir::term::{Term, Op, Sort, term, BvNaryOp, BvBinPred, BvBinOp, BvUnOp};
use std::fmt::{self, Display, Formatter};
use crate::circify::{CirCtx, Embeddable};
use crate::ir::term::{bv_lit};
use crate::front::PartyId;
use crate::front::python::ty::{Ty, PY_INT_SIZE};
use crate::circify::Typed;

#[derive(Clone, Debug)]
pub enum PyTermData {
    Bool(Term),
    Int(Term)
}

impl PyTermData {
    pub fn type_(&self) -> Ty {
        match self {
            Self::Bool(_) => Ty::Bool,
            Self::Int(_) => Ty::Int
        }
    }
    
    pub fn simple_term(&self) -> Term {
        match self {
            PyTermData::Bool(b) => b.clone(),
            PyTermData::Int(b) => b.clone(),
            _ => panic!(),
        }
    }
}

impl Display for PyTermData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PyTermData::Bool(x) => write!(f, "Bool({x})"),
            PyTermData::Int(x) => write!(f, "Int({x})")
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
            Some(Ty::Int) => PyTerm {
                term: PyTermData::Int(
                    term![Op::BvUext(PY_INT_SIZE-1); term![Op::BoolToBv; term.clone()]]
                )
            },
            None => panic!("Bad cast from {} to {:?}", ty, to_ty)
        },
        PyTermData::Int(ref term) => match to_ty {
            Some(Ty::Bool) => PyTerm {
                term: PyTermData::Bool(term![Op::Not; term![Op::Eq; bv_lit(0, PY_INT_SIZE), term.clone()]])
            },
            Some(Ty::Int) => PyTerm {
                term: PyTermData::Int(term.clone())
            },
            None => panic!("Bad cast from {} to {:?}", ty, to_ty)
        }
    }
}
#[derive(Clone, Debug)]
pub struct PyTerm {
    pub term: PyTermData
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
            Ty::Int => Self::T{
                term: PyTermData::Int(
                    ctx.cs.borrow_mut().new_var(
                        &name,
                        Sort::BitVector(PY_INT_SIZE),
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
            (PyTermData::Int(a), PyTermData::Int(b)) => Self::T {
                term: PyTermData::Int(term![Op::Ite; cond, a, b])
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
            Ty::Int => {
                PyTerm{
                    term: PyTermData::Int(Sort::BitVector(PY_INT_SIZE).default_term())
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

/// For operations which always coerce to numeric types or are bitvec ops
fn wrap_bin_arith(
    name: &str,
    func: fn(Term, Term) -> Term,
    a: PyTerm,
    b: PyTerm,
) -> Result<PyTerm, String> {
    match (&a.term) {
        // TODO handle each case explicitly?
        PyTermData::Int(x) => {
            let b_cast = cast(Some(Ty::Int), b).term.simple_term();
            Ok(PyTerm {
                term: PyTermData::Int(func(x.clone(), b_cast))
            })
        },
        PyTermData::Bool(_) => {
            let a_cast = cast(Some(Ty::Int), a).term.simple_term();
            let b_cast = cast(Some(Ty::Int), b).term.simple_term();
            Ok(PyTerm {
                term: PyTermData::Int(func(a_cast, b_cast))
            })
        },
        (_) => Err(format!(" op '{name}' on {a} and {b}")),
    }
}

fn add_int(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Add); a, b]
}

pub fn add(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("+", add_int, a, b)
}

fn bitand_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::And); a, b]
}

pub fn bitand(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("&", bitand_uint, a, b)
}

fn bitor_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Or); a, b]
}

pub fn bitor(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("|", bitor_uint, a, b)
}


fn bitxor_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Xor); a, b]
}

pub fn bitxor(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("^", bitxor_uint, a, b)
}

fn sub_uint(a: Term, b: Term) -> Term {
    term![Op::BvBinOp(BvBinOp::Sub); a, b]
}

pub fn sub(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("-", sub_uint, a, b)
}

fn mult_uint(a: Term, b: Term) -> Term {
    term![Op::BvNaryOp(BvNaryOp::Mul); a, b]
}

pub fn mult(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("*", mult_uint, a, b)
}

fn floor_div_uint(a: Term, b: Term) -> Term {
    term![Op::BvBinOp(BvBinOp::Udiv); a, b]
}

pub fn floor_div(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_arith("//", floor_div_uint, a, b)
}

fn wrap_un_arith(
    name: &str,
    func: fn(Term) -> Term,
    a: PyTerm
) -> Result<PyTerm, String> {
    match(&a.term) {
        PyTermData::Int(x) => {
            Ok(PyTerm {
                term: PyTermData::Int(func(x.clone()))
            })
        },
        PyTermData::Bool(x) => {
            let a_cast = cast(Some(Ty::Int), a).term.simple_term();
            Ok(PyTerm {
                term: PyTermData::Int(func(a_cast))
            })
        },
        _ => Err(format!(" op '{name}' on {a} casting failed"))
    }
}

fn minus_uint(a: Term) -> Term {
    term![Op::BvUnOp(BvUnOp::Neg); a]
}

pub fn minus(a: PyTerm) -> Result<PyTerm, String> {
    wrap_un_arith("-", minus_uint, a)
}

fn wrap_bin_cmp(
    name: &str,
    func: fn(Term, Term) -> Term,
    a: PyTerm,
    b: PyTerm,
) -> Result<PyTerm, String> {
    match (&a.term, &b.term) {
        (PyTermData::Bool(t0), PyTermData::Bool(t1)) => {
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0.clone(), t1.clone()))
            })
        },
        (PyTermData::Int(t0), PyTermData::Int(t1)) => {
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0.clone(), t1.clone()))
            })
        },
        (PyTermData::Int(t0), PyTermData::Bool(t1)) => {
            let t1_cast = cast(Some(Ty::Int), b).term.simple_term();
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0.clone(), t1_cast))
            })
        },
        (PyTermData::Bool(t0), PyTermData::Int(t1)) => {
            let t0_cast = cast(Some(Ty::Int), a).term.simple_term();
            Ok(PyTerm{
                term: PyTermData::Bool(func(t0_cast, t1.clone()))
            })
        }
        (_, _) => Err(format!("Cannot perform op {name} on {a} and {b}"))
    }
}

pub fn eq(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("==", eq_base, a, b)
}

pub fn ne(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("!=", ne_base, a, b)
}

fn eq_base(a: Term, b: Term) -> Term {
    term![Op::Eq; a, b]
}

fn ne_base(a: Term, b: Term) -> Term {
    term![Op::Not; term![Op::Eq; a, b]]
}

fn lt_int(a: Term, b: Term) -> Term {
    term![Op::BvBinPred(BvBinPred::Slt); a, b]
}

pub fn lt(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("<", lt_int, a, b)
}

fn le_int(a: Term, b: Term) -> Term {
    term![Op::BvBinPred(BvBinPred::Sle); a, b]
}

pub fn le(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp("<=", le_int, a, b)
}

fn gt_int(a: Term, b: Term) -> Term {
    term![Op::BvBinPred(BvBinPred::Sgt); a, b]
}

pub fn gt(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp(">", gt_int, a, b)
}

fn ge_int(a: Term, b: Term) -> Term {
    term![Op::BvBinPred(BvBinPred::Sge); a, b]
}

pub fn ge(a: PyTerm, b: PyTerm) -> Result<PyTerm, String> {
    wrap_bin_cmp(">=", ge_int, a, b)
}