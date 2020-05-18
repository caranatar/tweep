use crate::context::ContextPosition;
use std::borrow::{Borrow, Cow};
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;

/// A self-referencing context to be [`Pin`]ned and wrapped in a [`FullContext`]
///
/// Holds an optional file name, 1-indexed start and end positions, and a `Cow`
/// that holds the contents of the context. This allows all of the necessary
/// information for both parsing, and if the appropriate feature(s) are enabled,
/// user-level displaying of errors and warnings, to be held in a single struct
/// with minimal unncessary copying.
#[derive(Debug, Eq)]
pub struct InnerContext<'a> {
    file_name: Option<String>,
    start_position: ContextPosition,
    end_position: ContextPosition,
    contents: Cow<'a, str>,
    line_starts: Cow<'a, [usize]>,
    pub(crate) self_ref: NonNull<InnerContext<'a>>,
    _pin: PhantomPinned,
}

impl<'a> PartialEq for InnerContext<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
            && self.start_position == other.start_position
            && self.end_position == other.end_position
            && self.contents == other.contents
            && self.line_starts == other.line_starts
    }
}

mod util {
    use super::ContextPosition;

    pub(crate) fn line_starts<'a>(s: &'a str) -> impl 'a + Iterator<Item = usize> {
        std::iter::once(0).chain(s.match_indices('\n').map(|(i, _)| i + 1))
    }

    pub(crate) fn to_byte_index(
        p: &ContextPosition,
        line_starts: &[usize],
        inclusive: bool,
    ) -> usize {
        let mut x = line_starts[p.line - 1] + p.column;
        if !inclusive {
            x -= 1;
        }
        x
    }

    pub(crate) fn end_of_line(
        line: usize,
        line_starts: &[usize],
        contents: &str,
    ) -> ContextPosition {
        let start = line_starts[line - 1];
        let len = if line >= line_starts.len() {
            contents[start..].len()
        } else {
            // Don't want ending newline
            line_starts[line] - start - 1
        };
        ContextPosition::new(line, len)
    }
}

impl<'a> InnerContext<'a> {
    pub(crate) fn new_with_line_starts<T: Into<Cow<'a, str>>, U: Into<Cow<'a, [usize]>>>(
        file_name: Option<String>,
        start_position: ContextPosition,
        end_position: ContextPosition,
        contents: T,
        line_starts: U,
    ) -> Pin<Box<Self>> {
        let contents = contents.into();
        let line_starts = line_starts.into();
        let res = InnerContext {
            file_name,
            start_position,
            end_position,
            contents,
            line_starts,
            self_ref: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        let mut boxed = Box::pin(res);

        let foo: NonNull<InnerContext> = NonNull::from(boxed.as_ref().get_ref());
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).self_ref = foo;
        }

        boxed
    }

    /// Given a 1-indexed line number, returns a position at the end of the line
    pub(crate) fn end_of_line(&self, line: usize) -> ContextPosition {
        ContextPosition::new(
            line,
            util::end_of_line(
                self.get_start_position().subposition(line, 1).line,
                self.get_line_starts(),
                self.contents.borrow(),
            )
            .column,
        )
    }

    pub(crate) fn line_range(&self, line: usize) -> std::ops::RangeInclusive<ContextPosition> {
        ContextPosition::new(line, 1)..=self.end_of_line(line)
    }

    pub(crate) fn from(file_name: Option<String>, contents: String) -> Pin<Box<Self>> {
        let line_starts = util::line_starts(&contents).collect::<Vec<usize>>();
        let start = ContextPosition::new(1, 1);
        let end = util::end_of_line(line_starts.len(), &line_starts, &contents);
        Self::new_with_line_starts(file_name, start, end, contents, line_starts)
    }

    /// Gets a reference to the optional file name
    pub fn get_file_name(&self) -> &Option<String> {
        &self.file_name
    }

    /// Gets a reference to the 1-indexed start position of this context
    pub fn get_start_position(&self) -> &ContextPosition {
        &self.start_position
    }

    /// Gets a reference to the inclusive 1-indexed end position of this context
    pub fn get_end_position(&self) -> &ContextPosition {
        &self.end_position
    }

    /// Gets a reference to the contents of this context
    pub fn get_contents(&self) -> &str {
        let start = util::to_byte_index(&self.start_position, &self.line_starts, false);
        let end = util::to_byte_index(&self.end_position, &self.line_starts, true);
        &self.contents[start..end]
    }

    pub(crate) fn get_line_starts(&self) -> &[usize] {
        self.line_starts.borrow()
    }

    pub(crate) fn subcontext(
        &'a self,
        start_position: ContextPosition,
        end_position: ContextPosition,
    ) -> Pin<Box<Self>> {
        let contents: &'a str = self.contents.borrow();
        let contents = Cow::from(contents);
        let line_starts: &'a [usize] = self.line_starts.borrow();
        let line_starts = Cow::from(line_starts);
        let start_position = self
            .start_position
            .subposition(start_position.line, start_position.column);
        let end_position = self
            .start_position
            .subposition(end_position.line, end_position.column);
        Self::new_with_line_starts(
            self.file_name.clone(),
            start_position,
            end_position,
            contents,
            line_starts,
        )
    }
}

#[cfg(test)]
impl InnerContext<'_> {
    pub fn is_contents_borrowed(&self) -> bool {
        match self.contents {
            Cow::Borrowed(_) => true,
            Cow::Owned(_) => false,
        }
    }

    pub fn is_line_starts_borrowed(&self) -> bool {
        match self.line_starts {
            Cow::Borrowed(_) => true,
            Cow::Owned(_) => false,
        }
    }

    pub fn is_borrowed(&self) -> bool {
        self.is_contents_borrowed() && self.is_line_starts_borrowed()
    }

    pub fn is_owned(&self) -> bool {
        !(self.is_contents_borrowed() || self.is_line_starts_borrowed())
    }
}
