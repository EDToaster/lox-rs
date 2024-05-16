use std::iter::Peekable;

use itertools::Itertools;

use crate::{
    chunk::{ByteCode, Chunk},
    scanner::{Token, TokenScanner, TokenType},
    util::PrevPeekable,
    value::Value,
    vm::InterpretError,
};

#[repr(u8)]
enum Precedence {
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

pub struct Compiler<'a> {
    source: &'a str,
    scanner: PrevPeekable<TokenScanner<'a>>,
    // TODO in the future, we will have multiple chunks going at once
    chunk: Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Compiler<'a> {
        Compiler {
            chunk: Chunk::default(),
            source,
            scanner: PrevPeekable::from(TokenScanner::from_source(source)),
        }
    }

    fn report_error(token: &Token, msg: &str) {
        println!(
            "Error at line {}, token '{}': {msg}",
            token.line, token.lexeme
        );
    }

    /// Is the same as scanner.next() except it reports errors
    fn next_token(&mut self) -> Option<Token> {
        let mut takewhile = self
            .scanner
            .take_while_ref(|tok| tok.ttype == TokenType::Error);
        let err = takewhile.next();
        takewhile.last();

        // TODO, bubble this up somewhere higher instead of reporting it from here
        if let Some(err) = err {
            Self::report_error(&err, &format!("Unexpected Token '{}'", err.lexeme));
        }
        self.scanner.next()
    }

    fn consume_token(&mut self, ttype: TokenType, msg: &str) -> Result<Token, InterpretError> {
        if let Some(tok) = self.next_token() {
            if tok.ttype != ttype {
                Self::report_error(&tok, msg);
                return Err(InterpretError::Compiler);
            } else {
                return Ok(tok.clone());
            }
        }
        // TODO: report EOF error
        // self.report_error(&)
        Err(InterpretError::Compiler)
    }

    fn emit_constant(&mut self, token: &Token, value: Value) {
        let idx = self.chunk.push_constant(value);
        self.chunk
            .push(ByteCode::from_constant_index(idx), token.line)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), InterpretError> {}

    fn compile_expression(&mut self) -> Result<(), InterpretError> {
        self.parse_precedence(Precedence::Assignment)?;
        Ok(())
    }

    fn compile_number(&mut self) -> Result<(), InterpretError> {
        let token = self.scanner.prev_unwrap();
        self.emit_constant(&token, token.lexeme.parse().unwrap());
        Ok(())
    }

    fn compile_unary(&mut self) -> Result<(), InterpretError> {
        let op = self.scanner.prev_unwrap();

        // Compile operand
        self.parse_precedence(Precedence::Unary)?;

        match op.ttype {
            TokenType::Minus => self.chunk.push(ByteCode::Negate, op.line),
            // unreachable
            _ => {}
        }
        Ok(())
    }

    fn compile_binary(&mut self) -> Result<(), InterpretError> {
        let op 
    }

    fn compile_grouping(&mut self) -> Result<(), InterpretError> {
        self.compile_expression()?;
        self.consume_token(TokenType::RParen, "Expected ')' after expression");
        Ok(())
    }

    pub fn compile(&mut self) -> Result<Chunk, InterpretError> {
        self.compile_expression()?;

        todo!();

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
