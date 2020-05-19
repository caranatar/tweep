use crate::ScriptContent;
use crate::StoryData;
use crate::StoryTitle;
use crate::StylesheetContent;
use crate::TwineContent;

/// An enum of the types of content that can be inside a [`Passage`]
///
/// [`Passage`]: struct.Passage.html
#[derive(Debug)]
pub enum PassageContent {
    /// A non-special passage that contains Twine content
    Normal(TwineContent),

    /// A passage that contains the title of the story
    StoryTitle(StoryTitle),

    /// A passage that contains the story data defined by the specification
    StoryData(Option<StoryData>),

    /// A passage that is tagged with `script` and contains a script
    Script(ScriptContent),

    /// A passage that is tagged with `stylesheet` and contains CSS
    Stylesheet(StylesheetContent),
}

impl std::convert::From<TwineContent> for PassageContent {
    fn from(p: TwineContent) -> PassageContent {
        PassageContent::Normal(p)
    }
}

impl std::convert::From<StoryTitle> for PassageContent {
    fn from(t: StoryTitle) -> PassageContent {
        PassageContent::StoryTitle(t)
    }
}

impl std::convert::From<Option<StoryData>> for PassageContent {
    fn from(d: Option<StoryData>) -> PassageContent {
        PassageContent::StoryData(d)
    }
}

impl std::convert::From<StoryData> for PassageContent {
    fn from(d: StoryData) -> PassageContent {
        PassageContent::StoryData(Some(d))
    }
}

impl std::convert::From<ScriptContent> for PassageContent {
    fn from(s: ScriptContent) -> PassageContent {
        PassageContent::Script(s)
    }
}

impl std::convert::From<StylesheetContent> for PassageContent {
    fn from(s: StylesheetContent) -> PassageContent {
        PassageContent::Stylesheet(s)
    }
}
