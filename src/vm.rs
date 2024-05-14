use crate::{
    chunk::{ByteCode, Chunk},
    value::Value,
};

#[derive(Debug, Clone, Copy)]
pub enum InterpretError {
    COMPILE,
    RUNTIME,
}

struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &Chunk) -> VM {
        VM {
            chunk,
            stack: vec![],
        }
    }
}

pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
    let mut vm = VM::new(chunk);

    let mut iterator = chunk.into_iter();

    while let Some((offset, bytecode)) = iterator.next() {
        match bytecode {
            ByteCode::Return => break,
            ByteCode::Constant(idx) => vm.stack.push(chunk.get_constant(idx as usize)),
            ByteCode::ConstantLong(idx) => vm.stack.push(chunk.get_constant(idx as usize)),
            ByteCode::Negate => {
                let val = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                vm.stack.push(-val);
            }
            ByteCode::Add => {
                let r = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                let l = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                vm.stack.push(l + r);
            }
            ByteCode::Sub => {
                let r = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                let l = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                vm.stack.push(l - r);
            }
            ByteCode::Mul => {
                let r = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                let l = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                vm.stack.push(l * r);
            }
            ByteCode::Div => {
                let r = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                let l = vm.stack.pop().ok_or(InterpretError::RUNTIME)?;
                vm.stack.push(l / r);
            }
        }
    }

    dbg!(vm.stack);

    Ok(())
}
