use crate::context::{Position, FullContext};

/// A Context that holds only an optional file name and 1-indexed start position
///
/// Intended to be constructed only from a [`FullContext`] as a way of
/// discarding additional, unwanted information.
///
/// [`FullContext`]: struct.FullContext.html
pub struct PartialContext {
    file_name: Option<String>,
    start_position: Position,
}

impl PartialContext {
    /// Returns a reference to the optional file name
    pub fn get_file_name(&self) -> &Option<String> {
        &self.file_name
    }

    /// Returns a reference to the 1-indexed start position
    pub fn get_start_position(&self) -> &Position {
        &self.start_position
    }
}

impl std::convert::From<FullContext> for PartialContext {
    fn from(full: FullContext) -> PartialContext {
        PartialContext {
            file_name: full.get_file_name().clone(),
            start_position: *full.get_start_position(),
        }
    }
}

impl std::fmt::Display for PartialContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.file_name, self.start_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let contents = "hail eris".to_string();
        let c = FullContext::from(None, contents);
        let partial: PartialContext = c.into();
        assert_eq!(*partial.get_file_name(), None);
        assert_eq!(*partial.get_start_position(), Position::new(1, 1));
    }

    #[test]
    fn from_subcontext() {
        let name = "name.ext".to_string();
        let contents = "hail eris".to_string();
        let c = FullContext::from(Some(name), contents);
        let sub = c.subcontext(Position::new(1, 6)..=Position::new(2, 3));
        let partial: PartialContext = sub.into();
        assert_eq!(*partial.get_file_name(), Some("name.ext".to_string()));
        assert_eq!(*partial.get_start_position(), Position::new(1, 6));
    }
}
