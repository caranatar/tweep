use crate::context::{ContextPosition, InnerContext};
use std::pin::Pin;

/// Wraps a [`Pin`]ned, heap-allocated [`InnerContext`]
///
/// Provides an implementation of `subcontext` that does not require unsafe code
/// at the call site, `Deref`s to an [`InnerContext`] to provide all other
/// functionality.
///
/// [`Pin`]: std::pin::Pin
/// [`InnerContext`]: struct.InnerContext.html
pub struct FullContext<'a> {
    inner: Pin<Box<InnerContext<'a>>>,
}

impl<'a> FullContext<'a> {
    /// Creates a new `FullContext` from the given optional filename and contents
    pub fn from(file_name: Option<String>, contents: String) -> Self {
        let inner = InnerContext::from(file_name, contents);
        FullContext { inner }
    }

    /// Creates a subcontext out of the current context from the inclusive,
    /// 1-indexed start and end positions
    pub fn subcontext<T>(
        &self,
        range: T
    ) -> Self where T: SubContextRange {
        let (start, end) = range.into(self).into_inner();
        let context_ref = self.inner.self_ref.clone();
        let subcontext =
            unsafe { (&*context_ref.as_ptr()).subcontext(start, end) };

        FullContext { inner: subcontext }
    }
}

impl<'a> std::ops::Deref for FullContext<'a> {
    type Target = InnerContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

use std::ops::RangeInclusive;
use std::ops::RangeBounds;
use std::ops::Bound;
pub trait SubContextRange {
    fn into(self, context: &FullContext) -> RangeInclusive<ContextPosition>;
}

fn bound_to_position(ctx: &FullContext, pos: Bound<&ContextPosition>, start: bool) -> ContextPosition {
    match pos {
        Bound::Included(p) => *p,
        Bound::Excluded(_) => panic!("TODO"),
        Bound::Unbounded => if start {
            ContextPosition::new(1, 1)
        } else {
            let line = ctx.get_end_position().line - ctx.get_start_position().line + 1;
            let col = ctx.get_end_position().column;
            ContextPosition::new(line, col)
        }
    }
}

impl<T> SubContextRange for T where T: RangeBounds<ContextPosition> {
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
        let c = FullContext::from(None,owned);
        assert!(c.is_owned());
        assert_eq!(c.get_contents(), "hello");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 5));
    }

    #[test]
    fn subcontext() {
        let owned = "Hail Eris".to_string();
        let c = FullContext::from(None, owned);
        assert!(c.is_owned());
        assert_eq!(c.get_contents(), "Hail Eris");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 9));

        let sub = c.subcontext(ContextPosition::new(1, 6)..=ContextPosition::new(1, 9));
        assert!(sub.is_borrowed());
        assert_eq!(sub.get_contents(), "Eris");
        assert_eq!(*sub.get_file_name(), None);
        assert_eq!(*sub.get_start_position(), ContextPosition::new(1, 6));
        assert_eq!(*sub.get_end_position(), ContextPosition::new(1, 9));
    }
}
