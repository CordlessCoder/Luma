use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Expression<'s> {
    Lit(LiteralExpression<'s>),
    // Fn(Function<'s>),
}

#[derive(Debug, Clone)]
pub enum LiteralExpression<'s> {
    Str(Cow<'s, str>),
    Int(i64),
    Float(f64),
    Char(char),
}

// #[derive(Debug, Clone)]
// pub struct Function<'s> {
//     pub params: Vec<&'s str>,
//     pub body: Vec<Statement<'s>>,
// }
// #[derive(Debug, Clone)]
// pub struct Program<'s> {
//     pub filename: &'s str,
//     pub statements: Vec<Statement<'s>>,
// }
//
// #[derive(Debug, Clone)]
// pub enum Statement<'s> {
//     Expr(Expression<'s>),
//     Declaration(),
//     Return(Option<Expression<'s>>),
//     Continue,
//     Break,
// }

// #[derive(Debug, Clone)]
// pub enum Type<'s> {
//     Tuple(TupleType<'s>),
//     Path(TypePath<'s>),
// }
//
// #[derive(Debug, Clone)]
// pub struct TypePath<'s> {
//     pub components: Vec<&'s str>,
// }
//
// #[derive(Debug, Clone)]
// pub struct TupleType<'s> {
//     pub fields: Vec<Type<'s>>,
// }
