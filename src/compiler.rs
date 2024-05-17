use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::{
    chunk::{ByteCode, Chunk},
    scanner::{Token, TokenScanner, TokenType},
    util::PrevPeekable,
    value::Value,
    vm::InterpretError,
};

#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
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

impl Precedence {
    /// Should not be called on Error precedence
    fn next(self) -> Self {
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
            TokenType::String => Precedence::None,
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
fn report_error(token: &Token, msg: &str) {
    println!(
        "Error at line {}, token '{}': {msg}",
        token.line, token.lexeme
    );
}

fn report_error_eof(msg: &str) {
    println!("Error at end of file: {msg}");
}

struct ErrorIgnoreTokenScanner<'a> {
    inner: TokenScanner<'a>,
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
    pub fn consume_token(&mut self, ttype: TokenType, msg: &str) -> Result<Token, InterpretError> {
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

pub struct Compiler<'a> {
    source: &'a str,
    scanner: PrevPeekable<ErrorIgnoreTokenScanner<'a>>,
    // TODO in the future, we will have multiple chunks going at once
    chunk: Chunk,
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
        }
    }

    fn emit_constant(&mut self, token: &Token, value: Value) {
        let idx = self.chunk.push_constant(value);
        self.chunk
            .push(ByteCode::from_constant_index(idx), token.line)
    }

    fn compile_precedence(&mut self, precedence: Precedence) -> Result<(), InterpretError> {
        use TokenType::*;
        // Compile token as prefix
        match self.scanner.next() {
            Some(tok) => match tok.ttype {
                LParen => self.compile_grouping(),
                Minus => self.compile_unary(),
                Number => self.compile_number(),
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

    fn compile_expression(&mut self) -> Result<(), InterpretError> {
        self.compile_precedence(Precedence::Assignment)?;
        Ok(())
    }

    fn compile_number(&mut self) -> Result<(), InterpretError> {
        let token = self.scanner.prev_unwrap();
        self.emit_constant(&token, Value::Number(token.lexeme.parse().unwrap()));
        Ok(())
    }

    fn compile_literal(&mut self) -> Result<(), InterpretError> {
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

    fn compile_unary(&mut self) -> Result<(), InterpretError> {
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

    fn compile_binary(&mut self) -> Result<(), InterpretError> {
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

    fn compile_grouping(&mut self) -> Result<(), InterpretError> {
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::RParen, "Expected ')' after expression")?;
        Ok(())
    }

    pub fn compile(mut self) -> Result<Chunk, InterpretError> {
        self.compile_expression()?;

        self.chunk.disassemble();

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
