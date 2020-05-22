/// An enum of the types of warnings that can be produced by `tweep`
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarningKind {
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

    /// No passage called `Start` found and no start passage set in `StoryData`
    MissingStartPassage,

    /// Start passage set in `StoryData` that cannot be found
    DeadStartPassage(String),
}

#[cfg(feature = "warning-names")]
impl WarningKind {
    /// Gets a string representation of a `WarningKind` variant's name
    ///
    /// Enabled with "warning-names" feature
    pub fn get_name(&self) -> &str {
        match self {
            WarningKind::EscapedOpenSquare => "EscapedOpenSquare",
            WarningKind::EscapedCloseSquare => "EscapedCloseSquare",
            WarningKind::EscapedOpenCurly => "EscapedOpenCurly",
            WarningKind::EscapedCloseCurly => "EscapedCloseCurly",
            WarningKind::JsonError(_) => "JsonError",
            WarningKind::DuplicateStoryData => "DuplicateStoryData",
            WarningKind::DuplicateStoryTitle => "DuplicateStoryTitle",
            WarningKind::MissingStoryData => "MissingStoryData",
            WarningKind::MissingStoryTitle => "MissingStoryTitle",
            WarningKind::UnclosedLink => "UnclosedLink",
            WarningKind::WhitespaceInLink => "WhitespaceInLink",
            WarningKind::DeadLink(_) => "DeadLink",
            WarningKind::MissingStartPassage => "MissingStartPassage",
            WarningKind::DeadStartPassage(_) => "DeadStartPassage",
        }
    }
}

impl std::fmt::Display for WarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WarningKind::EscapedOpenSquare =>
                    "Escaped [ character in passage header".to_string(),
                WarningKind::EscapedCloseSquare =>
                    "Escaped ] character in passage header".to_string(),
                WarningKind::EscapedOpenCurly =>
                    "Escaped { character in passage header".to_string(),
                WarningKind::EscapedCloseCurly =>
                    "Escaped } character in passage header".to_string(),
                WarningKind::JsonError(error_str) =>
                    format!("Error encountered while parsing JSON: {}", error_str),
                WarningKind::DuplicateStoryData => "Multiple StoryData passages found".to_string(),
                WarningKind::DuplicateStoryTitle =>
                    "Multiple StoryTitle passages found".to_string(),
                WarningKind::MissingStoryData => "No StoryData passage found".to_string(),
                WarningKind::MissingStoryTitle => "No StoryTitle passage found".to_string(),
                WarningKind::UnclosedLink => "Unclosed passage link".to_string(),
                WarningKind::WhitespaceInLink => "Whitespace in passage link".to_string(),
                WarningKind::DeadLink(target) =>
                    format!("Dead link to nonexistant passage: {}", target),
                WarningKind::MissingStartPassage =>
                    "No passage \"Start\" found and no alternate starting passage set in StoryData"
                        .to_string(),
                WarningKind::DeadStartPassage(start) =>
                    format!("Start passage set to {}, but no such passage found", start),
            }
        )
    }
}

#[cfg(all(test, feature = "warning-names"))]
mod tests {
    use super::*;

    #[cfg(feature = "warning-names")]
    #[test]
    fn test_names() {
        assert_eq!(WarningKind::EscapedOpenSquare.get_name(), "EscapedOpenSquare");
        assert_eq!(WarningKind::EscapedCloseSquare.get_name(), "EscapedCloseSquare");
        assert_eq!(WarningKind::EscapedOpenCurly.get_name(), "EscapedOpenCurly");
        assert_eq!(WarningKind::EscapedCloseCurly.get_name(), "EscapedCloseCurly");
        assert_eq!(WarningKind::JsonError("x".to_string()).get_name(), "JsonError");
        assert_eq!(WarningKind::DuplicateStoryData.get_name(), "DuplicateStoryData");
        assert_eq!(WarningKind::DuplicateStoryTitle.get_name(), "DuplicateStoryTitle");
        assert_eq!(WarningKind::MissingStoryData.get_name(), "MissingStoryData");
        assert_eq!(WarningKind::MissingStoryTitle.get_name(), "MissingStoryTitle");
        assert_eq!(WarningKind::UnclosedLink.get_name(), "UnclosedLink");
        assert_eq!(WarningKind::WhitespaceInLink.get_name(), "WhitespaceInLink");
        assert_eq!(WarningKind::DeadLink("x".to_string()).get_name(), "DeadLink");
        assert_eq!(WarningKind::MissingStartPassage.get_name(), "MissingStartPassage");
        assert_eq!(WarningKind::DeadStartPassage("x".to_string()).get_name(), "DeadStartPassage");
    }
}
