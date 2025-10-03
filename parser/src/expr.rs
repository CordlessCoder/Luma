use std::{fmt::Display, iter::FusedIterator};

use ast::{BinOpKind, Expr, Intrinsic, LiteralExpression, SizeOf, Type, UnaryOpKind};
use diagnostics::ErrorComponent;
use lexer::Token;

use crate::Parser;

use super::SToken;

impl<'s, Tokens: Iterator<Item = Result<SToken<'s>, ErrorComponent>>> Parser<'s, Tokens> {
    /// Should only be called after a leading ( has been consumed
    fn parse_group(&mut self) -> Option<Expr<'s>> {
        let expr = self.parse_expr(BindingPower::Lowest)?;
        self.expect(&Token::RParen)?;
        Some(ast::Expr::Group(Box::new(expr)))
    }
    pub(crate) fn delimited_list_with_terminator<'p, T, P: FnMut(&'_ mut Self) -> Option<T>>(
        &'p mut self,
        mut parser: P,
        delimiter: &'p Token<'p>,
        terminator: &'p Token<'s>,
    ) -> impl FusedIterator<Item = T> + use<'p, 's, T, P, Tokens> {
        core::iter::from_fn(move || {
            if self.consume_if(|t| t == terminator) {
                return None;
            }
            let expr = parser(self)?;
            if !self.consume_if(|t| t == delimiter) {
                self.expect(terminator)?;
                return None;
            }
            Some(expr)
        })
        .fuse()
    }
    pub(crate) fn parse_type(&mut self) -> Option<Type<'s>> {
        let (next, span) = self.advance_split();
        let Some(next) = next else {
            self.new_parse_error(span, "Expected a type, found EOF.");
            return None;
        };
        Some(match next {
            Token::Ident(name) => Type::Named(name),
            Token::Void => Type::Void,
            Token::Bool => Type::Bool,
            Token::Char => Type::Char,
            Token::Int => Type::Int,
            Token::UInt => Type::UInt,
            Token::Float => Type::Float,
            Token::Double => Type::Double,
            Token::Str => Type::Str,
            Token::Star => {
                let pointee = self.parse_type()?;
                let pointee = Box::new(pointee);
                Type::Pointer { pointee }
            }
            Token::LBrace => {
                let ty = self.parse_type()?;
                self.expect(&Token::Semicolon)?;
                let size = self.parse_expr(BindingPower::Lowest)?;
                self.expect(&Token::RBracket)?;
                Type::Array(Box::new(ty), size)
            }
            t => {
                let msg = format!("Cannot parse {t} as a type.");
                self.new_parse_error(span, "Invalid type")
                    .set_long_message(msg);
                return None;
            }
        })
    }
    /// Should only be called after a leading [ has been consumed
    fn parse_array(&mut self) -> Option<Expr<'s>> {
        let elements = self
            .delimited_list_with_terminator(
                |p| p.parse_expr(BindingPower::Lowest),
                &Token::Comma,
                &Token::RBracket,
            )
            .collect();
        Some(ast::Expr::Array(elements))
    }
    fn parse_unary(&mut self) -> Option<Expr<'s>> {
        let (t, span) = self.advance_if_split(|t| token_to_uop(t).is_some());
        let op = t.as_ref().map(|t| token_to_uop(t).unwrap());
        let Some(op) = op else {
            let msg = format!("Expected a unary operator, found {t:?}");
            self.new_parse_error(span, "Unary operator error")
                .set_long_message(msg);
            return None;
        };
        let val = self.parse_expr(BindingPower::Unary)?;
        Some(Expr::UnaryOp(Box::new(ast::UnaryOp { val, op })))
    }
    /// Used for the left-hand side, to be later extended by [Self::left_denotation].
    pub(crate) fn null_denotation(&mut self) -> Option<Expr<'s>> {
        let (t, span) = self.advance_split();
        Some(match t? {
            Token::LParen => self.parse_group()?,
            Token::LBracket => self.parse_array()?,
            Token::Star => Expr::Intrinsic(Box::new(Intrinsic::Deref {
                addr: self.parse_expr(BindingPower::Unary)?,
            })),
            Token::Ampersand => Expr::Intrinsic(Box::new(Intrinsic::Addr {
                of: self.parse_expr(BindingPower::None)?,
            })),
            Token::Alloc => {
                self.peek_next_span().unwrap_or_else(|| self.end_span());
                self.expect(&Token::LParen)?;
                let size = {
                    let start = self
                        .peek_next_span()
                        .unwrap_or_else(|| self.end_span())
                        .start;
                    let mut args = self.delimited_list_with_terminator(
                        |p| p.parse_expr(BindingPower::None),
                        &Token::Comma,
                        &Token::RParen,
                    );
                    let [Some(size), None] = core::array::from_fn(|_| args.next()) else {
                        core::mem::drop(args);
                        let end = self.peek_next_span().unwrap_or_else(|| self.end_span()).end;
                        self.new_parse_error(start..end, "Invalid number of arguments to Alloc")
                            .set_long_message("Alloc expects a single argument.");
                        return None;
                    };
                    size
                };
                Expr::Intrinsic(Box::new(Intrinsic::Alloc { size }))
            }
            Token::Cast => {
                self.expect(&Token::Lt)?;
                let ty = self.parse_type()?;
                self.expect(&Token::Gt)?;
                self.expect(&Token::LParen)?;
                let val = self.parse_expr(BindingPower::None)?;
                self.expect(&Token::RParen)?;
                Expr::Intrinsic(Box::new(Intrinsic::Cast { ty, val }))
            }
            Token::Sizeof => {
                self.expect(&Token::Lt)?;
                let before = self.errors.components.len();
                let sizeof = if let Some(ty) = self.parse_type() {
                    SizeOf::Type(ty)
                } else {
                    let expr = self.parse_expr(BindingPower::None)?;
                    SizeOf::Val(expr)
                };
                self.errors.components.truncate(before);
                self.expect(&Token::Gt)?;
                Expr::Intrinsic(Box::new(Intrinsic::SizeOf(sizeof)))
            }
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
    pub(crate) fn expect_ident(&mut self, ctx: impl Display) -> Option<&'s str> {
        let (t, span) = self.advance_if_split(|t| matches!(t, Token::Ident(_)));
        let Some(Token::Ident(val)) = t else {
            self.new_parse_error(span, format!("Expected an identifier{ctx} found {t:?}"));
            return None;
        };
        Some(val)
    }
    pub(crate) fn postfix_expr(&mut self, lhs: Expr<'s>) -> Option<Expr<'s>> {
        let (next, _) = self.advance_split();
        let next = next.unwrap();

        Some(match next {
            Token::Dot => {
                let member = self.expect_ident(" after . member access")?;
                Expr::MemberAccess(Box::new(ast::MemberAccess {
                    object: lhs,
                    member,
                }))
            }
            Token::ColonColon => {
                let member = self.expect_ident(" after :: namespace access")?;
                Expr::NamespaceAccess(Box::new(ast::NamespaceAccess {
                    object: lhs,
                    member,
                }))
            }
            t @ (Token::PlusPlus | Token::MinusMinus) => {
                let op = if t == Token::PlusPlus {
                    UnaryOpKind::PostInc
                } else {
                    UnaryOpKind::PostDec
                };
                Expr::UnaryOp(Box::new(ast::UnaryOp { val: lhs, op }))
            }
            Token::LBracket => {
                let index = self.parse_expr(BindingPower::Lowest)?;
                self.expect(&Token::RBracket)?;
                Expr::Intrinsic(Box::new(Intrinsic::Index { target: lhs, index }))
            }
            _ => {
                unreachable!()
            }
        })
    }
    /// Applies operations to the given left-hand side, if they have a lower binding power than the
    /// context.
    /// Should only be called when more tokens are available
    pub(crate) fn left_denotation(&mut self, lhs: Expr<'s>) -> Option<Expr<'s>> {
        let (next, span) = self.peek_next_split();
        let next = next.unwrap();
        Some(match next {
            Token::LParen => {
                _ = self.advance();
                let args: Vec<Expr<'s>> = self
                    .delimited_list_with_terminator(
                        |p| p.parse_expr(BindingPower::Lowest),
                        &Token::Comma,
                        &Token::RParen,
                    )
                    .collect();
                Expr::Call(ast::Call {
                    callee: Box::new(lhs),
                    args,
                })
            }
            Token::Eq => {
                _ = self.advance();
                let val = self.parse_expr(BindingPower::Assign)?;
                Expr::Assignment(Box::new(ast::Assignment { target: lhs, val }))
            }
            Token::Dot
            | Token::ColonColon
            | Token::PlusPlus
            | Token::MinusMinus
            | Token::LBracket => self.postfix_expr(lhs)?,
            t if token_to_bop(t).is_some() => {
                let op = token_to_bop(t).unwrap();
                let bp = BindingPower::from_token(t);
                _ = self.advance();
                let rhs = self.parse_expr(bp)?;
                Expr::BinOp(Box::new(ast::BinOp { lhs, rhs, op }))
            }
            t => {
                let msg = format!("Unexpected token for binary operation: {t}");
                self.new_parse_error(span, "Internal error")
                    .set_long_message(msg);
                return None;
            }
        })
    }
    pub fn parse_expr(&mut self, outer_bp: BindingPower) -> Option<Expr<'s>> {
        let mut left = self.null_denotation()?;
        loop {
            let next = self.peek_next().map(|s| &s.inner);
            let new_bp = next.map(BindingPower::from_token).unwrap_or_default();
            if new_bp <= outer_bp {
                break;
            }
            left = self.left_denotation(left)?;
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
            Lt | Le | Gt | Ge => BP::Relational,
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

fn token_to_bop(tok: &Token<'_>) -> Option<BinOpKind> {
    use BinOpKind as B;
    use Token as T;
    Some(match tok {
        T::Minus => B::Sub,
        T::Plus => B::Add,
        T::Star => B::Mul,
        T::Ampersand => B::BitAnd,
        Token::Slash => B::Div,
        Token::Percent => B::Mod,
        Token::EqEq => B::Eq,
        Token::Ne => B::Ne,
        Token::Gt => B::Gt,
        Token::Lt => B::Lt,
        Token::Ge => B::Ge,
        Token::Le => B::Le,
        Token::RShift => B::Shr,
        Token::LShift => B::Shl,
        Token::BitOr => B::BitOr,
        Token::BitXor => B::BitXor,
        Token::And => B::And,
        Token::Or => B::Or,
        Token::Range => B::Range,
        _ => return None,
    })
}
