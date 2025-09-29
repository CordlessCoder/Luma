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
