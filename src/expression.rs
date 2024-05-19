use crate::{
    chunk::{ByteCode, Chunk},
    compiler::{
        report_error, report_error_eof, Compiler, CompilerResult, ErrorIgnoreTokenScanner,
        Precedence,
    },
    scanner::{Token, TokenScanner, TokenType},
    util::PrevPeekable,
    value::Value,
    vm::InterpretError,
};

impl<'a> Compiler<'a> {
    fn emit_constant(&mut self, token: &Token, value: Value) {
        let idx = self.chunk.push_constant(value);
        self.chunk
            .push(ByteCode::from_constant_index(idx), token.line)
    }

    fn compile_precedence(&mut self, precedence: Precedence) -> CompilerResult<()> {
        use TokenType::*;
        // Compile token as prefix
        match self.scanner.next() {
            Some(tok) => match tok.ttype {
                LParen => self.compile_grouping(),
                Minus => self.compile_unary(),
                Number => self.compile_number(),
                Str => self.compile_string(),
                False | True | Nil => self.compile_literal(),
                Bang => self.compile_unary(),
                _ => {
                    report_error(&tok, "Expected expression here");
                    Err(InterpretError::Compiler)
                }
            },

            None => {
                report_error_eof("EOF reached");
                Err(InterpretError::Compiler)
            }
        }?;

        // Compile token as infix
        while let Some(tok) = self.scanner.peek() {
            if precedence > Precedence::of(tok.ttype) {
                break;
            }

            match self.scanner.next() {
                Some(tok) => match tok.ttype {
                    Minus | Plus | Slash | Star | EqualEqual | Greater | GreaterEqual | Less
                    | LessEqual => self.compile_binary(),
                    _ => Ok(()),
                },
                None => {
                    report_error_eof("EOF reached");
                    Err(InterpretError::Compiler)
                }
            }?;
        }

        Ok(())
    }

    pub fn compile_expression(&mut self) -> CompilerResult<()> {
        self.compile_precedence(Precedence::Assignment)?;
        Ok(())
    }

    fn compile_number(&mut self) -> CompilerResult<()> {
        let token = self.scanner.prev_unwrap();
        self.emit_constant(&token, Value::Number(token.lexeme.parse().unwrap()));
        Ok(())
    }

    fn compile_string(&mut self) -> CompilerResult<()> {
        let token = self.scanner.prev_unwrap();
        let strlen = token.lexeme.len();
        self.emit_constant(&token, token.lexeme[1..strlen - 1].to_owned().into());
        Ok(())
    }

    fn compile_literal(&mut self) -> CompilerResult<()> {
        use TokenType::*;
        let token = self.scanner.prev_unwrap();
        match token.ttype {
            Nil => self.chunk.push(ByteCode::Nil, token.line),
            True => self.chunk.push(ByteCode::True, token.line),
            False => self.chunk.push(ByteCode::False, token.line),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn compile_unary(&mut self) -> CompilerResult<()> {
        use TokenType::*;
        let op = self.scanner.prev_unwrap();

        // Compile operand
        self.compile_precedence(Precedence::Unary)?;

        match op.ttype {
            Minus => self.chunk.push(ByteCode::Negate, op.line),
            Bang => self.chunk.push(ByteCode::Not, op.line),
            // unreachable
            _ => panic!("Operation {op:?} not handled"),
        }
        Ok(())
    }

    fn compile_binary(&mut self) -> CompilerResult<()> {
        use TokenType::*;
        let op = self.scanner.prev_unwrap();
        self.compile_precedence(Precedence::of(op.ttype).next())?;

        match op.ttype {
            Plus => self.chunk.push(ByteCode::Add, op.line),
            Minus => self.chunk.push(ByteCode::Sub, op.line),
            Star => self.chunk.push(ByteCode::Mul, op.line),
            Slash => self.chunk.push(ByteCode::Div, op.line),

            EqualEqual | BangEqual => self.chunk.push(ByteCode::Eq, op.line),
            Greater | GreaterEqual => self.chunk.push(ByteCode::Gt, op.line),
            Less | LessEqual => self.chunk.push(ByteCode::Lt, op.line),
            _ => panic!("Operation {op:?} not handled"),
        }

        match op.ttype {
            BangEqual | GreaterEqual | LessEqual => self.chunk.push(ByteCode::Not, op.line),
            _ => {}
        }

        Ok(())
    }

    fn compile_grouping(&mut self) -> CompilerResult<()> {
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::RParen, "Expected ')' after expression")?;
        Ok(())
    }
}