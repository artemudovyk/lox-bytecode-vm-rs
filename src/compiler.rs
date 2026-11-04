use crate::{
    Chunk, OpCode,
    lexer::{Keyword, Lexer, Literal, Token, TokenKind},
    value::Value,
};

pub fn compile(src: &str) -> Result<Chunk, CompileError> {
    let mut compiler = Compiler::new(src);
    compiler.run()?;
    Ok(compiler.chunk)
}

#[allow(dead_code)]
pub struct Compiler<'src> {
    chunk: Chunk,
    lexer: Lexer<'src>,
    in_class: CurrentClass<'src>,
    cur_func: Option<&'src str>,
}

#[allow(dead_code)]
impl<'src> Compiler<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            chunk: Chunk::new(),
            lexer: Lexer::new(src),
            in_class: CurrentClass::None,
            cur_func: None,
        }
    }

    fn run(&mut self) -> Result<(), CompileError> {
        while self.next_stmt()? {}
        Ok(())
    }

    fn assert_semicolon(&mut self) {
        assert!(
            matches!(self.lexer.next_token().kind, TokenKind::Semicolon),
            "Expect ';' at the end of the statement."
        );
    }

    fn expect_token(
        &mut self,
        token_kind: TokenKind<'src>,
        msg: &str,
    ) -> Result<Token<'src>, CompileError> {
        let token = self.lexer.next_token();
        if token.kind != token_kind {
            return Err(CompileError::ExpectedTokenNotFound(format!(
                "Error at '{}': {}",
                token.lexeme, msg
            )));
        }
        Ok(token)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt<'src>>, CompileError> {
        let mut statements = vec![];
        loop {
            match self.lexer.peek().kind {
                TokenKind::RightBrace => {
                    self.lexer.advance();
                    break;
                }
                // TODO: not sure about that, but I'll leave it here for a time being
                // need to think about a better way to check if block is properly closed
                TokenKind::Eof => {
                    return Err(CompileError::UnclosedBlock {
                        line: self.current_line(),
                    });
                }
                TokenKind::Comment => {
                    self.lexer.advance();
                }
                // Treat all identifiers in the block as function declarations if we are inside the
                // class declaration.
                TokenKind::Ident if self.in_class.in_scope() && self.cur_func.is_none() => {
                    statements.push(self.parse_function()?)
                }
                // _ => statements.push(
                //     self.next_stmt()?
                //         .expect("Statement is expected at this point"),
                // ),
                _ => todo!("Unhandled token kind while parsing block."),
            }
        }
        Ok(statements)
    }

    fn parse_function(&mut self) -> Result<Stmt<'src>, CompileError> {
        let ident = self
            .expect_token(TokenKind::Ident, "Expect identifier.")?
            .lexeme;
        let prev_func = self.cur_func;
        self.cur_func = Some(ident);
        // TODO: why it fails in my last test?
        self.expect_token(TokenKind::LeftParen, "Expect a '('.")?;
        let mut params = vec![];
        while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
            let t = self.lexer.next_token();
            match t.kind {
                TokenKind::Ident => {
                    params.push(t.lexeme);
                    if matches!(self.lexer.peek().kind, TokenKind::Ident) {
                        return Err(CompileError::MissingCommaBetweenParameters);
                    }
                }
                TokenKind::Comma => continue,
                _ => panic!("Unexpected token"),
            }
        }
        self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
        self.expect_token(TokenKind::LeftBrace, "Expect '{'.")?;
        let body = self.parse_block()?;
        if self.in_class.in_scope()
            && self.cur_func.is_some_and(|f| f == "init")
            && body.iter().any(|stmt| match stmt {
                Stmt::Return(Some(Expr::Atom(Atom::Nil))) => false,
                Stmt::Return(Some(_)) => true,
                _ => false,
            })
        {
            return Err(CompileError::InvalidInitializerReturn);
        }
        self.cur_func = prev_func;
        Ok(Stmt::FunDecl {
            ident,
            parameters: params,
            body,
        })
    }

    pub fn next_stmt(&mut self) -> Result<bool, CompileError> {
        #[allow(clippy::never_loop)]
        loop {
            #[allow(clippy::match_single_binding)]
            match &self.lexer.peek().kind {
                // TokenKind::Keyword(Keyword::Print) => {
                //     self.lexer.advance(); // Consume `print` keyword
                //     let expr = self.next_expr()?;
                //     self.expect_token(TokenKind::Semicolon, "Expect expression.")?;
                //     return Ok(true);
                // }
                // TokenKind::Keyword(Keyword::Var) => {
                //     self.lexer.advance(); // Consume `var` keyword
                //     let ident = self.expect_token(TokenKind::Ident, "Expect identifier.")?;
                //     let init_val = match self.lexer.peek().kind {
                //         TokenKind::Eq => {
                //             self.lexer.advance();
                //             self.next_expr()?
                //         }
                //         TokenKind::Semicolon => Expr::Atom(Atom::Nil),
                //         _ => Err(CompileError::UnexpectedToken {
                //             line: self.current_line(),
                //             lexeme: self.lexer.peek().lexeme.to_owned(),
                //         })?,
                //     };
                //     self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                //     return Ok(Some(Stmt::Var {
                //         name: ident.lexeme,
                //         init_val,
                //     }));
                // }
                // TokenKind::Keyword(Keyword::If) => {
                //     self.lexer.advance();
                //     self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                //     let cond = self.next_expr()?;
                //     self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                //     let stmt = match self.lexer.peek().kind {
                //         TokenKind::LeftBrace => {
                //             self.lexer.advance();
                //             let statements = self.parse_block()?;
                //             Box::new(Stmt::Block(statements))
                //         }
                //         _ => match self.next_stmt()? {
                //             Some(s) => Box::new(s),
                //             None => Err(CompileError::EmptyIfStatement {
                //                 line: self.current_line(),
                //             })?,
                //         },
                //     };
                //     let else_stmt = match self.lexer.peek().kind {
                //         TokenKind::Keyword(Keyword::Else) => {
                //             self.lexer.advance();
                //             match self.lexer.peek().kind {
                //                 TokenKind::LeftBrace => {
                //                     self.lexer.advance();
                //                     let statements = self.parse_block()?;
                //                     Some(Box::new(Stmt::Block(statements)))
                //                 }
                //                 _ => self.next_stmt()?.map(Box::new),
                //             }
                //         }
                //         _ => None,
                //     };
                //     return Ok(Some(Stmt::If {
                //         condition: cond,
                //         stmt,
                //         else_stmt,
                //     }));
                // }
                // TokenKind::Keyword(Keyword::While) => {
                //     self.lexer.advance();
                //     self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                //     let cond = self.next_expr()?;
                //     self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                //     let body = match self.lexer.peek().kind {
                //         TokenKind::LeftBrace => {
                //             self.lexer.advance();
                //             let statements = self.parse_block()?;
                //             Box::new(Stmt::Block(statements))
                //         }
                //         _ => match self.next_stmt()? {
                //             Some(s) => Box::new(s),
                //             None => Err(CompileError::EmptyWhileStatement {
                //                 line: self.current_line(),
                //             })?,
                //         },
                //     };
                //     return Ok(Some(Stmt::While {
                //         condition: cond,
                //         body,
                //     }));
                // }
                // TokenKind::Keyword(Keyword::For) => {
                //     self.lexer.advance();
                //     self.expect_token(TokenKind::LeftParen, "Expect '('.")?;
                //     let initializer = match self.lexer.peek().kind {
                //         TokenKind::Semicolon => {
                //             self.lexer.advance();
                //             None
                //         }
                //         _ => Some(self.next_stmt()?.expect("Expect a statement")),
                //     };
                //     let cond = self.next_expr()?;
                //     self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                //     let increment = match self.lexer.peek().kind {
                //         TokenKind::RightParen => None,
                //         _ => Some(self.next_expr()?),
                //     };
                //     self.expect_token(TokenKind::RightParen, "Expect ')'.")?;
                //     let body = match self.lexer.peek().kind {
                //         TokenKind::LeftBrace => {
                //             self.lexer.advance();
                //             let statements = self.parse_block()?;
                //             Box::new(Stmt::Block(statements))
                //         }
                //         _ => match self.next_stmt()? {
                //             Some(s) => Box::new(s),
                //             // TODO: change to for-related error
                //             None => Err(CompileError::EmptyWhileStatement {
                //                 line: self.current_line(),
                //             })?,
                //         },
                //     };
                //     return Ok(Some(Stmt::For {
                //         initializer: initializer.map(Box::new),
                //         condition: cond,
                //         increment,
                //         body,
                //     }));
                // }
                // TokenKind::Keyword(Keyword::Fun) => {
                //     self.lexer.advance(); // step over `fun` keyword
                //     let func = self.parse_function()?;
                //     return Ok(Some(func));
                // }
                // TokenKind::Keyword(Keyword::Class) => {
                //     self.lexer.advance(); // step over `class` keyword
                //     let ident = self
                //         .expect_token(TokenKind::Ident, "Expect class name identifier.")?
                //         .lexeme;
                //     self.in_class = CurrentClass::Class(ident);
                //
                //     let superclass = match self.lexer.peek().kind {
                //         TokenKind::Less => {
                //             self.lexer.advance();
                //             let sc_name = self
                //                 .expect_token(
                //                     TokenKind::Ident,
                //                     "Expect superclass name identifier.",
                //                 )?
                //                 .lexeme;
                //
                //             if ident == sc_name {
                //                 return Err(CompileError::InheritFromItselfNotAllowed);
                //             }
                //
                //             self.in_class = CurrentClass::Subclass(ident);
                //
                //             Some(sc_name)
                //         }
                //         _ => None,
                //     };
                //
                //     self.expect_token(TokenKind::LeftBrace, "Expect a '{' after the class name.")?;
                //     let methods = self.parse_block()?;
                //     self.in_class = CurrentClass::None;
                //     return Ok(Some(Stmt::ClassDecl {
                //         ident,
                //         methods,
                //         superclass,
                //     }));
                // }
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Return => {
                        self.lexer.advance();
                        match self.lexer.peek().kind {
                            TokenKind::Semicolon => {
                                self.lexer.advance();
                                self.emit(OpCode::Return);
                                return Ok(false);
                            }
                            _ => {
                                self.emit_expr(0)?;
                                self.expect_token(
                                    TokenKind::Semicolon,
                                    "Expect ';' at the end of the return statement.",
                                )?;
                                self.emit(OpCode::Return);
                                return Ok(false);
                            }
                        }
                    }
                    _ => todo!("Unhandled keyword while parsing next statement."),
                },
                // TokenKind::LeftBrace => {
                //     self.lexer.advance();
                //     return Ok(Some(Stmt::Block(self.parse_block()?)));
                // }
                // // Consume and ignore comments
                TokenKind::Comment => {
                    self.lexer.advance();
                    continue;
                }
                TokenKind::Eof => return Ok(false),
                // Fallback to expression statement;
                _ => {
                    // let expr = self.expr_bp(0)?;
                    self.emit_expr(0)?;
                    self.expect_token(TokenKind::Semicolon, "Expect ';'.")?;
                    // self.emit_expr(&expr);
                    return Ok(true);
                }
            }
        }
    }

    // fn emit_expr(&mut self, expr: &Expr<'_>) {
    //     match expr {
    //         Expr::Atom(atom) => match atom {
    //             Atom::Number(n) => {
    //                 let idx = self.chunk.push_constant(Value::new(*n));
    //                 self.chunk.push(OpCode::Constant, self.lexer.current_line());
    //                 self.chunk.push(idx, self.lexer.current_line());
    //             }
    //             _ => todo!(),
    //         },
    //         Expr::Cons(op, exprs) => {
    //             for expr in exprs {
    //                 self.emit_expr(expr);
    //             }
    //             match op {
    //                 Op::Add => self.chunk.push(OpCode::Add, self.lexer.current_line()),
    //                 Op::Minus => match exprs.len() {
    //                     1 => self.chunk.push(OpCode::Negate, self.lexer.current_line()),
    //                     2 => self.chunk.push(OpCode::Sub, self.lexer.current_line()),
    //                     _ => panic!("Unexpected number of args"),
    //                 },
    //                 Op::Mult => self.chunk.push(OpCode::Mult, self.lexer.current_line()),
    //                 Op::Div => self.chunk.push(OpCode::Div, self.lexer.current_line()),
    //                 _ => todo!(),
    //             }
    //         }
    //     }
    // }

    fn emit<T: Into<u8>>(&mut self, code: T) {
        self.chunk.push(code.into(), self.lexer.current_line());
    }

    fn emit_constant(&mut self, val: Value) {
        let idx = self.chunk.push_constant(val);
        self.emit(OpCode::Constant);
        self.emit(idx);
    }

    pub fn next_expr(&mut self) -> Result<Expr<'src>, CompileError> {
        self.expr_bp(0)
    }

    pub fn emit_expr(&mut self, min_bp: u8) -> Result<(), CompileError> {
        let token = self.lexer.next_token();
        match token.kind {
            TokenKind::Literal(literal) => match literal {
                Literal::Number(n) => self.emit_constant(Value::Number(n)),
                _ => todo!("Handle other literal types."),
            },
            TokenKind::Keyword(keyword) => match keyword {
                Keyword::True => self.emit(OpCode::True),
                Keyword::False => self.emit(OpCode::False),
                Keyword::Nil => self.emit(OpCode::Nil),
                _ => todo!("Handle other keywords."),
            },
            TokenKind::LeftParen => {
                self.emit_expr(0)?;
                self.expect_token(
                    TokenKind::RightParen,
                    "Expect ')' at the end of the grouping.",
                )?;
            }
            // Only operators that can be used as prefix are expected at that point
            _ if let Ok(op) = Op::try_from(&token)
                && let Some(bp) = op.prefix_bp()
                && let Some(op_code) = op.prefix_as_op_code() =>
            {
                self.emit_expr(bp)?;
                self.emit(op_code);
            }
            _ => todo!("Handle other token kinds for lhs."),
        }

        // let mut lhs = {
        //     if let Ok(atom) = Atom::try_from(&token) {
        //         Expr::Atom(atom)
        //         // TODO: change later to support non-constant atoms
        //         self.emit_constant(atom)
        //     } else if matches!(token.kind, TokenKind::Keyword(Keyword::This)) {
        //         if self.in_class.in_scope() {
        //             Expr::Atom(Atom::Ident {
        //                 name: "this",
        //                 // loc: Loc {
        //                 //     line: self.current_line(),
        //                 //     // TODO: placeholder
        //                 //     col: 0,
        //                 // },
        //             })
        //         } else {
        //             return Err(CompileError::InvalidThisUsage);
        //         }
        //     } else if matches!(token.kind, TokenKind::Keyword(Keyword::Super)) {
        //         match self.in_class {
        //             CurrentClass::Subclass(_) => {
        //                 match self.lexer.peek().kind {
        //                     TokenKind::Dot => Expr::Atom(Atom::Ident {
        //                         name: "super",
        //                         // loc: Loc {
        //                         //     line: self.current_line(),
        //                         //     // TODO: placeholder
        //                         //     col: 0,
        //                         // },
        //                     }),
        //                     _ => return Err(CompileError::SuperMustBeFollowedByGetExpression),
        //                 }
        //             }
        //             CurrentClass::Class(_) => {
        //                 return Err(CompileError::SuperInClassWithNoSuperclassNotAllowed);
        //             }
        //             CurrentClass::None => return Err(CompileError::SuperOutsideClassNotAllowed),
        //         }
        //     } else if token.unterminated() {
        //         return Err(CompileError::UnterminatedString {
        //             line: token.line,
        //             lexeme: token.lexeme.to_owned(),
        //         });
        //     } else if matches!(token.kind, TokenKind::LeftParen) {
        //         let members = self.emit_expr(0)?;
        //         {
        //             let t = self.lexer.next_token();
        //             if !matches!(t.kind, TokenKind::RightParen) {
        //                 return Err(CompileError::UnmatchedParen {
        //                     line: t.line,
        //                     lexeme: t.lexeme.to_owned(),
        //                 });
        //             }
        //         }
        //         Expr::Cons(Op::Group, vec![members])
        //     } else if let Ok(op) = Op::try_from(&token)
        //         && let Some(bp) = op.prefix_bp()
        //     {
        //         let rhs = self.emit_expr(bp)?;
        //         Expr::Cons(op, vec![rhs])
        //     } else {
        //         return Err(CompileError::UnexpectedToken {
        //             line: token.line,
        //             lexeme: token.lexeme.to_owned(),
        //         });
        //     }
        // };

        loop {
            let peeked = self.lexer.peek();

            if let Ok(op) = Op::try_from(peeked) {
                if let Some(l_bp) = op.postfix_bp()
                    && let Some(op_code) = op.postfix_as_op_code()
                {
                    if l_bp < min_bp {
                        break;
                    }
                    self.lexer.advance();
                    if op == Op::Call {
                        // let mut args = vec![lhs]; // function name should be the first vec member
                        let mut args = vec![];
                        while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
                            args.push(self.next_expr()?);
                            if matches!(self.lexer.peek().kind, TokenKind::Comma) {
                                self.lexer.advance(); // step over comma
                                continue;
                            }
                            if args.len() >= 255 {
                                return Err(CompileError::TooManyArguments);
                            }
                        }
                        self.lexer.advance(); // step over right paren
                        // Expr::Cons(Op::Call, args);
                    };
                    self.emit(op_code);
                    // Continue to check the next operator
                    continue;
                }

                if let Some((l_bp, r_bp)) = op.infix_bp()
                    && let Some(op_code) = op.infix_as_op_code()
                {
                    // If the next operator's left binding power is less than minimum, it is not strong
                    // enough to bind the next atom, so we break out of the loop with everything
                    // emitted so far as a bytecode and give up the execution back to caller.
                    if l_bp < min_bp {
                        break;
                    }
                    // Otherwise, we advance with the rest of the expression.
                    self.lexer.advance();
                    self.emit_expr(r_bp)?;
                    self.emit(op_code);
                    // Continue to check the next operator
                    continue;
                }
            } else if matches!(peeked.kind, TokenKind::RightParen)
                || peeked.is_eof()
                || matches!(peeked.kind, TokenKind::Comma)
            {
                break;
            }
            // else if matches!(peeked.kind, TokenKind::LeftParen) {
            //     // Parse a function call
            //     let mut args = vec![lhs];
            //     while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
            //         args.push(self.next_expr()?)
            //     }
            //     self.expect_token(TokenKind::RightParen)?;
            //     lhs = Expr::Cons(Op::Call, args)
            // }

            break;
        }

        // Return what was built so far as lhs: if nothing was built in a loop above, then it will
        // be an atom found at the beginning of the function. Otherwise it will be constructed
        // expression for this subtree bound by binding power.
        // Ok(lhs)

        Ok(())
    }

    pub fn expr_bp(&mut self, min_bp: u8) -> Result<Expr<'src>, CompileError> {
        let mut lhs = {
            let token = self.lexer.next_token();
            if let Ok(atom) = Atom::try_from(&token) {
                Expr::Atom(atom)
            } else if matches!(token.kind, TokenKind::Keyword(Keyword::This)) {
                if self.in_class.in_scope() {
                    Expr::Atom(Atom::Ident {
                        name: "this",
                        // loc: Loc {
                        //     line: self.current_line(),
                        //     // TODO: placeholder
                        //     col: 0,
                        // },
                    })
                } else {
                    return Err(CompileError::InvalidThisUsage);
                }
            } else if matches!(token.kind, TokenKind::Keyword(Keyword::Super)) {
                match self.in_class {
                    CurrentClass::Subclass(_) => {
                        match self.lexer.peek().kind {
                            TokenKind::Dot => Expr::Atom(Atom::Ident {
                                name: "super",
                                // loc: Loc {
                                //     line: self.current_line(),
                                //     // TODO: placeholder
                                //     col: 0,
                                // },
                            }),
                            _ => return Err(CompileError::SuperMustBeFollowedByGetExpression),
                        }
                    }
                    CurrentClass::Class(_) => {
                        return Err(CompileError::SuperInClassWithNoSuperclassNotAllowed);
                    }
                    CurrentClass::None => return Err(CompileError::SuperOutsideClassNotAllowed),
                }
            } else if token.unterminated() {
                return Err(CompileError::UnterminatedString {
                    line: token.line,
                    lexeme: token.lexeme.to_owned(),
                });
            } else if matches!(token.kind, TokenKind::LeftParen) {
                let members = self.expr_bp(0)?;
                {
                    let t = self.lexer.next_token();
                    if !matches!(t.kind, TokenKind::RightParen) {
                        return Err(CompileError::UnmatchedParen {
                            line: t.line,
                            lexeme: t.lexeme.to_owned(),
                        });
                    }
                }
                Expr::Cons(Op::Group, vec![members])
            } else if let Ok(op) = Op::try_from(&token)
                && let Some(bp) = op.prefix_bp()
            {
                let rhs = self.expr_bp(bp)?;
                Expr::Cons(op, vec![rhs])
            } else {
                return Err(CompileError::UnexpectedToken {
                    line: token.line,
                    lexeme: token.lexeme.to_owned(),
                });
            }
        };

        loop {
            let peeked = self.lexer.peek();
            if let Ok(op) = Op::try_from(peeked) {
                // TODO: continue from here. I work on property access.
                if let Some(l_bp) = op.postfix_bp() {
                    if l_bp < min_bp {
                        break;
                    }
                    self.lexer.advance();
                    lhs = match op {
                        // If a function call, parse all comma-separated arguments first
                        Op::Call => {
                            let mut args = vec![lhs]; // function name should be the first vec member
                            while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
                                args.push(self.next_expr()?);
                                if matches!(self.lexer.peek().kind, TokenKind::Comma) {
                                    self.lexer.advance(); // step over comma
                                    continue;
                                }
                                if args.len() >= 255 {
                                    return Err(CompileError::TooManyArguments);
                                }
                            }
                            self.lexer.advance(); // step over right paren
                            Expr::Cons(Op::Call, args)
                        }
                        // Otherwise treat it like any other postfix operator
                        _ => Expr::Cons(op, vec![lhs]),
                    };
                    // Continue to check the next operator
                    continue;
                }

                if let Some((l_bp, r_bp)) = op.infix_bp() {
                    // If the next operator's left binding power is less than minimum, it is not strong
                    // enough to bind the next atom, so we break and return lhs we've built so far
                    if l_bp < min_bp {
                        break;
                    }
                    self.lexer.advance();
                    let rhs = self.expr_bp(r_bp)?;
                    // What is built so far
                    lhs = Expr::Cons(op, vec![lhs, rhs]);
                    // Continue to check the next operator
                    continue;
                }
            } else if matches!(peeked.kind, TokenKind::RightParen)
                || peeked.is_eof()
                || matches!(peeked.kind, TokenKind::Comma)
            {
                break;
            }
            // else if matches!(peeked.kind, TokenKind::LeftParen) {
            //     // Parse a function call
            //     let mut args = vec![lhs];
            //     while !matches!(self.lexer.peek().kind, TokenKind::RightParen) {
            //         args.push(self.next_expr()?)
            //     }
            //     self.expect_token(TokenKind::RightParen)?;
            //     lhs = Expr::Cons(Op::Call, args)
            // }

            break;
        }

        // Return what was built so far as lhs: if nothing was built in a loop above, then it will
        // be an atom found at the beginning of the function. Otherwise it will be constructed
        // expression for this subtree bound by binding power.
        Ok(lhs)
    }

    pub fn current_line(&self) -> usize {
        self.lexer.current_line()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'src> {
    Expr(Expr<'src>),
    Print(Expr<'src>),
    Var {
        name: &'src str,
        init_val: Expr<'src>,
    },
    Block(Vec<Stmt<'src>>),
    If {
        condition: Expr<'src>,
        stmt: Box<Stmt<'src>>,
        else_stmt: Option<Box<Stmt<'src>>>,
    },
    While {
        condition: Expr<'src>,
        body: Box<Stmt<'src>>,
    },
    For {
        initializer: Option<Box<Stmt<'src>>>,
        condition: Expr<'src>,
        increment: Option<Expr<'src>>,
        body: Box<Stmt<'src>>,
    },
    Return(Option<Expr<'src>>),
    FunDecl {
        ident: &'src str,
        parameters: Vec<&'src str>,
        body: Vec<Stmt<'src>>,
    },
    ClassDecl {
        ident: &'src str,
        superclass: Option<&'src str>,
        methods: Vec<Stmt<'src>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    Atom(Atom<'src>),
    Cons(Op, Vec<Expr<'src>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom<'src> {
    Number(f64),
    String(&'src str),
    Bool(bool),
    Nil,
    Ident {
        name: &'src str,
        // loc: Loc
    },
}

impl<'src> TryFrom<&Token<'src>> for Atom<'src> {
    type Error = ();

    fn try_from(value: &Token<'src>) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Literal(Literal::Number(n)) => Ok(Atom::Number(n)),
            TokenKind::Literal(Literal::String { s, terminated }) => {
                if terminated {
                    Ok(Atom::String(s))
                } else {
                    Err(())
                }
            }
            TokenKind::Keyword(Keyword::True) => Ok(Atom::Bool(true)),
            TokenKind::Keyword(Keyword::False) => Ok(Atom::Bool(false)),
            TokenKind::Keyword(Keyword::Nil) => Ok(Atom::Nil),
            TokenKind::Ident => Ok(Atom::Ident {
                name: value.lexeme,
                // loc: Loc {
                //     line: value.line,
                //     col: value.col,
                // },
            }),
            _ => Err(()),
        }
    }
}

impl<'src> TryFrom<Token<'src>> for Atom<'src> {
    type Error = ();

    fn try_from(value: Token<'src>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Mult,
    Div,
    Group,
    Bang,
    Minus,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    EqEq,
    NotEq,
    Assign,
    And,
    Or,
    Call,
    Dot,
}

impl Op {
    fn infix_bp(&self) -> Option<(u8, u8)> {
        match self {
            Self::Assign => Some((2, 1)),
            Self::Or => Some((3, 2)),
            Self::And => Some((5, 4)),
            Self::Less
            | Self::LessEq
            | Self::Greater
            | Self::GreaterEq
            | Self::EqEq
            | Self::NotEq => Some((6, 7)),
            Self::Add | Self::Minus => Some((8, 9)),
            Self::Mult | Self::Div => Some((10, 11)),
            Self::Dot => Some((30, 29)),
            _ => None,
        }
    }

    fn prefix_bp(&self) -> Option<u8> {
        match self {
            Self::Bang => Some(20),
            Self::Minus => Some(20),
            _ => None,
        }
    }

    fn postfix_bp(&self) -> Option<u8> {
        match self {
            Self::Call => Some(20),
            _ => None,
        }
    }

    fn infix_as_op_code(&self) -> Option<OpCode> {
        match self {
            // Self::Assign => Some((2, 1)),
            // Self::Or => Some((3, 2)),
            // Self::And => Some((5, 4)),
            // Self::Less
            // | Self::LessEq
            // | Self::Greater
            // | Self::GreaterEq
            // | Self::EqEq
            // | Self::NotEq => Some((6, 7)),
            Self::Add => Some(OpCode::Add),
            Self::Minus => Some(OpCode::Sub),
            Self::Mult => Some(OpCode::Mult),
            Self::Div => Some(OpCode::Div),
            Self::EqEq => Some(OpCode::Equal),
            Self::Greater => Some(OpCode::Greater),
            Self::Less => Some(OpCode::Less),
            // Self::Dot => Some((30, 29)),
            _ => None,
        }
    }

    fn prefix_as_op_code(&self) -> Option<OpCode> {
        match self {
            Self::Minus => Some(OpCode::Negate),
            Self::Bang => Some(OpCode::Not),
            _ => None,
        }
    }

    fn postfix_as_op_code(&self) -> Option<OpCode> {
        todo!()
    }
}

impl<'src> TryFrom<&Token<'src>> for Op {
    type Error = ();

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Plus => Ok(Op::Add),
            TokenKind::Star => Ok(Op::Mult),
            TokenKind::Bang => Ok(Op::Bang),
            TokenKind::Minus => Ok(Op::Minus),
            TokenKind::Slash => Ok(Op::Div),
            TokenKind::Less => Ok(Op::Less),
            TokenKind::LessEqual => Ok(Op::LessEq),
            TokenKind::Greater => Ok(Op::Greater),
            TokenKind::GreaterEqual => Ok(Op::GreaterEq),
            TokenKind::EqEq => Ok(Op::EqEq),
            TokenKind::BangEq => Ok(Op::NotEq),
            TokenKind::Eq => Ok(Op::Assign),
            TokenKind::Keyword(Keyword::And) => Ok(Op::And),
            TokenKind::Keyword(Keyword::Or) => Ok(Op::Or),
            TokenKind::LeftParen => Ok(Op::Call),
            TokenKind::Dot => Ok(Op::Dot),
            _ => Err(()),
        }
    }
}

impl<'src> TryFrom<Token<'src>> for Op {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

enum CurrentClass<'src> {
    Class(&'src str),
    Subclass(&'src str),
    None,
}

impl<'src> CurrentClass<'src> {
    fn in_scope(&self) -> bool {
        match self {
            Self::Class(_) | Self::Subclass(_) => true,
            Self::None => false,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CompileError {
    #[error("[line {line}] Error at '{lexeme}': Expect expression.")]
    UnexpectedToken { line: usize, lexeme: String },
    #[error("[line {line}] Error at '{lexeme}'. Expect ')'.")]
    UnmatchedParen { line: usize, lexeme: String },
    #[error("[line {line}] Error at '{lexeme}'. Unterminated string.")]
    UnterminatedString { line: usize, lexeme: String },
    // TODO: somewhat collides with UnexpectedToken - think about a better name for both
    // I want for this error message
    #[error("{0}")]
    ExpectedTokenNotFound(String),
    #[error("[line {line}] Error at end: Expect '}}'.")]
    UnclosedBlock { line: usize },
    #[error("[line {line}] Expected statement after if condition.")]
    EmptyIfStatement { line: usize },
    #[error("[line {line}] Expected statement after while condition.")]
    EmptyWhileStatement { line: usize },
    #[error("Too many arguments. Functions can't have more than 255 arguments.")]
    TooManyArguments,
    #[error("Missing comma between parameters in function declaration.")]
    MissingCommaBetweenParameters,
    #[error("Can't use 'this' outside of a class.")]
    InvalidThisUsage,
    #[error("Can't return a value from an initializer.")]
    InvalidInitializerReturn,
    #[error("A class can't inherit from itself.")]
    InheritFromItselfNotAllowed,
    #[error("Can't use 'super' in a class with no superclass.")]
    SuperInClassWithNoSuperclassNotAllowed,
    #[error("Can't use 'super' outside of a class.")]
    SuperOutsideClassNotAllowed,
    #[error("'super' must be followed by get expression.")]
    SuperMustBeFollowedByGetExpression,
}

pub enum CompileErrorKind {
    UnexpectedToken { line: usize, lexeme: String },
    UnmatchedParen { line: usize, lexeme: String },
    UnterminatedString { line: usize, lexeme: String },
    // TODO: somewhat collides with UnexpectedToken - think about a better name for both
    // I want for this error message
    ExpectedTokenNotFound(String),
    UnclosedBlock { line: usize },
    EmptyIfStatement { line: usize },
    EmptyWhileStatement { line: usize },
    TooManyArguments,
    MissingCommaBetweenParameters,
    InvalidThisUsage,
    InvalidInitializerReturn,
    InheritFromItselfNotAllowed,
    SuperInClassWithNoSuperclassNotAllowed,
    SuperOutsideClassNotAllowed,
    SuperMustBeFollowedByGetExpression,
}

// #[derive(Debug, thiserror::Error)]
// pub struct CompileErrorNew {
//     line: usize,
//     kind: CompileErrorKind,
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_simple_expression() {
        let src = "1 + 2;";
        let chunk = compile(src).unwrap();
        // TODO: this bytecode needs to split statements
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_ADD
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn emits_correct_lines() {
        let src = r#"1 + 2;
3 - 4;
        "#;
        let chunk = compile(src).unwrap();
        // TODO: this bytecode needs to split statements
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_ADD
0005 2 OP_CONSTANT [2] 3.0
0007 | OP_CONSTANT [3] 4.0
0009 | OP_SUBTRACT
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn compiles_subtract_and_unary_negation() {
        let src = r#"-1;
10 - 5;
        "#;
        let chunk = compile(src).unwrap();
        // TODO: this bytecode needs to split statements
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_NEGATE
0003 2 OP_CONSTANT [1] 10.0
0005 | OP_CONSTANT [2] 5.0
0007 | OP_SUBTRACT
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn multiple_operations_in_expression() {
        let src = r#"1 + 2 + 3 - 4;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_ADD
0005 | OP_CONSTANT [2] 3.0
0007 | OP_ADD
0008 | OP_CONSTANT [3] 4.0
0010 | OP_SUBTRACT
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn lower_precedence_op_before_higher_precedence() {
        let src = r#"1 + 2 * 3;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_CONSTANT [2] 3.0
0006 | OP_MULTIPLY
0007 | OP_ADD
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn grouping_takes_precedence() {
        let src = r#"(1 + 2) * 3;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_ADD
0005 | OP_CONSTANT [2] 3.0
0007 | OP_MULTIPLY
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn division() {
        let src = r#"4 / 2;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 4.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_DIVIDE
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn return_statement() {
        let src = r#"return 1 + 2;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_ADD
0005 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn return_bool() {
        let src = r#"return false;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_FALSE
0001 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));

        let src = r#"return true;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_TRUE
0001 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk))
    }

    #[test]
    fn return_nil() {
        let src = r#"return nil;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_NIL
0001 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));
    }

    #[test]
    fn not() {
        let src = r#"return !true;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_TRUE
0001 | OP_NOT
0002 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));
    }

    #[test]
    fn eq() {
        let src = r#"return true == false;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_TRUE
0001 | OP_FALSE
0002 | OP_EQUAL
0003 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));
    }

    #[test]
    fn greater() {
        let src = r#"return 1 > 2;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_GREATER
0005 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));
    }

    #[test]
    fn less() {
        let src = r#"return 1 < 2;"#;
        let chunk = compile(src).unwrap();
        let expected = r#"
0000 1 OP_CONSTANT [0] 1.0
0002 | OP_CONSTANT [1] 2.0
0004 | OP_LESS
0005 | OP_RETURN
"#
        .strip_prefix("\n")
        .unwrap();
        assert_eq!(expected, format!("{}", chunk));
    }
}
