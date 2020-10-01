mod builder;

use paste::paste;
use std::ops::Range;

use self::builder::DiagnosticBuilder;

#[derive(Clone, Copy)]
enum Level {
    ICE,
    Error,
    Warning,
    Help,
    Info,
    Note
}

macro_rules! build_fn {
    ($name:ident, $level:expr) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<build_ $name>](&self, message: &str) -> DiagnosticBuilder {
                DiagnosticBuilder::new(message.to_string(), $level, self)
            }

            #[allow(dead_code)]
            pub fn [<build_ $name _span>](&self, span: Range<usize>, message: &str) -> DiagnosticBuilder {
                self.[<build_ $name>](message).with_span(span)
            }
        }
    };
}

pub struct DiagnosticsContext<'src> {
    source: &'src str,
    origin: Option<String>
}

impl<'src> DiagnosticsContext<'src> {
    pub fn new(source: &'src str, origin: Option<String>) -> Self {
        Self { source, origin }
    }

    #[allow(dead_code)]
    pub fn build_ice(&self, message: &str) -> DiagnosticBuilder {
        DiagnosticBuilder::new(message.to_string(), Level::ICE, self)
            .note("this is an internal error")
    }

    #[allow(dead_code)]
    pub fn build_ice_span(&self, span: Range<usize>, message: &str) -> DiagnosticBuilder {
        self.build_ice(message).with_span(span)
    }

    build_fn!(error, Level::Error);
    build_fn!(warning, Level::Warning);
    build_fn!(help, Level::Help);
    build_fn!(info, Level::Info);
    build_fn!(note, Level::Note);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let ctx = DiagnosticsContext::new("(lambda (x) 1 + x)", None);
        ctx.build_error("use of infix operator detected")
            .span_label(12..17, "no")
            .help("don't do that")
            .note("wouldn't have happened in Rust")
            .emit();
    }
}
