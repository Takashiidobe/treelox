use crate::error::{parser_error, Error};
use crate::expr::{expr, Expr};
use crate::interpreter::Interpreter;
use crate::stmt::{stmt, Stmt};
use crate::token::{Object, Token};

use std::collections::HashMap;

#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), Error> {
        statement.accept(self)
    }

    pub fn resolve_stmts(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        for statement in statements {
            self.resolve_stmt(statement)?
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expression: &Expr) -> Result<(), Error> {
        expression.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(ref mut scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                parser_error(
                    name,
                    "Variable with this name already declared in this scope.",
                );
            }
            scope.insert(name.lexeme.clone(), false);
        };
    }

    fn define(&mut self, name: &Token) {
        if let Some(ref mut scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &[Stmt],
        tpe: FunctionType,
    ) -> Result<(), Error> {
        let enclosing_function = self.current_function.clone();
        self.current_function = tpe;

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name, i);
            }
        }
    }
}

impl<'i> expr::Visitor<()> for Resolver<'i> {
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<(), Error> {
        self.resolve_expr(value)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<(), Error> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        _paren: &Token,
        arguments: &[Expr],
    ) -> Result<(), Error> {
        self.resolve_expr(callee)?;
        for argument in arguments {
            self.resolve_expr(argument)?;
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression)?;
        Ok(())
    }

    fn visit_literal_expr(&self, _value: &Object) -> Result<(), Error> {
        Ok(())
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<(), Error> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, _operator: &Token, right: &Expr) -> Result<(), Error> {
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<(), Error> {
        if let Some(scope) = self.scopes.last() {
            if let Some(flag) = scope.get(&name.lexeme) {
                if !*flag {
                    parser_error(name, "Cannot read local variable in its own initializer.");
                }
            }
        };
        self.resolve_local(name);
        Ok(())
    }
}

impl<'i> stmt::Visitor<()> for Resolver<'i> {
    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        self.begin_scope();
        self.resolve_stmts(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression)?;
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &[Stmt],
    ) -> Result<(), Error> {
        self.declare(name);
        self.define(name);

        self.resolve_function(params, body, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> Result<(), Error> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(then_branch)?;
        if let Some(else_stmt) = else_branch {
            self.resolve_stmt(else_stmt)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression)?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        if let FunctionType::None = self.current_function {
            parser_error(keyword, "Cannot return from top-level code.");
        }

        if let Some(return_value) = value {
            self.resolve_expr(return_value)?;
        }
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expr(init)?;
        }
        self.define(name);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Error> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)?;
        Ok(())
    }
}
