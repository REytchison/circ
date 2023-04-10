#![allow(unused_imports)]
use circ::front::python::parser::PythonParser;
use std::env;
use std::path::Path;
use python_parser::ast::Statement;
use std::fs;

/// Debug program to print out parsed python
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        println!("Usage: ./py_parser path-to-.py");
        return;
    }
    let path = Path::new(&args[1]);
    let code: String = fs::read_to_string(path).unwrap();
    let ast: Vec<Statement>  = PythonParser::parse_code(&code);
    for el in ast{
        println!("{:?}\n", el);
    }
}