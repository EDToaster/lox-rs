use std::collections::{BTreeMap, HashSet};

use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::{
    chunk::Chunk,
    scanner::{Token, TokenScanner, TokenType},
    util::PrevPeekable,
    vm::InterpretError,
};

#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    /// Should not be called on Error precedence
    pub fn next(self) -> Self {
        FromPrimitive::from_u8(self as u8 + 1).unwrap()
    }

    pub fn of(ttype: TokenType) -> Precedence {
        match ttype {
            TokenType::LParen => Precedence::None,
            TokenType::RParen => Precedence::None,
            TokenType::LBrace => Precedence::None,
            TokenType::RBrace => Precedence::None,
            TokenType::Comma => Precedence::None,
            TokenType::Dot => Precedence::None,
            TokenType::Minus => Precedence::Term,
            TokenType::Plus => Precedence::Term,
            TokenType::Semi => Precedence::None,
            TokenType::Slash => Precedence::Factor,
            TokenType::Star => Precedence::Factor,
            TokenType::Bang => Precedence::None,
            TokenType::BangEqual => Precedence::None,
            TokenType::Equal => Precedence::None,
            TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater => Precedence::Comparison,
            TokenType::GreaterEqual => Precedence::Comparison,
            TokenType::Less => Precedence::Comparison,
            TokenType::LessEqual => Precedence::Comparison,
            TokenType::Ident => Precedence::None,
            TokenType::Str => Precedence::None,
            TokenType::Number => Precedence::None,
            TokenType::And => Precedence::None,
            TokenType::Class => Precedence::None,
            TokenType::Else => Precedence::None,
            TokenType::False => Precedence::None,
            TokenType::For => Precedence::None,
            TokenType::Fun => Precedence::None,
            TokenType::If => Precedence::None,
            TokenType::Nil => Precedence::None,
            TokenType::Or => Precedence::None,
            TokenType::Print => Precedence::None,
            TokenType::Return => Precedence::None,
            TokenType::Super => Precedence::None,
            TokenType::This => Precedence::None,
            TokenType::True => Precedence::None,
            TokenType::Var => Precedence::None,
            TokenType::While => Precedence::None,
            TokenType::Error => Precedence::None,
        }
    }
}
pub fn report_error(token: &Token, msg: &str) {
    println!(
        "Error at line {}, token '{}': {msg}",
        token.line, token.lexeme
    );
}

pub fn report_error_eof(msg: &str) {
    println!("Error at end of file: {msg}");
}

pub struct ErrorIgnoreTokenScanner<'a> {
    pub inner: TokenScanner<'a>,
}

impl<'a> Iterator for ErrorIgnoreTokenScanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut takewhile = self
            .inner
            .take_while_ref(|tok| tok.ttype == TokenType::Error);
        let err = takewhile.next();
        takewhile.last();

        // TODO, bubble this up somewhere higher instead of reporting it from here
        if let Some(err) = err {
            report_error(&err, &format!("Unexpected Token '{}'", err.lexeme));
        }
        self.inner.next()
    }
}

impl<'a> PrevPeekable<ErrorIgnoreTokenScanner<'a>> {
    pub fn consume_token(
        &mut self,
        ttype: TokenType,
        msg: &str,
    ) -> Result<Token<'a>, InterpretError> {
        if let Some(tok) = self.next() {
            if tok.ttype != ttype {
                report_error(&tok, msg);
                return Err(InterpretError::Compiler);
            } else {
                return Ok(tok.clone());
            }
        }
        report_error_eof(msg);
        Err(InterpretError::Compiler)
    }
}

pub type CompilerResult<T> = Result<T, InterpretError>;

#[derive(Debug, Default)]
pub struct GlobalBindings<'a> {
    pub global_slots: BTreeMap<&'a str, u32>,
    pub undeclared_globals: HashSet<&'a str>,
}

impl<'a> GlobalBindings<'a> {
    fn next_undeclared_slot(&mut self) -> u32 {
        self.global_slots
            .last_entry()
            .map(|e| e.get() + 1)
            .unwrap_or(0)
    }

    pub fn use_binding(&mut self, name: &'a str) -> u32 {
        let next_idx = self.next_undeclared_slot();
        self.global_slots.get(name).cloned().unwrap_or_else(|| {
            self.global_slots.insert(name, next_idx);
            self.undeclared_globals.insert(name);
            next_idx
        })
    }

    pub fn declare_binding(&mut self, name: &'a str) -> Option<u32> {
        self.undeclared_globals.remove(name);
        if self.global_slots.contains_key(name) {
            // can't redeclare
            None
        } else {
            let next_idx = self.next_undeclared_slot();
            self.global_slots.insert(name, next_idx);
            Some(next_idx)
        }
    }
}

pub struct Compiler<'a> {
    pub source: &'a str,
    pub scanner: PrevPeekable<ErrorIgnoreTokenScanner<'a>>,
    // TODO in the future, we will have multiple chunks going at once
    pub chunk: Chunk,

    pub global_bindings: GlobalBindings<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Compiler<'a> {
        let scanner = PrevPeekable::from(ErrorIgnoreTokenScanner {
            inner: TokenScanner::from_source(source),
        });
        Compiler {
            chunk: Chunk::default(),
            source,
            scanner,
            global_bindings: GlobalBindings::default(),
        }
    }

    pub fn compile(mut self) -> CompilerResult<Chunk> {
        // self.compile_expression()?;

        while let Some(_) = self.scanner.peek() {
            self.compile_declaration()?;
        }

        self.chunk.disassemble();

        if !self.global_bindings.undeclared_globals.is_empty() {
            report_error_eof("{num_slots} global bindings were not declared, but are used");
            return Err(InterpretError::Compiler);
        }
        // TODO: safe convert
        self.chunk.global_slots = self.global_bindings.global_slots.keys().count() as u32;
        if let Some(t) = self.scanner.peek() {
            report_error(t, "Expected EOF");
            Err(InterpretError::Compiler)
        } else {
            Ok(self.chunk)
        }

        // self.check_eof()?;

        // Tmp debug
        // println!("Line Token");
        // let mut prev_line = 0;
        // while let Some(token) = scanner.next() {
        //     if token.line != prev_line {
        //         prev_line = token.line;
        //         print!("{prev_line: >4} ");
        //     } else {
        //         print!("   | ");
        //     }
        //     println!("{token:?}");
        // }
        // Ok(())
    }
}
