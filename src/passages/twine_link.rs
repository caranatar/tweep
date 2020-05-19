use crate::Position;
use crate::FullContext;

/// A link to a twee passage contained within a twee passage
#[derive(Debug, Eq, PartialEq)]
pub struct TwineLink {
    /// The name of the passage this link points to
    pub target: String,

    /// The context of the link
    pub context: FullContext,

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
    pub fn new(target: String, context: FullContext) -> Self {
        TwineLink {
            target,
            position: Position::default(),
            context,
        }
    }

    #[cfg(feature = "issue-context")]
    pub fn new(target: String, context:FullContext, context_len: usize) -> Self {
        TwineLink {
            target,
            context_len,
            position: Position::default(),
            context,
        }
    }
}
