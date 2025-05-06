use crate::b;
use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::expr::assignment::Assign;
use crate::interpreter::ast::expr::binary::Binary;
use crate::interpreter::ast::expr::call::Call;
use crate::interpreter::ast::expr::get::Get;
use crate::interpreter::ast::expr::grouping::Grouping;
use crate::interpreter::ast::expr::list::List;
use crate::interpreter::ast::expr::literal::Literal;
use crate::interpreter::ast::expr::logical::Logical;
use crate::interpreter::ast::expr::object::Obj;
use crate::interpreter::ast::expr::self_expr::SelfExpr;
use crate::interpreter::ast::expr::set::Set;
use crate::interpreter::ast::expr::superclass::Super;
use crate::interpreter::ast::expr::unary::Unary;
use crate::interpreter::ast::expr::variable::Variable;
use crate::interpreter::ast::stmt::Stmt;
use crate::interpreter::ast::stmt::block::Block;
use crate::interpreter::ast::stmt::class::Class;
use crate::interpreter::ast::stmt::export_stmt::Export;
use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::ast::stmt::if_stmt::If;
use crate::interpreter::ast::stmt::let_stmt::Let;
use crate::interpreter::ast::stmt::print::Print;
use crate::interpreter::ast::stmt::return_stmt::Return;
use crate::interpreter::ast::stmt::stmt_expr::StmtExpr;
use crate::interpreter::ast::stmt::use_stmt::Use;
use crate::interpreter::ast::stmt::while_stmt::While;
use crate::interpreter::error::Result;
use crate::interpreter::object::Object;
use crate::interpreter::parser::error::{ParserError, ParserErrorType};
use crate::interpreter::scanner::token::Token;
use crate::interpreter::scanner::token::token_type::TokenType;
use std::collections::HashMap;
use std::marker::PhantomData;

pub mod error;
pub mod resolver;
pub struct Parser<T> {
    phantom_data: PhantomData<T>,
    tokens: Vec<Token>,
    current: usize,
}

impl<T> Parser<T>
where
    T: 'static + Clone,
{
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            phantom_data: Default::default(),
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Stmt<T>>>> {
        let mut statements = vec![];

        let mut error_stack = vec![];

        while !self.is_at_end() {
            match self.export() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    error_stack.push(err);
                    self.synchronize()
                }
            }
        }

        if error_stack.is_empty() {
            return Ok(statements);
        }

        Err(error_stack
            .into_iter()
            .map(|err| err.to_string())
            .collect::<String>()
            .into())
    }

    fn export(&mut self) -> Result<Box<dyn Stmt<T>>> {
        if self._match(vec![TokenType::Use]) {
            return self.import();
        }
        if self._match(vec![TokenType::Export]) {
            return Ok(b!(Export::new(self.previous(), self.declaration()?)));
        }
        self.declaration()
    }

    fn import(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let name = self.previous();
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, ParserErrorType::ExpectedSemicolon)?;

        Ok(b!(Use::new(name, expr)))
    }

    fn declaration(&mut self) -> Result<Box<dyn Stmt<T>>> {
        if self._match(vec![TokenType::Let]) {
            return self.let_declaration();
        }

        if self._match(vec![TokenType::Fun]) {
            return self.fun_declaration();
        }

        if self._match(vec![TokenType::Class]) {
            return self.class_declaration();
        }

        self.statement()
    }

    fn class_declaration(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let name = self.consume(
            TokenType::Identifier,
            ParserErrorType::ExpectedIdentAfterClassDecl,
        )?;

        let mut super_class = None;
        if self._match(vec![TokenType::Less]) {
            self.consume(
                TokenType::Identifier,
                ParserErrorType::ExpectedSuperClassIdent,
            )?;
            super_class = Some(Variable::new(self.previous()));
        }

        self.consume(
            TokenType::LeftBrace,
            ParserErrorType::ExpectedLeftBraceBeforeBody,
        )?;

        let mut methods = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match self.fun_declaration()?.downcast::<Fun<T>>() {
                Ok(func) => methods.push(*func),
                Err(_) => {
                    return Err(ParserError::new(self.previous(), ParserErrorType::NotAFunc).into());
                }
            }
        }

        self.consume(
            TokenType::RightBrace,
            ParserErrorType::ExpectedMatchingBrace,
        )?;

        Ok(b!(Class::new(name, methods, super_class)))
    }

    fn fun_declaration(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let name = self.consume(
            TokenType::Identifier,
            ParserErrorType::ExpectedIdentAfterFunDecl,
        )?;

        self.consume(
            TokenType::LeftParen,
            ParserErrorType::ExpectedLeftParenAfterFunIdent,
        )?;

        let mut params = vec![];

        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, ParserErrorType::ExpectedParamName)?);
            while self.check(TokenType::Comma) {
                if params.len() >= 255 {
                    return Err(ParserError::new(
                        self.peek(),
                        ParserErrorType::CountOfParamsGreaterThen255,
                    )
                    .into());
                }
                params
                    .push(self.consume(TokenType::Identifier, ParserErrorType::ExpectedParamName)?);
            }
        }

        self.consume(
            TokenType::RightParen,
            ParserErrorType::ExpectedRightParenAfterParams,
        )?;
        self.consume(
            TokenType::LeftBrace,
            ParserErrorType::ExpectedLeftBraceBeforeBody,
        )?;

        let body = self.block_statement()?;
        Ok(b!(Fun::new(name, params, body)))
    }

    fn let_declaration(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let name = self.consume(TokenType::Identifier, ParserErrorType::ExpectedVariableName)?;

        let mut initializer = None;
        if self._match(vec![TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            ParserErrorType::ExpectedSemicolonAfterVarDecl,
        )?;

        Ok(b!(Let::new(name, initializer)))
    }

    fn statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        if self._match(vec![TokenType::Print]) {
            return self.print_statement();
        }
        if self._match(vec![TokenType::LeftBrace]) {
            return Ok(b!(Block::new(self.block_statement()?)));
        }

        if self._match(vec![TokenType::If]) {
            return self.if_statement();
        }

        if self._match(vec![TokenType::While]) {
            return self.while_statement();
        }

        if self._match(vec![TokenType::For]) {
            return self.for_statement();
        }

        if self._match(vec![TokenType::Return]) {
            return self.return_statement();
        }

        self.expr_statement()
    }

    fn return_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let token = self.previous();
        let expr = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, ParserErrorType::ExpectedSemicolon)?;
        Ok(b!(Return::new(token, expr)))
    }

    fn for_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        self.consume(
            TokenType::LeftParen,
            ParserErrorType::ExpectedLeftParenAfterFor,
        )?;

        let initializer: Option<Box<dyn Stmt<T>>> = if self._match(vec![TokenType::Let]) {
            Some(self.let_declaration()?)
        } else {
            Some(self.expr_statement()?)
        };

        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, ParserErrorType::ExpectedSemicolon)?;

        let mut increment = None;
        if !self.check(TokenType::Semicolon) {
            increment = Some(self.expression()?);
        }
        self.consume(
            TokenType::RightParen,
            ParserErrorType::ExpectedRightParenAfterForStatement,
        )?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = b!(Block::new(vec![body, b!(StmtExpr::new(increment))]));
        }

        if let Some(condition) = condition {
            body = b!(While::new(condition, body));
        }

        if let Some(initializer) = initializer {
            body = b!(Block::new(vec![initializer, body]));
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let condition = self.expression()?;

        let stmt = self.statement()?;

        Ok(b!(While::new(condition, stmt)))
    }

    fn if_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let condition = self.expression()?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self._match(vec![TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(b!(If::new(condition, then_branch, else_branch)))
    }

    fn block_statement(&mut self) -> Result<Vec<Box<dyn Stmt<T>>>> {
        let mut statements = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            ParserErrorType::ExpectedMatchingBrace,
        )?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, ParserErrorType::ExpectedSemicolon)?;
        Ok(b!(Print::new(value)))
    }

    fn expr_statement(&mut self) -> Result<Box<dyn Stmt<T>>> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, ParserErrorType::ExpectedSemicolon)?;
        Ok(b!(StmtExpr::new(expr)))
    }

    pub fn expression(&mut self) -> Result<Box<dyn Expr<T>>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<dyn Expr<T>>> {
        let expr = self.logic_or()?;
        if self._match(vec![TokenType::Equal]) {
            let token = self.previous();
            let value = self.assignment()?;

            if let Some(expr) = expr.downcast_ref::<Variable>() {
                let name = expr.get_token();
                return Ok(b!(Assign::new(name, value)));
            }

            if let Some(expr) = expr.downcast_ref::<Get<T>>() {
                let (name, obj) = expr.extract();
                return Ok(b!(Set::new(name.clone(), obj.clone_expr(), value)));
            }

            return Err(ParserError::new(token, ParserErrorType::InvalidAssignmentTarget).into());
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.logic_and()?;

        while self._match(vec![TokenType::Or]) {
            let token = self.previous();
            let right = self.logic_and()?;
            expr = b!(Logical::new(expr, token, right));
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.equality()?;

        while self._match(vec![TokenType::And]) {
            let token = self.previous();
            let right = self.equality()?;
            expr = b!(Logical::new(expr, token, right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.comparison()?;

        while self._match(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let token = self.previous();
            let right = self.comparison()?;
            expr = b!(Binary::new(expr, token, right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.term()?;

        while self._match(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let token = self.previous();
            let right = self.term()?;
            expr = b!(Binary::new(expr, token, right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.factor()?;

        while self._match(vec![TokenType::Minus, TokenType::Plus]) {
            let token = self.previous();
            let right = self.factor()?;
            expr = b!(Binary::new(expr, token, right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.unary()?;

        while self._match(vec![TokenType::Star, TokenType::Slash]) {
            let token = self.previous();
            let right = self.unary()?;
            expr = b!(Binary::new(expr, token, right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn Expr<T>>> {
        if self._match(vec![TokenType::Bang, TokenType::Minus]) {
            let token = self.previous();
            let right = self.call()?;
            return Ok(b!(Unary::new(token, right)));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut expr = self.primary()?;
        loop {
            if self._match(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self._match(vec![TokenType::Dot]) {
                let name = self.consume(
                    TokenType::Identifier,
                    ParserErrorType::ExpectedPropertyAfterDot,
                )?;
                expr = b!(Get::new(name, expr))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Box<dyn Expr<T>>) -> Result<Box<dyn Expr<T>>> {
        let mut arguments = vec![];
        if !self.check(TokenType::RightParen) {
            arguments.push(self.expression()?);
            while self._match(vec![TokenType::Comma]) {
                arguments.push(self.expression()?);
            }
        }

        let paren = self.consume(
            TokenType::RightParen,
            ParserErrorType::ExpectedRightParenAfterArguments,
        )?;

        if arguments.len() > 255 {
            return Err(ParserError::new(paren, ParserErrorType::CountOfArgsGreaterThen255).into());
        }

        Ok(b!(Call::new(expr, paren, arguments)))
    }

    fn primary(&mut self) -> Result<Box<dyn Expr<T>>> {
        if self._match(vec![TokenType::False]) {
            return Ok(b!(Literal::new(Some(Object::Bool(false)))));
        }
        if self._match(vec![TokenType::True]) {
            return Ok(b!(Literal::new(Some(Object::Bool(true)))));
        }
        if self._match(vec![TokenType::Nil]) {
            return Ok(b!(Literal::new(Some(Object::Nil))));
        }

        if self._match(vec![TokenType::Number, TokenType::String]) {
            return Ok(b!(Literal::new(self.previous().get_lit())));
        }

        if self._match(vec![TokenType::Identifier]) {
            return Ok(b!(Variable::new(self.previous())));
        }

        if self._match(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                ParserErrorType::ExpectedMatchingParens,
            )?;
            return Ok(b!(Grouping::new(expr)));
        }

        if self._match(vec![TokenType::Slf]) {
            return Ok(b!(SelfExpr::new(self.previous())));
        }

        if self._match(vec![TokenType::Super]) {
            let keyword = self.previous();
            self.consume(TokenType::Dot, ParserErrorType::ExpectedDotAfterSuper)?;
            let method = self.consume(
                TokenType::Identifier,
                ParserErrorType::ExpectedMethodAfterDot,
            )?;

            return Ok(b!(Super::new(keyword, method)));
        }

        if self._match(vec![TokenType::LeftBracket]) {
            return self.list();
        }

        if self._match(vec![TokenType::LeftBrace]) {
            return self.obj();
        }

        Err(self
            .error(self.peek(), ParserErrorType::ExpectedExpression)
            .into())
    }

    fn obj(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut values = HashMap::new();
        if !self.check(TokenType::RightBrace) {
            let name = if self._match(vec![
                TokenType::Identifier,
                TokenType::Number,
                TokenType::String,
            ]) {
                self.previous()
            } else {
                return Err(ParserError::new(self.peek(), ParserErrorType::ExpectedKey).into());
            };
            //println!("{name}");
            self.consume(TokenType::Colon, ParserErrorType::ExpectedColon)?;
            let value = self.expression()?;
            values.insert(name, value);
            while self._match(vec![TokenType::Comma]) {
                let name = if self._match(vec![
                    TokenType::Identifier,
                    TokenType::Number,
                    TokenType::String,
                ]) {
                    self.previous()
                } else {
                    return Err(ParserError::new(self.peek(), ParserErrorType::ExpectedKey).into());
                };
                self.consume(TokenType::Colon, ParserErrorType::ExpectedColon)?;
                let value = self.expression()?;
                values.insert(name, value);
            }
        }
        self.consume(TokenType::RightBrace, ParserErrorType::ExpectedRightBrace)?;
        Ok(b!(Obj::new(values)))
    }

    fn list(&mut self) -> Result<Box<dyn Expr<T>>> {
        let mut values = vec![];
        if !self.check(TokenType::RightBracket) {
            values.push(self.expression()?);
            while self._match(vec![TokenType::Comma]) {
                values.push(self.expression()?);
            }
        }
        self.consume(
            TokenType::RightBracket,
            ParserErrorType::ExpectedRightBracket,
        )?;
        Ok(b!(List::new(values)))
    }

    fn _match(&mut self, types: Vec<TokenType>) -> bool {
        for ty in types {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, ty: TokenType, error_ty: ParserErrorType) -> Result<Token> {
        if self.check(ty) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), error_ty).into())
    }

    fn check(&self, ty: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().get_type().eq(&ty)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().get_type().eq(&TokenType::Eof)
    }

    fn error(&self, token: Token, error_ty: ParserErrorType) -> ParserError {
        ParserError::new(token, error_ty)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().get_type().eq(&TokenType::Semicolon) {
                return;
            }

            match self.peek().get_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
