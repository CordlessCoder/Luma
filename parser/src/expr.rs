use ast::Expr;
use diagnostics::ErrorComponent;
use lexer::Token;

use crate::Parser;

use super::SToken;

impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    pub fn parse_group(&mut self) -> Option<Expr<'s>> {
        self.expect(&Token::LParen)?;
        let expr = self.parse_expr(BindingPower::Lowest)?;
        self.expect(&Token::RParen)?;
        Some(ast::Expr::Group(Box::new(expr)))
    }
    pub fn parse_expr(&mut self, bp: BindingPower) -> Option<Expr<'s>> {
        let (t, span) = self.peek_next_split();
        match t? {
            Token::LParen => self.parse_group(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingPower {
    /// No binding power
    None = 0,
    /// Lowest binding power
    Lowest,
    /// Assignment operators (=, +=, etc.)
    Assign,
    /// Ternary conditional operator (? :)
    Ternary,
    /// Logical OR operator (||)
    LogicalOr,
    /// Logical AND operator (&&)
    LogicalAnd,
    /// Bitwise OR operator (|)
    BitOr,
    /// Bitwise XOR operator (^)
    BitXor,
    /// Bitwise AND operator (&)
    BitAnd,
    /// Equality operators (==, !=)
    Equality,
    /// Relational operators (<, >, <=, >=)
    Relational,
    /// Range operations (..)
    Range,
    /// Shift operators (<<, >>)
    Shift,
    /// Addition and subtraction (+, -)
    Sum,
    /// Multiplication, division, modulo (*, /, %)
    Product,
    /// Exponentiation operator (**)
    Exponent,
    /// Unary operators (!, ~, +, -, prefix ++/--)
    Unary,
    /// Postfix operators (++/-- postfix)
    Postfix,
    /// Function call or indexing
    Call,
    /// Primary expressions (literals, variables)
    Primary,
}
