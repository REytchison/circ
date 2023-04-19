use python_parser::ast::{Argument, Expression};
use crate::front::python::{PyGen};
use std::ops::Range;


pub fn range(args: &[Argument]) -> Result<Range<i32>, String>{
    match args {
        [Argument::Positional(Expression::Int(end_int))] => {
            let end: i32 = PyGen::pyint_to_i32(end_int);
            return Ok(0..end);
        },
        [Argument::Positional(Expression::Int(start_int)), Argument::Positional(Expression::Int(end_int))] => {
            let start: i32 = PyGen::pyint_to_i32(start_int);
            let end: i32 = PyGen::pyint_to_i32(end_int);
            return Ok(start..end);
        },
        [Argument::Positional(Expression::Int(_start_int)), Argument::Positional(Expression::Int(_end_int)), 
            Argument::Positional(Expression::Int(_step_int))] => {
            /*
            let start: i32 = PyGen::pyint_to_i32(start_int);
            let end: i32 = PyGen::pyint_to_i32(end_int);
            let step: i32 = PyGen::pyint_to_i32(step_int);
            return Ok((start..end).step_by(step));
            */
            unimplemented!("For with custom step not supported yet");
        },
        _ => return Err("Error: range builtin has invalid args".to_string())
    }
}