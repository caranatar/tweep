use crate::context::ContextPosition;
use std::borrow::Borrow;
use std::rc::Rc;

/// A context that represents a span of twee code with a beginning, end, and
/// contents, along with a file name and some helper functions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullContext {
    file_name: Option<String>,
    start_position: ContextPosition,
    end_position: ContextPosition,
    contents: Rc<String>,
    line_starts: Rc<Vec<usize>>,
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
        line_starts: &Vec<usize>,
        contents: &String,
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

impl FullContext {
    pub(crate) fn new_with_line_starts(
        file_name: Option<String>,
        start_position: ContextPosition,
        end_position: ContextPosition,
        contents: Rc<String>,
        line_starts: Rc<Vec<usize>>,
    ) -> Self {
        let contents = contents.into();
        let line_starts = line_starts.into();
        let res = FullContext {
            file_name,
            start_position,
            end_position,
            contents,
            line_starts,
        };
        res
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

    pub(crate) fn line_bytes(&self, line: usize) -> std::ops::RangeInclusive<usize> {
        let (start, end) = self.line_range(line).into_inner();
        let start_byte = util::to_byte_index(&start, &self.line_starts, false);
        let end_byte = util::to_byte_index(&end, &self.line_starts, false);
        start_byte..=end_byte
    }
    
    pub(crate) fn line_range(&self, line: usize) -> std::ops::RangeInclusive<ContextPosition> {
        ContextPosition::new(line, 1)..=self.end_of_line(line)
    }

    /// Creates a new context from the given file name and string
    pub fn from(file_name: Option<String>, contents: String) -> Self {
        let line_starts = util::line_starts(&contents).collect::<Vec<usize>>();
        let start = ContextPosition::new(1, 1);
        let end = util::end_of_line(line_starts.len(), &line_starts, &contents);
        Self::new_with_line_starts(
            file_name,
            start,
            end,
            Rc::new(contents),
            Rc::new(line_starts),
        )
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

    /// Gets the span of this context as line bytes within the contents
    pub fn get_byte_range(&self) -> Range<usize> {
        let start = util::to_byte_index(&self.start_position, &self.line_starts, false);
        let end = util::to_byte_index(&self.end_position, &self.line_starts, true);
        start..end
    }
    
    /// Gets a reference to the contents of this context
    pub fn get_contents(&self) -> &str {
        let start = util::to_byte_index(&self.start_position, &self.line_starts, false);
        let mut end = util::to_byte_index(&self.end_position, &self.line_starts, true);
        if end < start {
            end = start;
        }
        &self.contents[start..end]
    }

    pub(crate) fn get_line_starts(&self) -> &Vec<usize> {
        self.line_starts.borrow()
    }

    /// Creates a subcontext out of the current context from the inclusive,
    /// 1-indexed start and end positions
    pub fn subcontext<T>(&self, range: T) -> Self
    where
        T: SubContextRange,
    {
        let (start, end) = range.into(self).into_inner();
        self.inner_subcontext(start, end)
    }

    pub(crate) fn inner_subcontext(
        &self,
        start_position: ContextPosition,
        end_position: ContextPosition,
    ) -> Self {
        let contents = self.contents.clone();
        let line_starts = self.line_starts.clone();
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

use std::ops::Bound;
use std::ops::Range;
use std::ops::RangeBounds;
use std::ops::RangeInclusive;
pub trait SubContextRange {
    fn into(self, context: &FullContext) -> RangeInclusive<ContextPosition>;
}

fn bound_to_position(
    ctx: &FullContext,
    pos: Bound<&ContextPosition>,
    start: bool,
) -> ContextPosition {
    match pos {
        Bound::Included(p) => *p,
        Bound::Excluded(p) => {
            if p.column <= 1 {
                if p.line <= 1 {
                    panic!("Bound position out of bounds");
                } else {
                    let line = p.line - 1;
                    let col = ctx.end_of_line(line).column;
                    ContextPosition::new(line, col)
                }
            } else {
                ContextPosition::new(p.line, p.column - 1)
            }
        }
        Bound::Unbounded => {
            if start {
                ContextPosition::new(1, 1)
            } else {
                let line = ctx.get_end_position().line - ctx.get_start_position().line + 1;
                let col = ctx.get_end_position().column;
                ContextPosition::new(line, col)
            }
        }
    }
}

impl<T> SubContextRange for T
where
    T: RangeBounds<ContextPosition>,
{
    fn into(self, ctx: &FullContext) -> RangeInclusive<ContextPosition> {
        let start = bound_to_position(ctx, self.start_bound(), true);
        let end = bound_to_position(ctx, self.end_bound(), false);
        start..=end
    }
}

#[cfg(test)]
mod tests {
    use super::ContextPosition;
    use super::FullContext;

    #[test]
    fn test_construction() {
        let owned = "hello".to_string();
        let c = FullContext::from(None, owned);
        assert_eq!(c.get_contents(), "hello");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 5));
    }

    #[test]
    fn subcontext() {
        let owned = "Hail Eris".to_string();
        let c = FullContext::from(None, owned);
        assert_eq!(c.get_contents(), "Hail Eris");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 9));

        let sub = c.subcontext(ContextPosition::new(1, 6)..=ContextPosition::new(1, 9));
        assert_eq!(sub.get_contents(), "Eris");
        assert_eq!(*sub.get_file_name(), None);
        assert_eq!(*sub.get_start_position(), ContextPosition::new(1, 6));
        assert_eq!(*sub.get_end_position(), ContextPosition::new(1, 9));
    }
}
