/// Represents the types of warnings that can be produced by `tweep`
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarningType {
    /// `\[` in a passage title
    EscapedOpenSquare,

    /// `\]` in a passage title
    EscapedCloseSquare,

    /// `\{` in a passage title
    EscapedOpenCurly,

    /// `\}` in a passage title
    EscapedCloseCurly,

    /// Error encountered while parsing JSON. Contains the text of the error
    JsonError(String),

    /// `StoryTitle` passage encountered after parsing a `StoryTitle` passage
    DuplicateStoryTitle,

    /// `StoryData` passage encountered after parsing a `StoryData` passage
    DuplicateStoryData,

    /// No `StoryTitle` passage parsed while parsing a [`Story`](struct.Story.html)
    MissingStoryTitle,

    /// No `StoryData` passage parsed while parsing a [`Story`](struct.Story.html)
    MissingStoryData,

    /// Encountered a link in a [`TwineContent`](struct.TwineContent.html) passage that was unterminated
    UnclosedLink,

    /// Encountered errant whitespace in a Twine link (e.g., `[[Text | Link]]`)
    WhitespaceInLink,

    /// Encountered a link to a passage name that does not match any parsed
    /// passage. Contains the passage name content of the dead link.
    DeadLink(String),
}

impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            WarningType::EscapedOpenSquare => format!("Escaped [ character in passage header"),
            WarningType::EscapedCloseSquare => format!("Escaped ] character in passage header"),
            WarningType::EscapedOpenCurly => format!("Escaped {{ character in passage header"),
            WarningType::EscapedCloseCurly => format!("Escaped }} character in passage header"),
            WarningType::JsonError(error_str) => format!("Error encountered while parsing JSON: {}", error_str),
            WarningType::DuplicateStoryData => format!("Multiple StoryData passages found"),
            WarningType::DuplicateStoryTitle => format!("Multiple StoryTitle passages found"),
            WarningType::MissingStoryData => format!("No StoryData passage found"),
            WarningType::MissingStoryTitle => format!("No StoryTitle passage found"),
            WarningType::UnclosedLink => format!("Unclosed passage link"),
            WarningType::WhitespaceInLink => format!("Whitespace in passage link"),
            WarningType::DeadLink(target) => format!("Dead link to nonexistant passage: {}", target),
        })
    }
}
