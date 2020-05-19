use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Position;

/// The content of a special passage with the `StoryTitle` name, which will be
/// used as the title for a parsed story
///
/// # Parse Errors
/// None
///
/// # Parse Warnings
/// None
///
/// # Examples
/// ```
/// use tweep::{FullContext, StoryTitle};
/// let context = FullContext::from(None, "Example Story".to_string());
/// let out = StoryTitle::parse(context);
/// assert_eq!(out.get_output().as_ref().ok().unwrap().title, "Example Story");
/// ```
#[derive(Debug)]
pub struct StoryTitle {
    /// The title content
    pub title: String,

    /// The position of the content
    pub position: Position,
}

impl StoryTitle {
    /// Parses a `StoryTitle` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        Output::new(Ok(StoryTitle {
            title: context.get_contents().to_string(),
            position: Position::default(),
        }))
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
        let out = StoryTitle::parse(FullContext::from(None, input.clone()));
        assert!(!out.has_warnings());
        let (res, _) = out.take();
        assert!(res.is_ok());
        let content = res.ok().unwrap();
        assert_eq!(content.title, input);
    }
}
