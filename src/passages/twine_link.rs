use crate::FullContext;

/// A link to a twee passage contained within a twee passage
#[derive(Debug, Eq, PartialEq)]
pub struct TwineLink {
    /// The name of the passage this link points to
    pub target: String,

    /// The context of the link
    pub context: FullContext,
}

impl TwineLink {
    /// Creates a new link with a default [`Position`]
    ///
    /// [`Position`]: enum.Position.html
    pub fn new(target: String, context: FullContext) -> Self {
        TwineLink {
            target,
            context,
        }
    }
}
