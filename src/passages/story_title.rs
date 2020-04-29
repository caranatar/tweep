use crate::ErrorList;
use crate::Output;
use crate::Parser;
use crate::Position;
use crate::Positional;

/// Represents a passage with the `StoryTitle` name, which will be used as the
/// title for the story
///
/// # Parse Errors
/// None
///
/// # Parse Warnings
/// None
#[derive(Debug)]
pub struct StoryTitle {
    /// The title content
    pub title: String,

    /// The position of the content
    pub position: Position,
}

impl<'a> Parser<'a> for StoryTitle {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        Output::new(Ok(StoryTitle {
            title: input.join("\n"),
            position: Position::default(),
        }))
    }
}

impl Positional for StoryTitle {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}
