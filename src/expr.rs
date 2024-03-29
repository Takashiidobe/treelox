use crate::{
    error::Error,
    token::{Object, Token},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Object,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
    },
}

pub mod expr {
    use crate::{
        error::Error,
        token::{Object, Token},
    };

    use super::Expr;

    pub trait Visitor<R> {
        fn visit_binary_expr(
            &mut self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> Result<R, Error>;
        fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<R, Error>;
        fn visit_literal_expr(&self, value: &Object) -> Result<R, Error>;
        fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<R, Error>;
        fn visit_variable_expr(&mut self, name: &Token) -> Result<R, Error>;
        fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<R, Error>;
        fn visit_logical_expr(
            &mut self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> Result<R, Error>;
        fn visit_call_expr(
            &mut self,
            callee: &Expr,
            paren: &Token,
            arguments: &[Expr],
        ) -> Result<R, Error>;
        fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<R, Error>;
        fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr)
            -> Result<R, Error>;
        fn visit_this_expr(&mut self, keyword: &Token) -> Result<R, Error>;
        fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<R, Error>;
    }
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn expr::Visitor<R>) -> Result<R, Error> {
        match self {
            Expr::Assign { name, value } => visitor.visit_assign_expr(name, value),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expr } => visitor.visit_grouping_expr(expr),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Variable { name } => visitor.visit_variable_expr(name),
            Expr::Logical {
                left,
                operator,
                right,
            } => visitor.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => visitor.visit_call_expr(callee, paren, arguments),
            Expr::Get { object, name } => visitor.visit_get_expr(object, name),
            Expr::Set {
                object,
                name,
                value,
            } => visitor.visit_set_expr(object, name, value),
            Expr::This { keyword } => visitor.visit_this_expr(keyword),
            Expr::Super { keyword, method } => visitor.visit_super_expr(keyword, method),
        }
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> Result<String, Error> {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: String, exprs: &[&Expr]) -> Result<String, Error> {
        let mut r = String::new();
        r.push('(');
        r.push_str(&name);
        for e in exprs {
            r.push(' ');
            r.push_str(&e.accept(self)?);
        }
        r.push(')');
        Ok(r)
    }
}

impl expr::Visitor<String> for AstPrinter {
    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(operator.lexeme.clone(), &[left, right])
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<String, Error> {
        self.parenthesize("group".to_string(), &[expr])
    }

    fn visit_literal_expr(&self, value: &Object) -> Result<String, Error> {
        Ok(value.to_string())
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<String, Error> {
        self.parenthesize(operator.lexeme.clone(), &[right])
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<String, Error> {
        Ok(name.lexeme.clone())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), &[value])
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        name: &Token,
        right: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), &[left, right])
    }
    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
    ) -> Result<String, Error> {
        let mut aggregated = vec![callee];
        aggregated.extend(arguments.iter());
        self.parenthesize(paren.lexeme.clone(), &aggregated)
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), &[object])
    }

    fn visit_set_expr(
        &mut self,
        object: &Expr,
        name: &Token,
        value: &Expr,
    ) -> Result<String, Error> {
        self.parenthesize(name.lexeme.clone(), &[object, value])
    }

    fn visit_this_expr(&mut self, _keyword: &Token) -> Result<String, Error> {
        Ok("this".to_string())
    }

    fn visit_super_expr(&mut self, _keyword: &Token, _method: &Token) -> Result<String, Error> {
        Ok("super".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    use insta::assert_debug_snapshot;

    macro_rules! test_printer {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($source.to_string());
                let tokens = scanner.scan_tokens();
                let mut parser = Parser::new(tokens);
                let expressions = parser.parse_exprs();
                let mut printer = AstPrinter;
                if let Ok(exprs) = expressions {
                    let res: Vec<_> = exprs.into_iter().map(|expr| printer.print(expr)).collect();
                    assert_debug_snapshot!(res);
                } else {
                    assert_debug_snapshot!(expressions);
                }
            }
        };
    }

    test_printer!(multiplication, "-123 * 45.67");
}
