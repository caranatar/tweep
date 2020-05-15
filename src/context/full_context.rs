use crate::context::{ContextPosition, InnerContext};
use std::borrow::Cow;
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
    /// Create a new `FullContext` with the given file name, start and end
    /// positions, and contents
    pub(crate) fn new<T: Into<Cow<'a, str>>>(
        file_name: Option<String>,
        start_position: ContextPosition,
        end_position: ContextPosition,
        contents: T,
    ) -> Self {
        let inner = InnerContext::new(file_name, start_position, end_position, contents);
        FullContext { inner }
    }

    pub fn subcontext(
        &self,
        start_position: ContextPosition,
        end_position: ContextPosition,
    ) -> Self {
        let context_ref = self.inner.self_ref.clone();
        let subcontext =
            unsafe { (&*context_ref.as_ptr()).subcontext(start_position, end_position) };

        FullContext { inner: subcontext }
    }
}

impl<'a> std::ops::Deref for FullContext<'a> {
    type Target = InnerContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::ContextPosition;
    use super::FullContext;

    #[test]
    fn test_construction() {
        let owned = "hello".to_string();
        let c = FullContext::new(
            None,
            ContextPosition::new(1, 1),
            ContextPosition::new(1, 5),
            owned,
        );
        assert!(c.is_owned());
        assert_eq!(c.get_contents(), "hello");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 5));

        let owned = "world".to_string();
        let borrowed: &str = &owned;
        let c = FullContext::new(
            None,
            ContextPosition::new(1, 1),
            ContextPosition::new(1, 5),
            borrowed,
        );
        assert!(c.is_contents_borrowed());
        assert!(!c.is_line_starts_borrowed());
        assert_eq!(c.get_contents(), "world");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 5));
    }

    #[test]
    fn subcontext() {
        let owned = "Hail Eris".to_string();
        let c = FullContext::new(
            None,
            ContextPosition::new(1, 1),
            ContextPosition::new(1, 9),
            owned,
        );
        assert!(c.is_owned());
        assert_eq!(c.get_contents(), "Hail Eris");
        assert_eq!(*c.get_file_name(), None);
        assert_eq!(*c.get_start_position(), ContextPosition::new(1, 1));
        assert_eq!(*c.get_end_position(), ContextPosition::new(1, 9));

        let sub = c.subcontext(ContextPosition::new(1, 6), ContextPosition::new(1, 9));
        assert!(sub.is_borrowed());
        assert_eq!(sub.get_contents(), "Eris");
        assert_eq!(*sub.get_file_name(), None);
        assert_eq!(*sub.get_start_position(), ContextPosition::new(1, 6));
        assert_eq!(*sub.get_end_position(), ContextPosition::new(1, 9));
    }
}
