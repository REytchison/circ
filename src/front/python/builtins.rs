use python_parser::ast::{Argument, Expression};
use crate::front::python::{PyGen};
use std::ops::Range;


pub fn range(args: &[Argument]) -> Result<Range<i32>, String>{
    match args {
        [Argument::Positional(Expression::Int(start_int))] => {
            let start: i32 = PyGen::pyint_to_i32(start_int);
            return Ok(0..start);
        },
        [Argument::Positional(_start_expr), Argument::Positional(_end_expr)] => {
            unimplemented!("End argument not added yet");
        },
        [Argument::Positional(_start_expr), Argument::Positional(_end_expr), Argument::Positional(_step_expr)] => {
            unimplemented!("Step argument not added yet");
        },
        _ => return Err("Error: range builtin has invalid args".to_string())
    }
}