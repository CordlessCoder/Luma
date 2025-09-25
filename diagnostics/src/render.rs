use owo_colors::OwoColorize;
use source::{Span, TextLine};

use crate::{AggregateError, ErrorComponent, ErrorWithSource};
use std::fmt::{Debug, Display, Write};

#[derive(Debug, Clone, Copy)]
pub struct RenderContext {
    pub width: u32,
    pub use_color: bool,
    pub lines_of_context: usize,
}

impl Default for RenderContext {
    fn default() -> Self {
        RenderContext {
            width: 80,
            use_color: false,
            lines_of_context: 1,
        }
    }
}

impl RenderContext {
    pub fn render_line(
        &self,
        writer: &mut impl Write,
        mut line: TextLine<'_>,
        // This span is relative to the line
        line_range: Span,
    ) -> std::fmt::Result {
        fn write_header(writer: &mut dyn Write, line: Option<u32>) -> std::fmt::Result {
            let Some(line) = line else {
                return write!(writer, "    {sep}", sep = '|'.magenta());
            };
            write!(
                writer,
                "{line:<4}{sep}",
                line = line.yellow(),
                sep = '|'.magenta()
            )
        }
        line.text = line.text.strip_suffix('\n').unwrap_or(line.text);
        let text = line.text;
        // Clamp end to start to ensure start <= end
        let line_range = line_range.start..line_range.end.max(line_range.start);
        if line_range.is_empty() {
            write_header(writer, Some(line.abs_line as u32 + 1))?;
            return write!(writer, "{text}");
        };
        let before = &line.text[..line_range.start];
        let highlighted = &line.text[line_range.clone()];
        let after = &line.text[line_range.end..];
        write_header(writer, Some(line.abs_line as u32 + 1))?;
        writeln!(writer, "{before}{}{after}", highlighted)?;
        write_header(writer, None)?;
        write!(writer, "{:width$}", "", width = before.chars().count())?;
        write!(
            writer,
            "{:^>width$}",
            "".red().bold(),
            width = highlighted.chars().count()
        )?;
        Ok(())
    }
}

pub struct RenderOnPrint<E: RenderableError> {
    pub err: E,
    pub ctx: RenderContext,
}

impl<E: RenderableError> RenderOnPrint<E> {
    pub fn new(err: E, ctx: RenderContext) -> Self {
        Self { err, ctx }
    }
}

impl<E: RenderableError> Display for RenderOnPrint<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.render(f, &self.ctx)?;
        Ok(())
    }
}
impl<E: RenderableError> Debug for RenderOnPrint<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.render(f, &self.ctx)?;
        Ok(())
    }
}

pub trait RenderableError: Sized {
    fn render(&self, writer: &mut impl Write, ctx: &RenderContext) -> std::fmt::Result;
    fn display(self, ctx: RenderContext) -> RenderOnPrint<Self> {
        RenderOnPrint::new(self, ctx)
    }
}

impl RenderableError for AggregateError {
    fn render(&self, mut writer: &mut impl Write, ctx: &RenderContext) -> std::fmt::Result {
        for err in &self.components {
            err.render(&mut writer, ctx)?;
        }
        Ok(())
    }
}

impl RenderableError for ErrorComponent {
    fn render(&self, writer: &mut impl Write, ctx: &RenderContext) -> std::fmt::Result {
        match self {
            ErrorComponent::WithSource(e) => e.render(writer, ctx)?,
        }
        Ok(())
    }
}

impl RenderableError for ErrorWithSource {
    fn render(&self, writer: &mut impl Write, ctx: &RenderContext) -> std::fmt::Result {
        let Self {
            message,
            source,
            highlight,
            highlight_message,
        } = self;
        let [start, end] = source.span_to_pos(highlight);
        let lines = {
            source
                .lines()
                .skip(start.line_0idx().saturating_sub(ctx.lines_of_context))
                .take_while(|t| t.abs_line <= end.line_0idx() + ctx.lines_of_context)
        };
        writeln!(writer, "{}", message.bold())?;
        write!(
            writer,
            "   {arrow} {path}:{line}:{col}",
            arrow = "-->".blue().bold(),
            path = source.path(),
            line = start.line().yellow(),
            col = start.col().yellow()
        )?;
        let mut prev_line_had_overlap = false;
        for line in lines {
            let overlap = highlight.start.max(line.span.start)..highlight.end.min(line.span.end);
            let overlap = overlap
                .start
                .saturating_sub(line.span.start)
                .min(line.text.len())
                ..overlap
                    .end
                    .saturating_sub(line.span.start)
                    .min(line.text.len());
            if let Some(highlight_message) = highlight_message
                && let true = overlap.is_empty()
                && let true = prev_line_had_overlap
            {
                write!(writer, "{}", highlight_message.red().bold())?;
            }
            writeln!(writer)?;
            ctx.render_line(writer, line, overlap.clone())?;
            prev_line_had_overlap = !overlap.is_empty();
        }
        writeln!(writer)?;
        Ok(())
    }
}
