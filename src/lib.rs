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

mod passage;
pub(crate) use passage::InternalTwineLink;
pub use passage::Passage;
pub use passage::PassageContent;
pub use passage::PassageHeader;
pub use passage::ScriptContent;
pub use passage::StoryTitle;
pub use passage::StylesheetContent;
pub use passage::StoryData;
pub use passage::TwineContent;
pub use passage::TwineLink;

mod position;
pub use position::Position;
pub use position::Positional;

mod story;
pub use story::Story;
pub use story::StoryPassages;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
