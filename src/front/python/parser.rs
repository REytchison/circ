//! Parsing python files
use python_parser::{file_input, make_strspan, ast::Statement};

/// Parse python files
pub struct PythonParser;


impl PythonParser {
    /// Parse python code
    pub fn parse_file(code: &str) -> Vec<Statement>{
        let strspan = make_strspan(code);
        // TODO fix error handling
        let ast: Vec<Statement> = file_input(strspan).unwrap().1;
        return ast;
    }
}