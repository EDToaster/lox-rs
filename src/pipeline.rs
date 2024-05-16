use crate::{
    compiler::Compiler,
    vm::{InterpretError, VM},
};

pub struct Pipeline;

impl Pipeline {
    pub fn interpret_source(&mut self, source: &str) -> Result<(), InterpretError> {
        let mut compiler = Compiler::new(source);
        VM::interpret(&compiler.compile()?)?;
        Ok(())
    }
}
