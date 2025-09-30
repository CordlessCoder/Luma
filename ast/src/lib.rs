use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum BinOpKind {
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// **
    Pow,
    /// ==
    Eq,
    /// !=
    Ne,
    /// <
    Lt,
    /// <=
    Le,
    /// >
    Gt,
    /// >=
    Ge,
    /// &&
    And,
    /// ||
    Or,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// ^
    BitXor,
    /// <<
    Shl,
    /// >>
    Shr,
    /// ..
    Range,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpKind {
    /// !
    Not,
    /// -
    Neg,
    /// +
    Pos,
    /// ~
    BitNot,
    /// ++x
    PreInc,
    /// --x
    PreDec,
    /// x++
    PostInc,
    /// x--
    PostDec,
    /// *x
    Deref,
    /// &x
    Addr,
}

#[derive(Debug, Clone)]
pub struct Function<'s, FieldName = ()> {
    pub params: Vec<(Type<'s>, FieldName)>,
    pub ret: Type<'s>,
}

#[derive(Debug, Clone)]
pub enum Type<'s> {
    Void,
    Bool,
    Char,
    Int,
    UInt,
    Float,
    Double,
    Str,
    Named(&'s str),
    Pointer { pointee: Box<Type<'s>> },
    Array(Box<Type<'s>>, Expr<'s>),
    Function(Box<Function<'s>>),
}

#[derive(Debug, Clone)]
pub struct StructField<'s> {
    pub name: &'s str,
    pub ty: Type<'s>,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub enum TypeDecl<'s> {
    Struct { fields: Vec<StructField<'s>> },
    Enum { members: Vec<&'s str> },
}

#[derive(Debug, Clone)]
pub struct Elif<'s> {
    pub cond: Expr<'s>,
    pub body: Block<'s>,
}

#[derive(Debug, Clone)]
pub struct Block<'s>(pub Vec<Stmt<'s>>);

#[derive(Debug, Clone)]
pub struct SwitchCase<'s> {
    pub body: Block<'s>,
    pub values: Vec<Expr<'s>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'s> {
    /// A function definition with parameter names
    Function(Function<'s, &'s str>),
    Use {
        module: &'s str,
        alias: Option<&'s str>,
    },
    VarDecl {
        name: &'s str,
        ty: Type<'s>,
        init: Option<Expr<'s>>,
        public: bool,
        mutable: bool,
    },
    Expr(Expr<'s>),
    If {
        cond: Expr<'s>,
        then_body: Block<'s>,
        elifs: Vec<Elif<'s>>,
        else_body: Option<Block<'s>>,
    },
    TypeDecl {
        ty: TypeDecl<'s>,
        public: bool,
        name: &'s str,
    },
    Loop {
        cond: Expr<'s>,
        initializers: Block<'s>,
        post_ops: Block<'s>,
        body: Block<'s>,
    },
    Return(Option<Expr<'s>>),
    Block(Block<'s>),
    Print {
        values: Vec<Expr<'s>>,
        newline: bool,
    },
    Break,
    Continue,
    Defer(Block<'s>),
    Switch {
        value: Expr<'s>,
        cases: Vec<SwitchCase<'s>>,
        default: Option<Block<'s>>,
    },
}

#[derive(Debug, Clone)]
pub struct Ternary<'s> {
    pub then_val: Expr<'s>,
    pub else_val: Expr<'s>,
    pub cond: Expr<'s>,
}

#[derive(Debug, Clone)]
pub struct BinOp<'s> {
    pub lhs: Expr<'s>,
    pub rhs: Expr<'s>,
    pub op: BinOpKind,
}

#[derive(Debug, Clone)]
pub struct UnaryOp<'s> {
    pub val: Expr<'s>,
    pub op: UnaryOpKind,
}

#[derive(Debug, Clone)]
pub struct Assignment<'s> {
    pub target: Expr<'s>,
    pub val: Expr<'s>,
}

#[derive(Debug, Clone)]
pub struct Call<'s> {
    pub callee: Box<Expr<'s>>,
    pub args: Vec<Expr<'s>>,
}

#[derive(Debug, Clone)]
pub struct MemberAccess<'s> {
    pub object: Expr<'s>,
    pub member: &'s str,
}

#[derive(Debug, Clone)]
pub struct NamespaceAccess<'s> {
    pub object: Expr<'s>,
    pub member: &'s str,
}

#[derive(Debug, Clone)]
pub enum SizeOf<'s> {
    Val(Expr<'s>),
    Type(Type<'s>),
}

#[derive(Debug, Clone)]
pub enum Intrinsic<'s> {
    Index {
        target: Expr<'s>,
        index: Expr<'s>,
    },
    Memcpy {
        from: Expr<'s>,
        to: Expr<'s>,
        size: Expr<'s>,
    },
    Deref {
        addr: Expr<'s>,
    },
    Addr {
        of: Expr<'s>,
    },
    Alloc {
        size: Expr<'s>,
    },
    Free {
        ptr: Expr<'s>,
    },
    Cast {
        ty: Type<'s>,
        val: Expr<'s>,
    },
    SizeOf(SizeOf<'s>),
}

#[derive(Debug, Clone)]
pub enum Expr<'s> {
    Lit(LiteralExpression<'s>),
    Ident(&'s str),
    BinOp(Box<BinOp<'s>>),
    UnaryOp(Box<UnaryOp<'s>>),
    Ternary(Box<Ternary<'s>>),
    Call(Call<'s>),
    Assignment(Box<Assignment<'s>>),
    MemberAccess(Box<MemberAccess<'s>>),
    NamespaceAccess(Box<NamespaceAccess<'s>>),
    Group(Box<Expr<'s>>),
    Array(Vec<Expr<'s>>),
    Intrinsic(Box<Intrinsic<'s>>),
}

#[derive(Debug, Clone)]
pub enum LiteralExpression<'s> {
    Str(Cow<'s, str>),
    Int(u64),
    Float(f64),
    Char(char),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Program<'s> {
    pub modules: Vec<Module<'s>>,
}

#[derive(Debug, Clone)]
pub struct Module<'s> {
    pub body: Block<'s>,
    pub name: &'s str,
}
