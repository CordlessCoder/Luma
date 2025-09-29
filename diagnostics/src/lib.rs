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
    pub components: Vec<ErrorComponent>,
}

impl AggregateError {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
    pub fn has_error(&self) -> bool {
        self.components.iter().any(|c| c.level == ErrorLevel::Error)
    }
    pub fn add_error(&mut self, component: ErrorComponent) -> &mut ErrorComponent {
        self.components.push(component);
        self.components.last_mut().unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ErrorComponent {
    pub level: ErrorLevel,
    pub short_message: String,
    pub long_message: String,
    source: SourceFile,
    highlight: Span,
    highlight_message: Option<String>,
}

impl ErrorComponent {
    pub fn new(source: SourceFile, short_message: String, span: Span) -> Self {
        ErrorComponent {
            short_message,
            level: ErrorLevel::Error,
            long_message: String::new(),
            source,
            highlight: span,
            highlight_message: None,
        }
    }
    pub fn set_highlight_message(&mut self, message: impl ToString) -> &mut Self {
        self.highlight_message = Some(message.to_string());
        self
    }
    pub fn set_level(&mut self, level: ErrorLevel) -> &mut Self {
        self.level = level;
        self
    }
    pub fn set_long_message(&mut self, message: impl ToString) -> &mut Self {
        self.long_message = message.to_string();
        self
    }
}
