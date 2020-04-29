use crate::ErrorList;
use crate::Output;
use crate::Parser;
use crate::Position;
use crate::Positional;

/// Represents the content of a [`Passage`] tagged with `stylesheet`, containing
/// CSS data.
///
/// No validation is done when parsing this content.
///
/// # Parse Errors
/// None
///
/// # Parse Warnings
/// None
///
/// [`Passage`]: struct.Passage.html
#[derive(Debug)]
pub struct StylesheetContent {
    /// The stylesheet content
    pub content: String,

    /// The position of the content
    pub position: Position,
}

impl<'a> Parser<'a> for StylesheetContent {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        Output::new(Ok(StylesheetContent {
            content: input.join("\n"),
            position: Position::default(),
        }))
    }
}

impl Positional for StylesheetContent {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}
