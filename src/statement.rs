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
            self.compile_var_decl(true)
        } else if let Some(_) = self.scanner.advance_if_match(TokenType::Val) {
            self.compile_var_decl(false)
        } else {
            self.compile_statement()
        }
    }

    fn compile_var_decl(&mut self, mutable: bool) -> CompilerResult<()> {
        let tok = self
            .scanner
            .consume_token(TokenType::Ident, "Expected identifier after 'var'")?;

        let name = tok.lexeme;

        // Compile expression if needed
        if let Some(_) = self.scanner.advance_if_match(TokenType::Equal) {
            self.compile_expression()?;
        } else {
            self.chunk.push(ByteCode::Nil, tok.line);
        }

        // Then, allocate global or local variable. We do this after compiling subexpression so that
        // the following can work:
        //   var a = "hello";
        //   { var a = a + ", world!"; }
        let slot = if self.scope.depth > 0 {
            // local
            if !self.scope.add_local(tok.clone(), mutable) {
                report_error(
                    &tok,
                    &format!("Cannot redeclare variable '{name}' in the same scope"),
                );
                return Err(InterpretError::Compiler);
            }
            // We dont actually care here
            0u32
        } else {
            // global
            if !mutable {
                report_error(&tok, "Immutable global variables are not allowed");
                return Err(InterpretError::Compiler);
            }
            match self.global_bindings.declare_binding(name) {
                Some(slot) => slot,
                None => {
                    report_error(&tok, &format!("Variable '{name}' already declared"));
                    return Err(InterpretError::Compiler);
                }
            }
        };

        self.scanner
            .consume_token(TokenType::Semi, "Expected ';' after variable declaration")?;

        if self.scope.depth == 0 {
            self.chunk.push(ByteCode::SetGlobal(slot), tok.line);
            self.chunk.push(ByteCode::Pop, tok.line);
        }

        Ok(())
    }

    fn compile_statement(&mut self) -> CompilerResult<()> {
        if let Some(_) = self.scanner.advance_if_match(TokenType::Print) {
            self.compile_print_statement()?;
        } else if let Some(t) = self.scanner.advance_if_match(TokenType::LBrace) {
            self.scope.increment_depth();
            self.compile_block()?;
            let num_locals = self.scope.decrement_depth();
            for _ in 0..num_locals {
                self.chunk.push(ByteCode::Pop, t.line);
            }
        } else {
            // Must be an expression statement
            self.compile_expression_statement()?;
            let line = self.scanner.prev_unwrap().line;
            self.chunk.push(ByteCode::Pop, line);
        }

        Ok(())
    }

    fn compile_block(&mut self) -> CompilerResult<()> {
        while let Some(t) = self.scanner.peek() {
            if t.ttype == TokenType::RBrace {
                break;
            }
            self.compile_decl()?;
        }

        self.scanner
            .consume_token(TokenType::RBrace, "Expected '}' after block")?;
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
