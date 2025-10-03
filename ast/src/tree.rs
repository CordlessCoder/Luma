use std::fmt::{self, Display, Write};

use crate::{
    Assignment, BinOp, BinOpKind, Block, Call, Elif, Expr, Function, Intrinsic, LiteralExpression,
    MemberAccess, Module, NamespaceAccess, SizeOf, Stmt, StructField, SwitchCase, Ternary, Type,
    TypeDecl, UnaryOp, UnaryOpKind,
};

#[derive(Debug, Clone, Copy)]
enum Section {
    Middle,
    Last,
}

#[derive(Debug, Clone)]
pub struct TreeCtx {
    prefix: Vec<Section>,
    pub empty: &'static str,
    pub connector: &'static str,
    pub vertical: &'static str,
    pub end_connector: &'static str,
}
impl TreeCtx {
    pub const fn new() -> Self {
        Self {
            prefix: Vec::new(),
            empty: "   ",
            connector: "├──",
            vertical: "│  ",
            end_connector: "└──",
        }
    }
    fn add_level(&mut self) {
        self.prefix.push(Section::Middle);
    }
    fn make_last(&mut self) {
        *self.prefix.last_mut().unwrap() = Section::Last;
    }
    fn pop_level(&mut self) {
        self.prefix.pop().unwrap();
    }
}

impl Default for TreeCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeCtx {
    fn struct_header(&self, writer: &mut impl Write, name: impl Display) -> fmt::Result {
        self.write_identation(writer)?;
        writeln!(writer, "{name}")
    }
    fn write_identation(&self, writer: &mut impl Write) -> fmt::Result {
        for section in self.prefix.iter().rev().skip(1).rev() {
            let text = match section {
                Section::Middle => self.vertical,
                Section::Last => self.empty,
            };
            writer.write_str(text)?;
        }
        if let Some(last) = self.prefix.last() {
            let text = match last {
                Section::Middle => self.connector,
                Section::Last => self.end_connector,
            };
            writer.write_str(text)?;
        }
        Ok(())
    }
    fn fmt_single_field(
        &mut self,
        writer: &mut impl Write,
        name: &str,
        val: &impl TreeDisplay,
    ) -> fmt::Result {
        self.struct_header(writer, name)?;
        self.add_level();
        self.make_last();
        val.fmt_tree(self, writer)?;
        self.pop_level();
        Ok(())
    }
}

pub trait TreeDisplay {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result;
}

impl<T: TreeDisplay> TreeDisplay for Option<&T> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        let Some(s) = self else { return Ok(()) };
        s.fmt_tree(ctx, writer)
    }
}

impl<T: TreeDisplay> TreeDisplay for &[T] {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        let Some((last, rest)) = self.split_last() else {
            return Ok(());
        };
        ctx.add_level();
        for val in rest {
            val.fmt_tree(ctx, writer)?;
        }
        ctx.make_last();
        last.fmt_tree(ctx, writer)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for &str {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.write_identation(writer)?;
        writeln!(writer, "{self:?}")
    }
}

impl TreeDisplay for u64 {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.write_identation(writer)?;
        writeln!(writer, "{self}")
    }
}

impl TreeDisplay for f64 {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.write_identation(writer)?;
        writeln!(writer, "{self}")
    }
}

impl TreeDisplay for char {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.write_identation(writer)?;
        writeln!(writer, "{self:?}")
    }
}

impl TreeDisplay for bool {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.write_identation(writer)?;
        writeln!(writer, "{self}")
    }
}

impl TreeDisplay for Elif<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Elif")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Cond", &self.cond)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Body", &self.body)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for StructField<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Field")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Name", &self.name)?;
        ctx.fmt_single_field(writer, "Public", &self.public)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Type", &self.ty)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for TypeDecl<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use TypeDecl::*;
        match self {
            Struct { fields } => ctx.fmt_single_field(writer, "Struct", &&fields[..]),
            Enum { members } => ctx.fmt_single_field(writer, "Enum", &&members[..]),
        }
    }
}

impl TreeDisplay for SwitchCase<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Case")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Cases", &&self.cases[..])?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Body", &self.body)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Stmt<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use Stmt::*;
        match self {
            Break => ctx.struct_header(writer, "Break")?,
            Continue => ctx.struct_header(writer, "Continue")?,
            Expr(e) => e.fmt_tree(ctx, writer)?,
            Return(e) => ctx.fmt_single_field(writer, "Return", &e.as_ref())?,
            Function(f) => ctx.fmt_single_field(writer, "Function", f)?,
            Use { module, alias } => {
                ctx.struct_header(writer, "Use")?;
                ctx.add_level();
                if alias.is_none() {
                    ctx.make_last();
                }
                ctx.fmt_single_field(writer, "Module", module)?;
                if let Some(alias) = alias {
                    ctx.make_last();
                    ctx.fmt_single_field(writer, "Alias", alias)?;
                }
                ctx.pop_level();
            }
            If {
                cond,
                then_body,
                elifs,
                else_body,
            } => {
                ctx.struct_header(writer, "If")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Cond", cond)?;
                ctx.fmt_single_field(writer, "Then", then_body)?;
                ctx.fmt_single_field(writer, "Elifs", &&elifs[..])?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Else", &else_body.as_ref())?;
                ctx.pop_level();
            }
            VarDecl {
                name,
                ty,
                init,
                public,
                mutable,
            } => {
                ctx.struct_header(writer, "Var")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Name", name)?;
                ctx.fmt_single_field(writer, "Type", ty)?;
                if let Some(init) = init {
                    ctx.fmt_single_field(writer, "Init", init)?;
                }
                ctx.fmt_single_field(writer, "Public", public)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Mutable", mutable)?;
                ctx.pop_level();
            }
            Block(b) => b.fmt_tree(ctx, writer)?,
            TypeDecl { ty, public, name } => {
                ctx.struct_header(writer, "Type Declaration")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Name", name)?;
                ctx.fmt_single_field(writer, "Public", public)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Type", ty)?;
                ctx.pop_level();
            }
            Defer(s) => ctx.fmt_single_field(writer, "Defer", &**s)?,
            Print { values, newline } => {
                ctx.struct_header(writer, "Print")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Newline", newline)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Values", &&values[..])?;
                ctx.pop_level();
            }
            Loop {
                cond,
                initializers,
                post_ops,
                body,
            } => {
                ctx.struct_header(writer, "Loop")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Cond", &cond.as_ref())?;
                ctx.fmt_single_field(writer, "Initializers", initializers)?;
                ctx.fmt_single_field(writer, "Post ops", post_ops)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Body", body)?;
                ctx.pop_level();
            }
            Switch {
                value,
                cases,
                default,
            } => {
                ctx.struct_header(writer, "Switch")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Value", value)?;
                ctx.fmt_single_field(writer, "Default", &default.as_deref())?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Cases", &&cases[..])?;
                ctx.pop_level();
            }
        }
        Ok(())
    }
}

impl TreeDisplay for UnaryOp<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        self.op.fmt_tree(ctx, writer)?;
        ctx.add_level();
        ctx.make_last();
        self.val.fmt_tree(ctx, writer)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for BinOp<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        self.op.fmt_tree(ctx, writer)?;
        ctx.add_level();
        self.lhs.fmt_tree(ctx, writer)?;
        ctx.make_last();
        self.rhs.fmt_tree(ctx, writer)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Ternary<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Ternary")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Cond", &self.cond)?;
        ctx.fmt_single_field(writer, "Then", &self.then_val)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Else", &self.else_val)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Call<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Call")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Callee", &*self.callee)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Args", &&self.args[..])?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Assignment<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Assignment")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Target", &self.target)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Value", &self.val)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for MemberAccess<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, ".")?;
        ctx.add_level();
        self.object.fmt_tree(ctx, writer)?;
        ctx.make_last();
        self.member.fmt_tree(ctx, writer)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for NamespaceAccess<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "::")?;
        ctx.add_level();
        self.object.fmt_tree(ctx, writer)?;
        ctx.make_last();
        self.member.fmt_tree(ctx, writer)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for SizeOf<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        match self {
            SizeOf::Val(v) => ctx.fmt_single_field(writer, "SizeOf value", v),
            SizeOf::Type(ty) => ctx.fmt_single_field(writer, "SizeOf type", ty),
        }
    }
}

impl TreeDisplay for Intrinsic<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use Intrinsic::*;
        match self {
            Addr { of } => ctx.fmt_single_field(writer, "Addrof", of)?,
            Index { target, index } => {
                ctx.struct_header(writer, "Indexed")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "Target", target)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Idx", index)?;
                ctx.pop_level();
            }
            Deref { addr } => ctx.fmt_single_field(writer, "Deref", addr)?,
            Alloc { size } => ctx.fmt_single_field(writer, "Alloc", size)?,
            Free { ptr } => ctx.fmt_single_field(writer, "Free", ptr)?,
            Memcpy { from, to, size } => {
                ctx.struct_header(writer, "Memcpy")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "From", from)?;
                ctx.fmt_single_field(writer, "To", to)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Size", size)?;
                ctx.pop_level();
            }
            Cast { ty, val } => {
                ctx.struct_header(writer, "Cast")?;
                ctx.add_level();
                ctx.fmt_single_field(writer, "To", ty)?;
                ctx.make_last();
                ctx.fmt_single_field(writer, "Value", val)?;
                ctx.pop_level();
            }
            SizeOf(s) => s.fmt_tree(ctx, writer)?,
        }
        Ok(())
    }
}

impl TreeDisplay for Expr<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use Expr::*;
        match self {
            Lit(lit) => lit.fmt_tree(ctx, writer),
            Ident(name) => ctx.fmt_single_field(writer, "Ident", name),
            BinOp(op) => op.fmt_tree(ctx, writer),
            UnaryOp(op) => op.fmt_tree(ctx, writer),
            Ternary(tern) => tern.fmt_tree(ctx, writer),
            Call(c) => c.fmt_tree(ctx, writer),
            Assignment(a) => a.fmt_tree(ctx, writer),
            MemberAccess(a) => a.fmt_tree(ctx, writer),
            NamespaceAccess(n) => n.fmt_tree(ctx, writer),
            Group(g) => ctx.fmt_single_field(writer, "Group", &**g),
            Array(val) => ctx.fmt_single_field(writer, "Array", &&val[..]),
            Intrinsic(i) => i.fmt_tree(ctx, writer),
        }
    }
}
impl TreeDisplay for LiteralExpression<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use LiteralExpression::*;
        ctx.write_identation(writer)?;
        writeln!(writer, "Literal")?;
        ctx.add_level();
        ctx.make_last();
        match self {
            Str(text) => ctx.fmt_single_field(writer, "Text", &&text[..])?,
            Int(i) => ctx.fmt_single_field(writer, "Int", i)?,
            Float(f) => ctx.fmt_single_field(writer, "Float", f)?,
            Char(c) => ctx.fmt_single_field(writer, "Char", c)?,
            Bool(b) => ctx.fmt_single_field(writer, "Bool", b)?,
        }
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Block<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Block")?;
        self.0.as_slice().fmt_tree(ctx, writer)
    }
}

impl TreeDisplay for Function<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.pop_level();
        ctx.add_level();
        let crate::Function { body, params, ret } = self;
        ctx.struct_header(writer, "Params")?;
        ctx.add_level();
        for (ty, name) in params.iter().take(params.len().saturating_sub(1)) {
            ctx.fmt_single_field(writer, name, ty)?;
        }
        ctx.make_last();
        if let Some((ty, name)) = params.last() {
            ctx.fmt_single_field(writer, name, ty)?;
        }
        ctx.pop_level();
        ctx.fmt_single_field(writer, "Return", ret)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Body", body)?;
        Ok(())
    }
}

impl TreeDisplay for Type<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use Type::*;
        let name = match self {
            Void => "void",
            Bool => "bool",
            Char => "char",
            Int => "int",
            UInt => "uint",
            Float => "float",
            Double => "double",
            Str => "str",
            Named(_) => "Named",
            Pointer { .. } => "Pointer",
            Array(_, _) => "Array",
            Function(_) => "Function",
        };
        ctx.write_identation(writer)?;
        writeln!(writer, "Type: {name}")?;
        ctx.add_level();
        match self {
            Named(..) | Pointer { .. } | Array(..) | Function(..) => {
                ctx.make_last();
            }
            _ => (),
        };
        match self {
            Named(name) => {
                ctx.write_identation(writer)?;
                writer.write_str(name)?;
            }
            Pointer { pointee } => pointee.fmt_tree(ctx, writer)?,
            Array(ty, len) => {
                ctx.fmt_single_field(writer, "Length", len)?;
                ctx.fmt_single_field(writer, "Type", &**ty)?;
            }
            Function(func) => {
                func.fmt_tree(ctx, writer)?;
            }
            _ => (),
        }
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for Module<'_> {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        ctx.struct_header(writer, "Module")?;
        ctx.add_level();
        ctx.fmt_single_field(writer, "Name", &self.name)?;
        ctx.make_last();
        ctx.fmt_single_field(writer, "Body", &self.body)?;
        ctx.pop_level();
        Ok(())
    }
}

impl TreeDisplay for UnaryOpKind {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use UnaryOpKind::*;
        let text = match self {
            Not => "!",
            Neg => "-",
            Pos => "+",
            BitNot => "~",
            PreInc => "++x",
            PreDec => "--x",
            PostInc => "x++",
            PostDec => "x--",
            Deref => "*x",
            Addr => "&x",
        };
        ctx.struct_header(writer, text)
    }
}

impl TreeDisplay for BinOpKind {
    fn fmt_tree(&self, ctx: &mut TreeCtx, writer: &mut impl Write) -> fmt::Result {
        use BinOpKind::*;
        let text = match self {
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Pow => "**",
            Eq => "==",
            Ne => "!=",
            Lt => "<",
            Le => "<=",
            Gt => ">",
            Ge => ">=",
            And => "&&",
            Or => "||",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            Shl => "<<",
            Shr => ">>",
            Range => "..",
        };
        ctx.struct_header(writer, text)
    }
}
