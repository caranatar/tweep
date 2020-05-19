use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Position;

/// The content of a [`Passage`] tagged with `script`, containing script data.
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
pub struct ScriptContent {
    /// The full content of the passage
    pub content: String,

    /// The position of the content
    pub position: Position,
}

impl ScriptContent {
    /// Parses a `ScriptContent` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        Output::new(Ok(ScriptContent {
            content: context.get_contents().to_string(),
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
        let out = ScriptContent::parse(FullContext::from(None, input.clone()));
        assert!(!out.has_warnings());
        let (res, _) = out.take();
        assert!(res.is_ok());
        let content = res.ok().unwrap();
        assert_eq!(content.content, input);
    }
}
