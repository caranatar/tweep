use crate::FullContext;

/// Internal structure representing a Twine link, with a target Passage name and
/// column/row offset within the Twine passage
#[derive(Debug)]
pub(crate) struct InternalTwineLink {
    /// The name of the target Passage
    pub(crate) target: String,

    /// The context of the link
    pub(crate) context: FullContext,
}
