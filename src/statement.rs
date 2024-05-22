use itertools::Itertools;

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
        if self.scanner.advance_if_match(TokenType::Print).is_some() {
            self.compile_print_statement()?;
        } else if self.scanner.advance_if_match(TokenType::If).is_some() {
            self.compile_if_statement()?;
        } else if self.scanner.advance_if_match(TokenType::While).is_some() {
            self.compile_while_statement()?;
        } else if self.scanner.advance_if_match(TokenType::For).is_some() {
            self.compile_for_statement()?;
        } else if self.scanner.advance_if_match(TokenType::Match).is_some() {
            self.compile_match_statement()?;
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

    fn compile_if_statement(&mut self) -> CompilerResult<()> {
        let line = self.scanner.prev_unwrap().line;
        //   condition
        //   jz else
        //   pop
        //   true_branch
        //   jump end
        // else:
        //   pop
        //   false_branch
        // end:

        self.scanner
            .consume_token(TokenType::LParen, "Expected '(' after if")?;
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::RParen, "Expected ')' after condition")?;

        // Jump if false
        let else_label = self.chunk.allocate_new_label();
        let end_label = self.chunk.allocate_new_label();

        self.chunk
            .push_monkey_patch(ByteCode::JumpF(0), line, else_label);
        self.chunk.push(ByteCode::Pop, line);
        self.compile_statement()?;
        self.chunk
            .push_monkey_patch(ByteCode::JumpOffset(0), line, end_label);
        self.chunk.push_label(else_label);
        self.chunk.push(ByteCode::Pop, line);

        if let Some(_) = self.scanner.advance_if_match(TokenType::Else) {
            self.compile_statement()?;
        }

        self.chunk.push_label(end_label);

        Ok(())
    }

    fn compile_while_statement(&mut self) -> CompilerResult<()> {
        // cond:
        //   cond
        //   jump_f .end
        //   pop
        //   body
        //   jump .cond
        // end:
        //   pop

        let line = self.scanner.prev_unwrap().line;

        let cond_label = self.chunk.allocate_new_label();
        let end_label = self.chunk.allocate_new_label();

        self.scanner
            .consume_token(TokenType::LParen, "Expected '(' after while")?;
        self.chunk.push_label(cond_label);
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::RParen, "Expected ')' after condition")?;

        self.chunk
            .push_monkey_patch(ByteCode::JumpF(0), line, end_label);
        self.chunk.push(ByteCode::Pop, line);

        // compile body and jump back to cond
        self.compile_statement()?;
        self.chunk
            .push_monkey_patch(ByteCode::JumpOffset(0), line, cond_label);

        self.chunk.push_label(end_label);
        self.chunk.push(ByteCode::Pop, line);
        Ok(())
    }

    fn compile_for_statement(&mut self) -> CompilerResult<()> {
        //   init
        // cond:
        //   cond
        //   jump_f .end
        //   jump .body
        // post:
        //   post
        //   pop
        //   jump .cond
        // body:
        //   pop
        //   body
        //   jump .post
        // end:
        //   pop

        let line = self.scanner.prev_unwrap().line;

        let cond_label = self.chunk.allocate_new_label();
        let post_label = self.chunk.allocate_new_label();
        let body_label = self.chunk.allocate_new_label();
        let end_label = self.chunk.allocate_new_label();

        self.scanner
            .consume_token(TokenType::LParen, "Expected '(' after 'for'")?;

        // ';' or decl
        if self.scanner.advance_if_match(TokenType::Semi).is_none() {
            self.compile_decl()?;
        }
        // ';' or cond
        self.chunk.push_label(cond_label);
        if let Some(t) = self.scanner.advance_if_match(TokenType::Semi) {
            self.chunk.push(ByteCode::True, t.line);
        } else {
            self.compile_expression()?;
            self.scanner
                .consume_token(TokenType::Semi, "Expected ';' after for condition")?;
        }
        self.chunk
            .push_monkey_patch(ByteCode::JumpF(0), line, end_label);
        self.chunk
            .push_monkey_patch(ByteCode::JumpOffset(0), line, body_label);

        // ')' or post
        self.chunk.push_label(post_label);
        if self.scanner.advance_if_match(TokenType::RParen).is_none() {
            self.compile_expression()?;
            self.chunk.push(ByteCode::Pop, line);
            self.scanner
                .consume_token(TokenType::RParen, "Expected ')' after for")?;
        }
        self.chunk
            .push_monkey_patch(ByteCode::JumpOffset(0), line, cond_label);

        // Body
        self.chunk.push_label(body_label);
        self.chunk.push(ByteCode::Pop, line);
        self.compile_statement()?;
        self.chunk
            .push_monkey_patch(ByteCode::JumpOffset(0), line, post_label);

        self.chunk.push_label(end_label);
        self.chunk.push(ByteCode::Pop, line);

        Ok(())
    }

    fn compile_match_statement(&mut self) -> CompilerResult<()> {
        //   match_expr
        // branch_1:
        // expr_1:
        //   dup
        //   expr_1
        //   eq
        //   not
        //   jz .statement_a
        //   pop
        // expr_2:
        //   dup
        //   expr_2
        //   eq
        //   not
        //   jz .statement_a
        //   pop
        //
        //   jump .branch_2
        // statement_a:
        //   pop
        //   statement_a
        //   jump .end
        //
        // branch_2:
        // ...
        //
        // branch_n:
        // end:
        //   pop

        let line = self.scanner.prev_unwrap().line;

        self.scanner
            .consume_token(TokenType::LParen, "Expected '(' after match")?;
        self.compile_expression()?;
        self.scanner
            .consume_token(TokenType::RParen, "Expected ')' after match expression")?;

        let end_label = self.chunk.allocate_new_label();
        let mut next_branch = self.chunk.allocate_new_label();

        self.scanner
            .consume_token(TokenType::LBrace, "Expected '{' after match expression")?;

        while let None = self.scanner.advance_if_match(TokenType::RBrace) {
            let this_statement = self.chunk.allocate_new_label();

            self.chunk.push_label(next_branch);
            next_branch = self.chunk.allocate_new_label();

            // match each condition
            loop {
                if self.scanner.advance_if_match(TokenType::Else).is_some() {
                    // it doesn't matter... it gets popped off the stack
                    self.chunk.push(ByteCode::Dup, line);
                    self.chunk
                        .push_monkey_patch(ByteCode::JumpOffset(0), line, this_statement);
                    break;
                }

                self.chunk.push(ByteCode::Dup, line);
                self.compile_expression()?;
                self.chunk.push(ByteCode::Eq, line);
                self.chunk.push(ByteCode::Not, line);
                self.chunk
                    .push_monkey_patch(ByteCode::JumpF(0), line, this_statement);
                self.chunk.push(ByteCode::Pop, line);

                if self.scanner.advance_if_match(TokenType::Bar).is_none() {
                    break;
                }
            }

            self.scanner
                .consume_token(TokenType::FatArrow, "Expected '=>' after match conditions")?;

            // Compile branches
            self.chunk
                .push_monkey_patch(ByteCode::JumpOffset(0), line, next_branch);

            self.chunk.push_label(this_statement);
            self.chunk.push(ByteCode::Pop, line);
            self.compile_statement()?;
            self.chunk
                .push_monkey_patch(ByteCode::JumpOffset(0), line, end_label);
        }

        self.chunk.push_label(end_label);
        self.chunk.push_label(next_branch);
        self.chunk.push(ByteCode::Pop, line);

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
