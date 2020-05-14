use crate::Position;
use crate::Positional;

/// A link to a twee passage contained within a twee passage
#[derive(Debug, Eq, PartialEq)]
pub struct TwineLink {
    /// The name of the passage this link points to
    pub target: String,

    /// The position of the link
    pub position: Position,

    /// The length of the link text
    #[cfg(feature = "issue-context")]
    pub context_len: usize,
}

impl TwineLink {
    /// Creates a new link with a default [`Position`]
    ///
    /// [`Position`]: enum.Position.html
    #[cfg(not(feature = "issue-context"))]
    pub fn new(target: String) -> Self {
        TwineLink {
            target,
            position: Position::default(),
        }
    }

    #[cfg(feature = "issue-context")]
    pub fn new(target: String, context_len: usize) -> Self {
        TwineLink {
            target,
            context_len,
            position: Position::default(),
        }
    }
}

impl Positional for TwineLink {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}
