use crate::error::Error;
use crate::{expr::Expr, token::Token};

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    #[default]
    Null,
}

pub mod stmt {
    use crate::{expr::Expr, token::Token};

    use super::{Error, Stmt};

    pub trait Visitor<R> {
        fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<R, Error>;
        fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<R, Error>;
        fn visit_print_stmt(&mut self, expression: &Expr) -> Result<R, Error>;
        fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<R, Error>;
        fn visit_if_stmt(
            &mut self,
            condition: &Expr,
            then_branch: &Stmt,
            else_branch: &Option<Stmt>,
        ) -> Result<R, Error>;
        fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<R, Error>;
    }
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn stmt::Visitor<R>) -> Result<R, Error> {
        match self {
            Stmt::Block { statements } => visitor.visit_block_stmt(statements),
            Stmt::Expression { expr } => visitor.visit_expression_stmt(expr),
            Stmt::Print { expr } => visitor.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(name, initializer),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => visitor.visit_while_stmt(condition, body),
            Stmt::Null => unimplemented!(),
        }
    }
}
