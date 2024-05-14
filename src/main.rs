mod chunk;
mod value;

use std::env;

use chunk::Chunk;

use crate::chunk::ByteCode;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut chunk = Chunk::default();

    chunk.push(ByteCode::Return, 0);

    let idx = chunk.push_constant(1.0);
    chunk.push(ByteCode::Constant(idx as u8), 0);

    let idx = chunk.push_constant(2.0);
    chunk.push(ByteCode::ConstantLong(idx as u32), 2);

    println!("Line of instruction 0: {:?}", chunk.get_line(0));

    // TODO: Print line numbers for instructions
    chunk.disassemble();
}
