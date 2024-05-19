use crate::{
    chunk::ByteCode,
    compiler::{report_error, Compiler, CompilerResult},
    scanner::{Token, TokenType},
    util::PrevPeekable,
    vm::InterpretError,
};

impl<'a> Compiler<'a> {
    //declaration    → varDecl
    //               | statement ;
    //statement      → exprStmt
    //               | printStmt ;

    pub fn compile_decl(&mut self) -> CompilerResult<()> {
        if let Some(_) = self.scanner.advance_if_match(TokenType::Var) {
            self.compile_var_decl()
        } else {
            self.compile_statement()
        }
    }

    fn compile_var_decl(&mut self) -> CompilerResult<()> {
        let tok = self
            .scanner
            .consume_token(TokenType::Ident, "Expected identifier after 'var'")?;

        let name = tok.lexeme;

        let slot = match self.global_bindings.declare_binding(name) {
            Some(slot) => slot,
            None => {
                report_error(&tok, &format!("Variable '{name}' already declared"));
                return Err(InterpretError::Compiler);
            }
        };

        if let Some(_) = self.scanner.advance_if_match(TokenType::Equal) {
            self.compile_expression()?;
        } else {
            self.chunk.push(ByteCode::Nil, tok.line);
        }

        self.scanner
            .consume_token(TokenType::Semi, "Expected ';' after variable declaration")?;

        self.chunk.push(ByteCode::SetGlobal(slot), tok.line);
        self.chunk.push(ByteCode::Pop, tok.line);

        Ok(())
    }

    fn compile_statement(&mut self) -> CompilerResult<()> {
        if let Some(_) = self.scanner.advance_if_match(TokenType::Print) {
            self.compile_print_statement()?;
        } else {
            // Must be an expression statement
            self.compile_expression_statement()?;
            let line = self.scanner.prev_unwrap().line;
            self.chunk.push(ByteCode::Pop, line);
        }

        Ok(())
    }

    fn compile_expression_statement(&mut self) -> CompilerResult<()> {
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::Semi, "Expected ';' after expression")?;
        Ok(())
    }

    fn compile_print_statement(&mut self) -> CompilerResult<()> {
        let line = self.scanner.prev_unwrap().line;
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::Semi, "Expected ';' after value")?;
        self.chunk.push(ByteCode::Print, line);
        Ok(())
    }
}

impl<'a, I> PrevPeekable<I>
where
    I: Iterator<Item = Token<'a>>,
{
    pub fn advance_if_match(&mut self, ttype: TokenType) -> Option<Token<'a>> {
        match self.peek() {
            Some(Token {
                ttype: next_ttype,
                lexeme: _,
                line: _,
            }) => {
                if next_ttype == &ttype {
                    return self.next();
                }
            }
            None => {}
        }
        return None;
    }
}
