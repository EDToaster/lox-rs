use crate::{
    compiler::Compiler,
    vm::{InterpretError, VM},
};

pub struct Pipeline;

impl Pipeline {
    pub fn interpret_source(&mut self, source: &str) -> Result<(), InterpretError> {
        // let mut scanner = ErrorIgnoreTokenScanner {
        //     inner: TokenScanner::from_source(source),
        // };

        // while let Some(t) = scanner.next() {
        //     println!("{t:?}");
        // }

        let compiler = Compiler::new(source);

        VM::interpret(&compiler.compile()?.chunk)?;
        Ok(())
    }
}
