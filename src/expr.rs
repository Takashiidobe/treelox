use crate::token::{Object, Token};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
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
}

pub trait Visitor<R> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping_expr(&self, expr: &Expr) -> R;
    fn visit_literal_expr(&self, value: &Object) -> R;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> R;
}

impl Expr {
    pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expr } => visitor.visit_grouping_expr(expr),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
        }
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: String, exprs: &[&Expr]) -> String {
        let mut r = String::new();
        r.push('(');
        r.push_str(&name);
        for expr in exprs {
            r.push(' ');
            r.push_str(&expr.accept(self));
        }
        r.push(')');
        r
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.clone(), &[left, right])
    }

    fn visit_grouping_expr(&self, expr: &Expr) -> String {
        self.parenthesize("group".to_string(), &[expr])
    }

    fn visit_literal_expr(&self, value: &Object) -> String {
        value.to_string()
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.clone(), &[right])
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;

    use super::*;

    #[test]
    fn test_multiplication() {
        let expression = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    r#type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                right: Box::new(Expr::Literal {
                    value: Object::Number(123.0),
                }),
            }),
            operator: Token {
                r#type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    value: Object::Number(45.67),
                }),
            }),
        };

        let result = AstPrinter.print(expression);

        assert_eq!(result, "(* (- 123) (group 45.67))");
    }
}
