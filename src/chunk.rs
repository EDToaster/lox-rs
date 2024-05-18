use itertools::Itertools;
use num_traits::ToPrimitive;

use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ByteCode {
    Return = 0,
    Constant(u8) = 1,
    ConstantLong(u32) = 2,

    // Literals
    Nil = 3,
    True = 4,
    False = 5,

    // Arith
    Negate = 0x10,
    Add,
    Sub,
    Mul,
    Div,

    // Bool
    Not = 0x20,
    Eq,
    Gt,
    Lt,
}

impl ByteCode {
    pub fn from_constant_index(index: u32) -> ByteCode {
        u8::try_from(index)
            .map(|idx| Self::Constant(idx))
            .unwrap_or(Self::ConstantLong(index))
    }
}

#[derive(Debug)]
pub struct Chunk {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
    // Vec of line number to start
    line_info: Vec<(usize, usize)>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            bytecode: vec![],
            constants: vec![],
            line_info: vec![(0, 0)],
        }
    }
}

#[derive(Debug)]
pub struct ChunkIterator<'a> {
    pub ptr: usize,
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
        // unwrap since the vec is always non-empty
        let &(last_line, _) = self.line_info.last().unwrap();

        if last_line != line {
            self.line_info.push((line, offset));
        }
    }

    pub fn get_line(&self, offset: usize) -> usize {
        self.line_info
            .iter()
            .take_while_ref(|(_, o)| o <= &offset)
            .last()
            .map(|l| l.1)
            .unwrap_or(0)
    }

    pub fn push_constant(&mut self, value: Value) -> u32 {
        self.constants.push(value);
        // TODO, do safe casting
        (self.constants.len() as u32) - 1
    }

    pub fn get_constant(&self, idx: u32) -> Value {
        // TODO: remove clone since we wouldn't want to clone a str
        self.constants[idx as usize].clone()
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
        use ByteCode::*;
        let offset = self.bytecode.len();
        match bytecode {
            Return => self.push_raw(0),
            Constant(index) => self.push_raw_slice(&[1, index]),
            ConstantLong(index) => {
                self.push_raw(2);
                self.push_raw_slice(&index.to_le_bytes());
            }
            Nil => self.push_raw(3),
            True => self.push_raw(4),
            False => self.push_raw(5),
            Negate => self.push_raw(0x10),
            Add => self.push_raw(0x11),
            Sub => self.push_raw(0x12),
            Mul => self.push_raw(0x13),
            Div => self.push_raw(0x14),

            Not => self.push_raw(0x20),
            Eq => self.push_raw(0x21),
            Gt => self.push_raw(0x22),
            Lt => self.push_raw(0x23),
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
            3 => Some((opcode_ptr, ByteCode::Nil)),
            4 => Some((opcode_ptr, ByteCode::True)),
            5 => Some((opcode_ptr, ByteCode::False)),
            0x10 => Some((opcode_ptr, ByteCode::Negate)),
            0x11 => Some((opcode_ptr, ByteCode::Add)),
            0x12 => Some((opcode_ptr, ByteCode::Sub)),
            0x13 => Some((opcode_ptr, ByteCode::Mul)),
            0x14 => Some((opcode_ptr, ByteCode::Div)),
            0x20 => Some((opcode_ptr, ByteCode::Not)),
            0x21 => Some((opcode_ptr, ByteCode::Eq)),
            0x22 => Some((opcode_ptr, ByteCode::Gt)),
            0x23 => Some((opcode_ptr, ByteCode::Lt)),
            _ => None,
        }
    }
}
