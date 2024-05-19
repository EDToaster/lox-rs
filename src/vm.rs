

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
    pub globals: Vec<Value>,
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
            globals: vec![Value::Nil; chunk.global_slots as usize],
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
                True => vm.stack.push(true.into()),
                False => vm.stack.push(false.into()),
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

                    vm.stack.push(val.into());
                }
                Add | Sub | Mul | Div => {
                    let r = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let l = vm.stack.pop().ok_or(InterpretError::Runtime)?;

                    let res = match (bytecode, l, r) {
                        (Add, Value::Number(l), Value::Number(r)) => (l + r).into(),
                        (Sub, Value::Number(l), Value::Number(r)) => (l - r).into(),
                        (Mul, Value::Number(l), Value::Number(r)) => (l * r).into(),
                        (Div, Value::Number(l), Value::Number(r)) => (l / r).into(),
                        (Add, Value::Str(l), Value::Str(r)) => format!("{l}{r}").into(),
                        (Mul, Value::Str(l), Value::Number(r)) if r.fract() == 0.0 => {
                            l.repeat(r as usize).into()
                        },
                        (_, l, r) => 
                            return report_error(
                                chunk.get_line(offset),
                                &bytecode,
                                &format!("Unsupported operands for operation {bytecode:?}, found {l:?}, {r:?}"),
                            )
                    };

                    vm.stack.push(res);
                }
                Not => {
                    let val = !vm.stack.pop().ok_or(InterpretError::Runtime)?.is_truthy();
                    vm.stack.push(val.into());
                }
                Eq => {
                    let r = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    let l = vm.stack.pop().ok_or(InterpretError::Runtime)?;
                    vm.stack.push((r == l).into())
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
                        (l, r) => {
                            return report_error(
                                chunk.get_line(offset),
                                &bytecode,
                                &format!("Operands must both be numbers, found {l:?}, {r:?}"),
                            )
                        }
                    };
                    vm.stack.push(res.into())
                }
                Print => {
                    println!("{}", vm.stack.pop().ok_or(InterpretError::Runtime)?);
                }
                SetGlobal(slot) => {
                    let val = vm.stack.last().ok_or(InterpretError::Runtime)?.clone();
                    vm.globals[slot as usize] = val;
                }
                GetGlobal(slot) => {
                    let val = vm.globals[slot as usize].clone();
                    vm.stack.push(val);
                }
                Pop => {
                    vm.stack.pop().ok_or(InterpretError::Runtime)?;
                },
            }
        }

        dbg!(vm.stack);
        dbg!(vm.globals);

        Ok(())
    }
}
