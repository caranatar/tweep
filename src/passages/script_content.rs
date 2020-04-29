use crate::ErrorList;
use crate::Output;
use crate::Parser;
use crate::Position;
use crate::Positional;

/// Represents the content of a [`Passage`] tagged with `script`, containing
/// script data.
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

impl<'a> Parser<'a> for ScriptContent {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        Output::new(Ok(ScriptContent {
            content: input.join("\n"),
            position: Position::default(),
        }))
    }
}

impl Positional for ScriptContent {
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
baz"#.to_string();
        let v:Vec<&str> = input.split('\n').collect();
        let out = ScriptContent::parse(&v);
        assert!(!out.has_warnings());
        let (res, _) = out.take();
        assert!(res.is_ok());
        let content = res.ok().unwrap();
        assert_eq!(content.content, input);
    }
}
