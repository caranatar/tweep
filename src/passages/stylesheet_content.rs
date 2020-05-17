use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Position;
use crate::Positional;

/// The contents of a [`Passage`] tagged with `stylesheet`, containing CSS data.
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

impl StylesheetContent {
    /// Parses a `StylesheetContent` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        Output::new(Ok(StylesheetContent {
            content: context.get_contents().to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input = r#"foo
bar
baz"#
            .to_string();
        let out = StylesheetContent::parse(FullContext::from(None, input.clone()));
        assert!(!out.has_warnings());
        let (res, _) = out.take();
        assert!(res.is_ok());
        let content = res.ok().unwrap();
        assert_eq!(content.content, input);
    }
}
