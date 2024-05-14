use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ByteCode {
    CONSTANT(u8) = 0,
    RETURN = 1,
}

#[derive(Debug, Default)]
pub struct Chunk {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
}

#[derive(Debug)]
pub struct ChunkIterator<'a> {
    ptr: usize,
    inner: &'a Chunk,
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = (usize, ByteCode);
    type IntoIter = ChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIterator {
            inner: self,
            ptr: 0,
        }
    }
}

impl Chunk {
    pub fn push_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self) {
        println!("== CONSTANTS ==");
        self.constants
            .iter()
            .enumerate()
            .for_each(|(i, v)| println!("{i:#06x}: {v:?}"));
        println!("===============");
        self.into_iter()
            .for_each(|(offset, code)| println!("{offset:#06x}: {code:?}"));
        println!("===============");
    }

    pub fn push_raw(&mut self, raw: &[u8]) {
        self.bytecode.extend_from_slice(raw)
    }

    pub fn push(&mut self, bytecode: ByteCode) {
        match bytecode {
            ByteCode::CONSTANT(index) => self.push_raw(&[0, index]),
            ByteCode::RETURN => self.push_raw(&[1]),
        }
    }
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = (usize, ByteCode);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.inner.bytecode.len() {
            return None;
        }

        let opcode: u8 = self.inner.bytecode[self.ptr];
        let opcode_ptr = self.ptr;
        self.ptr += 1;
        match opcode {
            0 => {
                self.ptr += 1;
                Some((
                    opcode_ptr,
                    ByteCode::CONSTANT(self.inner.bytecode[opcode_ptr + 1]),
                ))
            }
            1 => Some((opcode_ptr, ByteCode::RETURN)),
            _ => None,
        }
    }
}
