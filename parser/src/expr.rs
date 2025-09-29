use ast::{Expr, LiteralExpression, UnaryOpKind};
use diagnostics::ErrorComponent;
use lexer::Token;

use crate::Parser;

use super::SToken;

impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    // Should only be called after a leading ( has been consumed
    pub fn parse_group(&mut self) -> Option<Expr<'s>> {
        let expr = self.parse_expr(BindingPower::Lowest)?;
        self.expect(&Token::RParen)?;
        Some(ast::Expr::Group(Box::new(expr)))
    }
    pub fn parse_unary(&mut self) -> Option<Expr<'s>> {
        let (t, span) = self.advance_if_split(|t| token_to_uop(t).is_some());
        let op = t.as_ref().and_then(token_to_uop);
        let Some(op) = op else {
            let msg = format!("Expected a unary operator, found {t:?}");
            self.new_parse_error(span, "Unary operator error")
                .set_long_message(msg);
            return None;
        };
        let val = self.parse_expr(BindingPower::Unary)?;
        Some(Expr::UnaryOp(Box::new(ast::UnaryOp { val, op })))
    }
    pub fn null_denotation(&mut self) -> Option<Expr<'s>> {
        let (t, span) = self.advance_split();
        Some(match t? {
            Token::LParen => self.parse_group()?,
            Token::IntLit(i) => Expr::Lit(LiteralExpression::Int(i)),
            Token::FloatLit(i) => Expr::Lit(LiteralExpression::Float(i)),
            Token::CharLiteral(c) => Expr::Lit(LiteralExpression::Char(c)),
            Token::BoolLit(b) => Expr::Lit(LiteralExpression::Bool(b)),
            Token::StringLit(s) => Expr::Lit(LiteralExpression::Str(s.clone())),
            Token::Ident(i) => Expr::Ident(i),
            t @ (Token::Minus | Token::Plus | Token::Not | Token::PlusPlus | Token::MinusMinus) => {
                self.put_back(utils::Spanned::new(t, span));
                self.parse_unary()?
            }
            t => {
                let msg = format!("Parsing {t} as a null denotation is not yet supported");
                self.new_parse_error(span, "TODO").set_long_message(msg);
                return None;
            }
        })
    }
    pub fn left_denotation(&mut self, lhs: Expr<'s>, bp: BindingPower) -> Option<Expr<'s>> {
        todo!()
    }
    pub fn parse_expr(&mut self, outer_bp: BindingPower) -> Option<Expr<'s>> {
        let mut left = self.null_denotation()?;
        loop {
            let next = self.peek_next().map(|s| &s.inner);
            let new_bp = next.map(BindingPower::from_token).unwrap_or_default();
            if new_bp <= outer_bp {
                break;
            }
            left = self.left_denotation(left, new_bp)?;
        }
        Some(left)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum BindingPower {
    /// No binding power
    #[default]
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

impl BindingPower {
    pub fn from_token(t: &Token<'_>) -> Self {
        use BindingPower as BP;
        use Token::*;
        match t {
            Eq => BP::Assign,
            Question => BP::Ternary,
            Or => BP::LogicalOr,
            And => BP::LogicalOr,
            BitOr => BP::BitOr,
            BitXor => BP::BitXor,
            Ampersand => BP::BitAnd,
            EqEq | Ne => BP::Equality,
            Lt | LtEq | Gt | GtEq => BP::Relational,
            Plus | Minus => BP::Sum,
            Star | Slash | Percent => BP::Product,
            PlusPlus | MinusMinus => BP::Postfix,
            LParen | LBracket | Dot | ColonColon => BP::Call,
            Range => BP::Range,
            _ => BP::None,
        }
    }
}

fn token_to_uop(tok: &Token<'_>) -> Option<UnaryOpKind> {
    use Token as T;
    use UnaryOpKind as U;
    Some(match tok {
        T::Not => U::Not,
        T::Minus => U::Neg,
        T::Plus => U::Pos,
        T::Tilde => U::BitNot,
        T::PlusPlus => U::PreInc,
        T::MinusMinus => U::PreDec,
        T::Star => U::Deref,
        T::Ampersand => U::Addr,
        _ => return None,
    })
}
