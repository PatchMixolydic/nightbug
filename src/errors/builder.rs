use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation}
};
use std::ops::Range;

use super::{DiagnosticsContext, Level};

impl From<Level> for AnnotationType {
    fn from(level: Level) -> Self {
        match level {
            Level::ICE | Level::Error => AnnotationType::Error,
            Level::Warning => AnnotationType::Warning,
            Level::Help => AnnotationType::Help,
            Level::Info => AnnotationType::Info,
            Level::Note => AnnotationType::Note
        }
    }
}

struct Label {
    contents: Option<String>,
    level: Level,
    span: Range<usize>
}

impl<'label> From<&'label Label> for Annotation<'label> {
    fn from(label: &'label Label) -> Self {
        Annotation {
            label: Some(label.contents.as_ref().map_or("", |x| x.as_str())),
            id: None,
            annotation_type: label.level.into()
        }
    }
}

impl<'label> From<&'label Label> for SourceAnnotation<'label> {
    fn from(label: &'label Label) -> Self {
        SourceAnnotation {
            label: label.contents.as_ref().map_or("", |x| x.as_str()),
            range: (label.span.start, label.span.end),
            annotation_type: label.level.into()
        }
    }
}

#[must_use = "must emit the diagnostic for it to be seen"]
pub struct DiagnosticBuilder<'ctx, 'src> {
    title: String,
    level: Level,
    labels: Vec<Label>,
    footers: Vec<Label>,
    context: &'ctx DiagnosticsContext<'src>
}

#[allow(dead_code)]
impl<'ctx, 'src> DiagnosticBuilder<'ctx, 'src> {
    pub(super) fn new(
        title: String,
        level: Level,
        context: &'ctx DiagnosticsContext<'src>
    ) -> Self {
        Self {
            title,
            level,
            labels: Vec::new(),
            footers: Vec::new(),
            context
        }
    }

    pub fn span_label(mut self, span: Range<usize>, message: &str) -> Self {
        self.labels.push(Label {
            contents: Some(message.to_string()),
            level: self.level,
            span
        });
        self
    }

    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.labels.push(Label {
            contents: None,
            level: self.level,
            span
        });
        self
    }

    pub fn help(mut self, message: &str) -> Self {
        self.footers.push(Label {
            contents: Some(message.to_string()),
            level: Level::Help,
            span: 0..0
        });
        self
    }

    pub fn note(mut self, message: &str) -> Self {
        self.footers.push(Label {
            contents: Some(message.to_string()),
            level: Level::Note,
            span: 0..0
        });
        self
    }

    pub fn emit(self) {
        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(&self.title),
                id: None,
                annotation_type: self.level.into()
            }),

            footer: self.footers.iter().map(Annotation::from).collect(),

            slices: vec![Slice {
                source: self.context.source,
                line_start: 1,
                origin: self.context.origin.as_ref().map(|x| x.as_str()),
                fold: true,
                annotations: self.labels.iter().map(SourceAnnotation::from).collect()
            }],

            opt: FormatOptions {
                color: true,
                ..Default::default()
            }
        };

        let dl = DisplayList::from(snippet);
        // TODO: customizable output
        eprintln!("{}", dl);
    }
}
