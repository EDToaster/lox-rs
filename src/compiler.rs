use crate::{scanner::TokenScanner, vm::InterpretError};

pub struct Compiler {}

impl Compiler {
    pub fn compile(&mut self, source: &str) -> Result<(), InterpretError> {
        let mut scanner = TokenScanner::from_source(source);

        // Tmp debug
        println!("Line Token");
        let mut prev_line = 0;
        while let Some(token) = scanner.next() {
            if token.line != prev_line {
                prev_line = token.line;
                print!("{prev_line: >4} ");
            } else {
                print!("   | ");
            }
            println!("{token:?}");
        }
        Ok(())
    }
}
