mod chunk;
mod value;
mod vm;

use std::env;

use chunk::Chunk;

use crate::{chunk::ByteCode, vm::interpret};

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut chunk = Chunk::default();

    let idx = chunk.push_constant(1.2);
    chunk.push(ByteCode::Constant(idx as u8), 0);

    let idx = chunk.push_constant(3.4);
    chunk.push(ByteCode::Constant(idx as u8), 0);

    chunk.push(ByteCode::Add, 0);

    let idx = chunk.push_constant(5.6);
    chunk.push(ByteCode::Constant(idx as u8), 0);

    chunk.push(ByteCode::Div, 0);
    chunk.push(ByteCode::Negate, 0);

    println!("Line of instruction 0: {:?}", chunk.get_line(0));

    // TODO: Print line numbers for instructions
    chunk.disassemble();

    // Run the VM
    let res = interpret(&chunk);
    println!("{res:?}");
}
