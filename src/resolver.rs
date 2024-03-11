use crate::error::{parser_error, Error};
use crate::expr::{expr, Expr};
use crate::interpreter::Interpreter;
use crate::stmt::{stmt, Stmt};
use crate::token::{Object, Token};

use std::collections::HashMap;
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
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
        function_type: FunctionType,
    ) -> Result<(), Error> {
        let enclosing_function = mem::replace(&mut self.current_function, function_type);

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

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<(), Error> {
        self.resolve_expr(expression)?;
        Ok(())
    }

    fn visit_literal_expr(&self, _value: &Object) -> Result<(), Error> {
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

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<(), Error> {
        self.resolve_expr(value)?;
        self.resolve_local(name);
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

    fn visit_get_expr(&mut self, object: &Expr, _name: &Token) -> Result<(), Error> {
        self.resolve_expr(object)?;
        Ok(())
    }

    fn visit_set_expr(&mut self, object: &Expr, _name: &Token, value: &Expr) -> Result<(), Error> {
        self.resolve_expr(value)?;
        self.resolve_expr(object)?;
        Ok(())
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<(), Error> {
        match self.current_class {
            ClassType::None => parser_error(keyword, "Cannot use 'this' outside of a class."),
            ClassType::Class => self.resolve_local(keyword),
        }
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
            if let FunctionType::Initializer = self.current_function {
                parser_error(keyword, "Cannot return value from initializer.");
            }
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

    fn visit_class_stmt(&mut self, name: &Token, methods: &[Stmt]) -> Result<(), Error> {
        let enclosing_class = mem::replace(&mut self.current_class, ClassType::Class);

        self.declare(name);
        self.define(name);

        self.begin_scope();
        self.scopes
            .last_mut()
            .expect("Scopes is empty.")
            .insert("this".to_owned(), true);

        for method in methods {
            if let Stmt::Function { name, params, body } = method {
                let declaration = if name.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(params, body, declaration)?;
            } else {
                unreachable!()
            }
        }

        self.end_scope();

        self.current_class = enclosing_class;

        Ok(())
    }
}
