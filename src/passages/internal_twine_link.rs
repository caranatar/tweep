use crate::FullContext;

/// Internal structure representing a Twine link, with a target Passage name and
/// column/row offset within the Twine passage
#[derive(Debug)]
pub(crate) struct InternalTwineLink {
    /// The name of the target Passage
    pub(crate) target: String,

    /// The context of the link
    pub(crate) context: FullContext,

    /// The column at which the link occurs within the row
    pub(crate) col_offset: usize,

    /// The row at which the link occurs within the passage content
    pub(crate) row_offset: usize,

    /// The length of the link string
    #[cfg(feature = "issue-context")]
    pub(crate) context_len: usize,
}
