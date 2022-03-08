use std::{path::PathBuf, sync::Arc, borrow::Cow, cmp};

/// Represents a span of the source code in a specific file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    // The original source code.
    src: Arc<str>,
    // The byte position in the string of the start of the span.
    start: usize,
    // The byte position in the string of the end of the span.
    end: usize,
    // A reference counted pointer to the file from which this span originated.
    path: Option<Arc<PathBuf>>,
}

impl Span {
    pub fn from_pest(pest_span: pest::Span, path: Option<Arc<PathBuf>>) -> Span {
        Span {
            src: pest_span.input().clone(),
            start: pest_span.start(),
            end: pest_span.end(),
            path,
        }
    }

    /*
    pub fn from_pest_no_path(pest_span: pest::Span) -> Span {
        Span {
            span: pest_span,
            path: None,
        }
    }
    */

    pub fn new(src: Arc<str>, start: usize, end: usize, path: Option<Arc<PathBuf>>) -> Option<Span> {
        if src.get(start..end).is_none() {
            return None;
        }
        Some(Span { src, start, end, path })
    }

    /*
    pub fn new_no_path(src: Arc<str>, start: usize, end: usize) -> Option<Span> {
        Some(Span {
            span: pest::Span::new(src, start, end)?,
            path: None,
        })
    }
    */

    pub fn src(&self) -> &Arc<str> {
        &self.src
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn path(&self) -> Option<&Arc<PathBuf>> {
        self.path.as_ref()
    }

    pub fn path_str(&self) -> Option<Cow<'_, str>> {
        self.path.as_deref().map(|path| path.to_string_lossy())
    }

    pub fn start_pos(&self) -> pest::Position {
        pest::Position::new(self.src.clone(), self.start).unwrap()
    }

    pub fn end_pos(&self) -> pest::Position {
        pest::Position::new(self.src.clone(), self.end).unwrap()
    }

    pub fn split(&self) -> (pest::Position, pest::Position) {
        let start = self.start_pos();
        let end = self.end_pos();
        (start, end)
    }

    pub fn str(self) -> String {
        self.as_str().to_owned()
    }

    pub fn as_str(&self) -> &str {
        &self.src[self.start..self.end]
    }

    pub fn input(&self) -> &str {
        &self.src
    }

    /*
    pub fn path(&self) -> String {
        self.path
            .as_deref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "".to_string())
    }
    */

    pub fn trim(self) -> Span {
        let start_delta = self.as_str().len() - self.as_str().trim_start().len();
        let end_delta = self.as_str().len() - self.as_str().trim_end().len();
        Span {
            src: self.src,
            start: self.start + start_delta,
            end: self.end - end_delta,
            path: self.path,
        }
    }
}

/// This panics if the spans are not from the same file. This should
/// only be used on spans that are actually next to each other.
pub fn join_spans(s1: Span, s2: Span) -> Span {
    // FIXME(canndrew): This is horrifying. Where did it come from and why is it needed?
    if s1.as_str() == "core" {
        return s2;
    }

    assert!(
        Arc::ptr_eq(&s1.src, &s2.src) && s1.path == s2.path,
        "Spans from different files cannot be joined.",
    );

    Span {
        src: s1.src,
        start: cmp::min(s1.start, s2.start),
        end: cmp::max(s1.end, s2.end),
        path: s1.path,
    }
}
