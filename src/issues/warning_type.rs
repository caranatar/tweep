/// An enum of the types of warnings that can be produced by `tweep`
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

    /// No passage called `Start` found and no start passage set in `StoryData`
    MissingStartPassage,

    /// Start passage set in `StoryData` that cannot be found
    DeadStartPassage(String),
}

#[cfg(feature = "warning-names")]
impl WarningType {
    /// Gets a string representation of a `WarningKind` variant's name
    ///
    /// Enabled with "warning-names" feature
    pub fn get_name(&self) -> &str {
        match self {
            WarningType::EscapedOpenSquare => "EscapedOpenSquare",
            WarningType::EscapedCloseSquare => "EscapedCloseSquare",
            WarningType::EscapedOpenCurly => "EscapedOpenCurly",
            WarningType::EscapedCloseCurly => "EscapedCloseCurly",
            WarningType::JsonError(_) => "JsonError",
            WarningType::DuplicateStoryData => "DuplicateStoryData",
            WarningType::DuplicateStoryTitle => "DuplicateStoryTitle",
            WarningType::MissingStoryData => "MissingStoryData",
            WarningType::MissingStoryTitle => "MissingStoryTitle",
            WarningType::UnclosedLink => "UnclosedLink",
            WarningType::WhitespaceInLink => "WhitespaceInLink",
            WarningType::DeadLink(_) => "DeadLink",
            WarningType::MissingStartPassage => "MissingStartPassage",
            WarningType::DeadStartPassage(_) => "DeadStartPassage",
        }
    }
}

impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WarningType::EscapedOpenSquare =>
                    "Escaped [ character in passage header".to_string(),
                WarningType::EscapedCloseSquare =>
                    "Escaped ] character in passage header".to_string(),
                WarningType::EscapedOpenCurly =>
                    "Escaped { character in passage header".to_string(),
                WarningType::EscapedCloseCurly =>
                    "Escaped } character in passage header".to_string(),
                WarningType::JsonError(error_str) =>
                    format!("Error encountered while parsing JSON: {}", error_str),
                WarningType::DuplicateStoryData => "Multiple StoryData passages found".to_string(),
                WarningType::DuplicateStoryTitle =>
                    "Multiple StoryTitle passages found".to_string(),
                WarningType::MissingStoryData => "No StoryData passage found".to_string(),
                WarningType::MissingStoryTitle => "No StoryTitle passage found".to_string(),
                WarningType::UnclosedLink => "Unclosed passage link".to_string(),
                WarningType::WhitespaceInLink => "Whitespace in passage link".to_string(),
                WarningType::DeadLink(target) =>
                    format!("Dead link to nonexistant passage: {}", target),
                WarningType::MissingStartPassage =>
                    "No passage \"Start\" found and no alternate starting passage set in StoryData"
                        .to_string(),
                WarningType::DeadStartPassage(start) =>
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
        assert_eq!(WarningType::EscapedOpenSquare.get_name(), "EscapedOpenSquare");
        assert_eq!(WarningType::EscapedCloseSquare.get_name(), "EscapedCloseSquare");
        assert_eq!(WarningType::EscapedOpenCurly.get_name(), "EscapedOpenCurly");
        assert_eq!(WarningType::EscapedCloseCurly.get_name(), "EscapedCloseCurly");
        assert_eq!(WarningType::JsonError("x".to_string()).get_name(), "JsonError");
        assert_eq!(WarningType::DuplicateStoryData.get_name(), "DuplicateStoryData");
        assert_eq!(WarningType::DuplicateStoryTitle.get_name(), "DuplicateStoryTitle");
        assert_eq!(WarningType::MissingStoryData.get_name(), "MissingStoryData");
        assert_eq!(WarningType::MissingStoryTitle.get_name(), "MissingStoryTitle");
        assert_eq!(WarningType::UnclosedLink.get_name(), "UnclosedLink");
        assert_eq!(WarningType::WhitespaceInLink.get_name(), "WhitespaceInLink");
        assert_eq!(WarningType::DeadLink("x".to_string()).get_name(), "DeadLink");
        assert_eq!(WarningType::MissingStartPassage.get_name(), "MissingStartPassage");
        assert_eq!(WarningType::DeadStartPassage("x".to_string()).get_name(), "DeadStartPassage");
    }
}
