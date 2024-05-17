use crate::{
    chunk::{ByteCode, Chunk},
    value::Value,
};

#[derive(Debug, Clone, Copy)]
pub enum InterpretError {
    Compiler,
    Runtime,
}

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
}

fn report_error(line: usize, bytecode: &ByteCode, msg: &str) -> Result<(), InterpretError> {
    println!("Error at line {line}, bytecode '{bytecode:?}': {msg}");
    Err(InterpretError::Runtime)
}

impl<'a> VM<'a> {
    pub fn new(chunk: &Chunk) -> VM {
        VM {
            chunk,
            stack: vec![],
        }
    }

    pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
        let mut vm = VM::new(chunk);

        let mut iterator = chunk.into_iter();

        while let Some((offset, bytecode)) = iterator.next() {
            use ByteCode::*;
            match bytecode {
                Return => break,
                Constant(idx) => vm.stack.push(chunk.get_constant(idx as u32)),
                ConstantLong(idx) => vm.stack.push(chunk.get_constant(idx as u32)),
                Nil => vm.stack.push(Value::Nil),
                True => vm.stack.push(Value::Bool(true)),
                False => vm.stack.push(Value::Bool(false)),
                Negate => {
                    let val = match vm.stack.pop().ok_or(InterpretError::Runtime)? {
                        Value::Number(val) => -val,
                        v => {
                            return report_error(
                                chunk.get_line(offset),
                                &bytecode,
                                &format!("Operand must be a number, found {v:?}"),
                            )
                        }
                    };

                    vm.stack.push(Value::Number(val));
                }
                Add | Sub | Mul | Div => {
                    let r = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let l = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let (l, r) = match (l, r) {
                        (Value::Number(l), Value::Number(r)) => (l, r),
                        _ => {
                            return report_error(
                                chunk.get_line(offset),
                                &bytecode,
                                &format!("Operands must be numbers, found {l:?}, {r:?}"),
                            )
                        }
                    };
                    let res = match bytecode {
                        Add => l + r,
                        Sub => l - r,
                        Mul => l * r,
                        Div => l / r,
                        _ => unreachable!(),
                    };
                    vm.stack.push(Value::Number(res));
                }
                Not => {
                    let val = !vm.stack.pop().ok_or(InterpretError::Runtime)?.is_truthy();
                    vm.stack.push(Value::Bool(val));
                }
                Eq => {
                    let r = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let l = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    vm.stack.push(Value::Bool(r == l))
                }
                Gt | Lt => {
                    let r = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let l = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let res = match (l, r) {
                        (Value::Number(l), Value::Number(r)) => match bytecode {
                            Gt => l > r,
                            Lt => l < r,
                            _ => unreachable!(),
                        },
                        _ => {
                            return report_error(
                                chunk.get_line(offset),
                                &bytecode,
                                &format!("Operands must both be numbers, found {l:?}, {r:?}"),
                            )
                        }
                    };
                    vm.stack.push(Value::Bool(res))
                }
            }
        }

        dbg!(vm.stack);

        Ok(())
    }
}
