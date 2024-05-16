use crate::{compiler::Compiler, vm::InterpretError};

pub struct Pipeline {}

impl Pipeline {
    pub fn interpret_source(&mut self, source: &str) -> Result<(), InterpretError> {
        let mut compiler = Compiler {};
        compiler.compile(source)?;
        Ok(())
    }
}
