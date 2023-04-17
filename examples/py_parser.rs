#![allow(unused_imports)]
use circ::front::python::parser::PythonParser;
use std::env;
use std::path::Path;
use python_parser::ast::{Statement, CompoundStatement, Funcdef};
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
        match el {
            Statement::Compound(boxed_compound) => {
                match *boxed_compound{
                    CompoundStatement::Funcdef(funcdef) => {
                        println!("FUNCTION: {}", funcdef.name);
                        for stmt in funcdef.code{
                            println!("{:?}", stmt);
                        }
                    }
                    _ => println!("NOT A FUNCTION")
                }
            },
            _ => println!("NOT A COMPOUND STATEMENT")
        }
    }
}