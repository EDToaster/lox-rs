use std::ops::RangeInclusive;

use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ByteCode {
    Return = 0,
    Constant(u8) = 1,
    ConstantLong(u32) = 2,
}

#[derive(Debug, Default)]
pub struct Chunk {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
    // Debug info
    line_info: Vec<(usize, RangeInclusive<usize>)>,
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
    // Debug info

    /// Extend line_info to the current offset
    fn extend_line_info(&mut self, line: usize, offset: usize) {
        if self.line_info.is_empty() {
            self.line_info.push((line, offset..=offset));
            return;
        }

        let last = self.line_info.last_mut().unwrap();
        if last.0 == line {
            last.1 = last.1.start().to_owned()..=offset;
        } else {
            self.line_info.push((line, offset..=offset));
        }
    }

    pub fn get_line(&self, offset: usize) -> Option<usize> {
        self.line_info
            .iter()
            .find(|(_, o)| o.contains(&offset))
            .map(|p| p.0)
    }

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

    fn push_raw_slice(&mut self, raw: &[u8]) {
        self.bytecode.extend_from_slice(raw)
    }

    fn push_raw(&mut self, value: u8) {
        self.bytecode.push(value)
    }

    /// return the offset at the start of the encoded instruction
    pub fn push(&mut self, bytecode: ByteCode, line: usize) {
        let offset = self.bytecode.len();
        match bytecode {
            ByteCode::Return => self.push_raw_slice(&[0]),
            ByteCode::Constant(index) => self.push_raw_slice(&[1, index]),
            ByteCode::ConstantLong(index) => {
                self.push_raw(2);
                self.push_raw_slice(&index.to_le_bytes());
            }
        }
        self.extend_line_info(line, offset);
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
            0 => Some((opcode_ptr, ByteCode::Return)),
            1 => {
                self.ptr += 1;
                Some((
                    opcode_ptr,
                    ByteCode::Constant(self.inner.bytecode[opcode_ptr + 1]),
                ))
            }
            2 => {
                self.ptr += 4;
                Some((
                    opcode_ptr,
                    ByteCode::ConstantLong(u32::from_le_bytes(
                        self.inner.bytecode[opcode_ptr + 1..opcode_ptr + 5]
                            .try_into()
                            .unwrap(),
                    )),
                ))
            }
            _ => None,
        }
    }
}
