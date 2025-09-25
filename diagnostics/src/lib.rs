//! The main idea behind the diagnostic structure is that an error message can be composed of
//! multiple [ErrorComponent]s.
//! For example, an error message like:
//!
//!```rust,ignore
//!error[E0308]: mismatched types
//!   --> src/main.rs:2:18
//!    |
//! 2  |     let x: i32 = "42";
//!    |                  ^^^^ expected `i32`, found `&str`
//! ```
pub mod render;

// Errors can occur during:
// - Lexing
// - Parsing
// - Type checking
// - Codegen
// - Execution
//
// Everything that isn't execution can be a simple error message + point to source
//
// Errors during execution likely need extra information to describe the runtime context

use source::{SourceFile, Span};
#[derive(Debug, Clone, Default)]
pub struct AggregateError {
    components: Vec<ErrorComponent>,
}

impl AggregateError {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    pub fn add_error(&mut self, component: ErrorComponent) -> &mut ErrorComponent {
        self.components.push(component);
        self.components.last_mut().unwrap()
    }
}

impl AggregateError {}

#[derive(Debug, Clone)]
pub enum ErrorComponent {
    WithSource(ErrorWithSource),
}

impl From<ErrorWithSource> for ErrorComponent {
    fn from(value: ErrorWithSource) -> Self {
        ErrorComponent::WithSource(value)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorWithSource {
    message: String,
    source: SourceFile,
    highlight: Span,
    highlight_message: Option<String>,
}

impl ErrorWithSource {
    pub fn new(source: SourceFile, message: String, span: Span) -> Self {
        ErrorWithSource {
            message,
            source,
            highlight: span,
            highlight_message: None,
        }
    }
    pub fn set_highlight_message(&mut self, message: impl ToString) {
        self.highlight_message = Some(message.to_string());
    }
}
