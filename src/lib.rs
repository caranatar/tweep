//! Parser for the Twee 3 interactive fiction format
//!
//! Work in progress

#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]
mod issues;
pub use issues::Error;
pub use issues::ErrorList;
pub use issues::ErrorType;
pub use issues::Warning;
pub use issues::WarningType;

mod output;
pub use output::Output;

mod parser;
pub use parser::Parser;

mod passages;
pub(crate) use passages::InternalTwineLink;
pub use passages::Passage;
pub use passages::PassageContent;
pub use passages::PassageHeader;
pub use passages::ScriptContent;
pub use passages::StoryTitle;
pub use passages::StylesheetContent;
pub use passages::StoryData;
pub use passages::TwineContent;
pub use passages::TwineLink;

mod positions;
pub use positions::Position;
pub use positions::Positional;

mod stories;
pub use stories::Story;
pub use stories::StoryPassages;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
