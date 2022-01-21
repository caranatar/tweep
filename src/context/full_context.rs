use crate::context::Position;
use crate::context::PositionKind;
use std::borrow::Borrow;
use std::sync::Arc;

/// A context that represents a span of twee code with a beginning, end, and
/// contents, along with a file name and some helper functions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullContext {
    file_name: Option<String>,
    start_position: Position,
    end_position: Position,
    contents: Arc<String>,
    line_starts: Arc<Vec<usize>>,
}

mod util {
    use super::Position;

    pub(crate) fn line_starts<'a>(s: &'a str) -> impl 'a + Iterator<Item = usize> {
        std::iter::once(0).chain(s.match_indices('\n').map(|(i, _)| i + 1))
    }

    pub(crate) fn to_byte_index(p: &Position, line_starts: &[usize], inclusive: bool) -> usize {
        let mut x = line_starts[p.line - 1] + p.column;
        if !inclusive {
            x -= 1;
        }
        x
    }

    pub(crate) fn end_of_line(line: usize, line_starts: &[usize], contents: &str) -> Position {
        let start = line_starts[line - 1];
        let len = if line >= line_starts.len() {
            contents[start..].len()
        } else {
            // Don't want ending newline
            line_starts[line] - start - 1
        };
        Position::abs(line, len)
    }
}

impl FullContext {
    pub(crate) fn new_with_line_starts(
        file_name: Option<String>,
        start_position: Position,
        end_position: Position,
        contents: Arc<String>,
        line_starts: Arc<Vec<usize>>,
    ) -> Self {
        FullContext {
            file_name,
            start_position,
            end_position,
            contents,
            line_starts,
        }
    }

    /// Given a 1-indexed line number, returns a position at the end of the line
    pub(crate) fn end_of_line(&self, line: usize, kind: PositionKind) -> Position {
        let (line, col) = match kind {
            PositionKind::Absolute => (
                line,
                util::end_of_line(
                    line,
                    self.get_line_starts(),
                    self.contents.as_str().borrow(),
                )
                .column,
            ),
            PositionKind::Relative => {
                let line = self.get_start_position().subposition(line, 1).line;
                (
                    line,
                    util::end_of_line(
                        line,
                        self.get_line_starts(),
                        self.contents.as_str().borrow(),
                    )
                    .column,
                )
            }
        };
        Position::abs(line, col)
    }

    #[cfg(feature = "full-context")]
    pub(crate) fn line_bytes(&self, line: usize) -> std::ops::RangeInclusive<usize> {
        let (start, end) = self.line_range(line, PositionKind::Absolute).into_inner();
        let start_byte = util::to_byte_index(&start, &self.line_starts, false);
        let end_byte = util::to_byte_index(&end, &self.line_starts, false);
        start_byte..=end_byte
    }

    #[cfg(feature = "full-context")]
    pub(crate) fn line_range(
        &self,
        line: usize,
        kind: PositionKind,
    ) -> std::ops::RangeInclusive<Position> {
        let start = match kind {
            PositionKind::Absolute => Position::abs(line, 1),
            PositionKind::Relative => Position::rel(line, 1),
        };
        start..=self.end_of_line(line, kind)
    }

    /// Creates a new context from the given file name and string
    pub fn from(file_name: Option<String>, contents: String) -> Self {
        let line_starts = util::line_starts(&contents).collect::<Vec<usize>>();
        let start = Position::abs(1, 1);
        let end = util::end_of_line(line_starts.len(), &line_starts, &contents);
        Self::new_with_line_starts(
            file_name,
            start,
            end,
            Arc::new(contents),
            Arc::new(line_starts),
        )
    }

    /// Gets a reference to the optional file name
    pub fn get_file_name(&self) -> &Option<String> {
        &self.file_name
    }

    /// Gets a reference to the 1-indexed start position of this context
    pub fn get_start_position(&self) -> &Position {
        &self.start_position
    }

    /// Gets a reference to the inclusive 1-indexed end position of this context
    pub fn get_end_position(&self) -> &Position {
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
        start_position: Position,
        end_position: Position,
    ) -> Self {
        let contents = self.contents.clone();
        let line_starts = self.line_starts.clone();
        let start_position = match start_position.kind {
            PositionKind::Absolute => start_position,
            PositionKind::Relative => self
                .start_position
                .subposition(start_position.line, start_position.column),
        };
        let end_position = match end_position.kind {
            PositionKind::Absolute => end_position,
            PositionKind::Relative => self
                .start_position
                .subposition(end_position.line, end_position.column),
        };
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
    fn into(self, context: &FullContext) -> RangeInclusive<Position>;
}

fn bound_to_position(ctx: &FullContext, pos: Bound<&Position>, start: bool) -> Position {
    let ret = match pos {
        Bound::Included(p) => match p.kind {
            PositionKind::Absolute => *p,
            PositionKind::Relative => ctx.get_start_position().subposition(p.line, p.column),
        },
        Bound::Excluded(p) => {
            let abs = match p.kind {
                PositionKind::Absolute => *p,
                PositionKind::Relative => ctx.get_start_position().subposition(p.line, p.column),
            };

            if abs.column > 1 {
                Position {
                    column: abs.column - 1,
                    ..abs
                }
            } else if abs.line > 1 {
                let line = p.line - 1;
                let col = ctx.end_of_line(line, PositionKind::Absolute).column;
                Position::abs(line, col)
            } else {
                panic!("Tried to take exclusive range ending at: {:?}", abs);
            }
        }
        Bound::Unbounded => {
            if start {
                *ctx.get_start_position()
            } else {
                *ctx.get_end_position()
            }
        }
    };
    assert_eq!(ret.kind, PositionKind::Absolute);
    ret
}

impl<T> SubContextRange for T
where
    T: RangeBounds<Position>,
{
    fn into(self, ctx: &FullContext) -> RangeInclusive<Position> {
        let start = bound_to_position(ctx, self.start_bound(), true);
        let end = bound_to_position(ctx, self.end_bound(), false);
        start..=end
    }
}

#[cfg(test)]
mod tests {
    use super::FullContext;
    use super::Position;

    #[test]
    fn test_construction() {
        let owned = "hello".to_string();
        let c = FullContext::from(None, owned);
        assert_eq!(c.get_contents(), "hello");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), Position::abs(1, 1));
        assert_eq!(*c.get_end_position(), Position::abs(1, 5));
    }

    #[test]
    fn subcontext() {
        let owned = "Hail Eris".to_string();
        let c = FullContext::from(None, owned);
        assert_eq!(c.get_contents(), "Hail Eris");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), Position::abs(1, 1));
        assert_eq!(*c.get_end_position(), Position::abs(1, 9));

        let sub = c.subcontext(Position::rel(1, 6)..=Position::rel(1, 9));
        assert_eq!(sub.get_contents(), "Eris");
        assert_eq!(*sub.get_file_name(), None);
        assert_eq!(*sub.get_start_position(), Position::abs(1, 6));
        assert_eq!(*sub.get_end_position(), Position::abs(1, 9));
    }
}
