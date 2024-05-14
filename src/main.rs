mod chunk;
mod value;

use std::env;

use chunk::Chunk;

use crate::chunk::ByteCode;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut chunk = Chunk::default();

    chunk.push(ByteCode::RETURN);

    let idx = chunk.push_constant(1.0);
    chunk.push(ByteCode::CONSTANT(idx as u8));
    chunk.disassemble();
}
