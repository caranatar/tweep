use crate::PassageContent;
use crate::Passage;
use crate::PassageHeader;
use crate::TwineContent;

/// A special Twine passage to be used in [`Story`]s without the need to go
/// through an enum to get the passage content
///
/// [`Story`]: struct.Story.html
pub struct TwinePassage {
    /// The header
    pub header: PassageHeader,

    /// The content
    pub content: TwineContent,
}

impl std::convert::From<Passage> for TwinePassage {
    fn from(passage: Passage) -> Self {
        let header = passage.header;
        let content = if let PassageContent::Normal(content) = passage.content {
            content
        } else {
            panic!("");
        };
        TwinePassage { header, content }
    }
}
