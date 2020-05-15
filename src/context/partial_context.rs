use crate::context::{ContextPosition, FullContext};

/// A Context that holds only an optional file name and 1-indexed start position
///
/// Intended to be constructed only from a [`FullContext`] as a way of
/// discarding additional, unwanted information.
///
/// [`FullContext`]: struct.FullContext.html
pub struct PartialContext {
    file_name: Option<String>,
    start_position: ContextPosition,
}

impl PartialContext {
    /// Returns a reference to the optional file name
    pub fn get_file_name(&self) -> &Option<String> {
        &self.file_name
    }

    /// Returns a reference to the 1-indexed start position
    pub fn get_start_position(&self) -> &ContextPosition {
        &self.start_position
    }
}

impl std::convert::From<FullContext<'_>> for PartialContext {
    fn from(full: FullContext<'_>) -> PartialContext {
        PartialContext {
            file_name: full.get_file_name().clone(),
            start_position: full.get_start_position().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let contents = "hail eris".to_string();
        let c = FullContext::new(
            None,
            ContextPosition::new(1, 1),
            ContextPosition::new(5, 23),
            contents,
        );
        let partial: PartialContext = c.into();
        assert_eq!(*partial.get_file_name(), None);
        assert_eq!(*partial.get_start_position(), ContextPosition::new(1, 1));
    }

    #[test]
    fn from_subcontext() {
        let name = "name.ext".to_string();
        let contents = "hail eris".to_string();
        let c = FullContext::new(
            Some(name),
            ContextPosition::new(1, 1),
            ContextPosition::new(5, 23),
            contents,
        );
        let sub = c.subcontext(ContextPosition::new(1, 6), ContextPosition::new(2, 3));
        let partial: PartialContext = sub.into();
        assert_eq!(*partial.get_file_name(), Some("name.ext".to_string()));
        assert_eq!(*partial.get_start_position(), ContextPosition::new(1, 6));
    }
}
