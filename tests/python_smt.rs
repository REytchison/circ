//! run smt tests: cargo test --test python_smt --features smt,python
#![cfg(all(feature = "python", feature="smt"))]
use circ::front::python::{self, Python};
use circ::ir::opt::{opt, Opt};
use circ::ir::term::Computations;
use std::path::PathBuf;
use circ::target::smt::find_model;
use circ::front::FrontEnd;
/// Optimize according to SMT mode in circ.rs
fn smt_optimize(cs: Computations) -> Computations{
    let mut opts = Vec::new();
    opts.push(Opt::ScalarizeVars);
    opts.push(Opt::Flatten);
    opts.push(Opt::Sha);
    opts.push(Opt::ConstantFold(Box::new([])));
    opts.push(Opt::ParseCondStores);
    // Tuples must be eliminated before oblivious array elim
    opts.push(Opt::Tuple);
    opts.push(Opt::ConstantFold(Box::new([])));
    opts.push(Opt::Tuple);
    opts.push(Opt::Obliv);
    // The obliv elim pass produces more tuples, that must be eliminated
    opts.push(Opt::Tuple);
    //if options.circ.ram.enabled {
    //opts.push(Opt::PersistentRam);
    //opts.push(Opt::VolatileRam);
    //opts.push(Opt::SkolemizeChallenges);
    //}
    opts.push(Opt::LinearScan);
    // The linear scan pass produces more tuples, that must be eliminated
    opts.push(Opt::Tuple);
    opts.push(Opt::Flatten);
    opts.push(Opt::ConstantFold(Box::new([])));
    opt(cs, opts)

}

/// Determine whether SMT assumptions hold in given python file
fn smt_holds_test(test_file: &str) -> bool{
    let mut file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    file.push("examples/python/smt");
    file.push(test_file);
    let inputs = python::Inputs {
        file: file,
        sv_functions: true
    };
    let mut cs = Python::gen(inputs);
    cs = smt_optimize(cs);
    let main_comp = cs.get("main").clone();
    println!("outputs: {:?}", main_comp.outputs);
    assert_eq!(main_comp.outputs.len(), 1);
    let model = find_model(&main_comp.outputs[0]);
    let model_holds = model.is_none();
    model_holds
}

#[test]
fn basic_fail(){
    let holds_expected = false;
    let test_python_file = "assert_assume_fails.py";
    let model_holds = smt_holds_test(&test_python_file);
    assert_eq!(model_holds, holds_expected);
}

#[test]
fn basic_ok(){
    let holds_expected = true;
    let test_python_file = "assert_assume_ok.py";
    let model_holds = smt_holds_test(&test_python_file);
    assert_eq!(model_holds, holds_expected);
}

#[test]
fn assign_ok(){
    let holds_expected = true;
    let test_python_file = "assign_ok.py";
    let model_holds = smt_holds_test(&test_python_file);
    assert_eq!(model_holds, holds_expected);
}

#[test]
fn if_ok(){
    let holds_expected = true;
    let test_python_file = "if_ok.py";
    let model_holds = smt_holds_test(&test_python_file);
    assert_eq!(model_holds, holds_expected);
}

#[test]
fn for_ok(){
    let holds_expected = true;
    let test_python_file = "for_ok.py";
    let model_holds = smt_holds_test(&test_python_file);
    assert_eq!(model_holds, holds_expected);
}