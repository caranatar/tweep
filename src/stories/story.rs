use crate::ErrorList;
use crate::Passage;
use crate::PassageContent;
use crate::Output;
use crate::StoryPassages;
use crate::StoryData;
use std::path::Path;
use std::collections::HashMap;

/// Represents a parsed Twee story
#[derive(Default)]
pub struct Story {
    /// The story title
    pub title: Option<String>,

    /// The story data as defined by the specification
    pub data: Option<StoryData>,

    /// Map from passage name to `Passage` for any non-special passages
    pub passages: HashMap<String, Passage>,

    /// A list of the contents of any passages tagged with `script`
    pub scripts: Vec<String>,

    /// A list of the contents of any passages tagged with `stylesheet`
    pub stylesheets: Vec<String>,
}

impl Story {
    /// Parses an input `String` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_string(input: String) -> Output<Result<Self, ErrorList>> {
        StoryPassages::from_string(input).into_result()
    }

    /// Parses an input `&[&str]` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_slice(input: &[&str]) -> Output<Result<Self, ErrorList>> {
        StoryPassages::from_slice(input).into_result()
    }

    /// Parses a `Story` from the given [`Path`]. If the given path is a file,
    /// parses that file and returns the `Story`. If it is a directory, it looks
    /// for any files with `.tw` or `.twee` extensions and parses them. Returns
    /// the parsed output or a list of errors, along with a list of any
    /// [`Warning`]s
    ///
    /// [`Path`]: std::path::Path
    /// [`Warning`]: struct.Warning.html
    pub fn from_path<P: AsRef<Path>>(input: P) -> Output<Result<Self, ErrorList>> {
        StoryPassages::from_path(input).into_result()
    }
}

impl std::convert::From<StoryPassages> for Story {
    fn from(s: StoryPassages) -> Story {
        let title = match s.title {
            Some(c) => match c.content {
                PassageContent::StoryTitle(t) => Some(t.title),
                _ => panic!("Expected title to be StoryTitle"),
            },
            None => None,
        };

        let data = match s.data {
            Some(c) => match c.content {
                PassageContent::StoryData(d, _) => d,
                _ => panic!("Expected data to be StoryData"),
            },
            None => None,
        };

        let scripts = s.scripts.into_iter().map(|p| {
            match p.content {
                PassageContent::Script(script) => script.content,
                _ => panic!("Expected script to be Script"),
            }
        }).collect();

        let stylesheets = s.stylesheets.into_iter().map(|p| {
            match p.content {
                PassageContent::Stylesheet(stylesheet) => stylesheet.content,
                _ => panic!("Expected stylesheet to be Stylesheet"),
            }
        }).collect();

        let passages = s.passages;

        Story { title, data, passages, scripts, stylesheets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::Warning;
    use crate::WarningType;

    #[test]
    fn warning_offsets() {
        let input = r#":: A passage
This
That
The Other


:: A\[nother passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        use crate::Positional;
        let out = Story::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        assert_eq!(warnings[0], Warning::new(WarningType::EscapedOpenSquare).with_row(6).with_column(4));
    }

    #[test]
    fn file_input() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#":: A passage
This
That
The Other


:: Another passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        use std::fs::File;
        use std::io::Write;
        let dir = tempdir()?;
        let file_path = dir.path().join("test.twee");
        let mut file = File::create(file_path.clone())?;
        writeln!(file, "{}", input)?;

        let out = Story::from_path(file_path);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title = story.title.unwrap();
        assert_eq!(title, "Test Story");
        assert_eq!(warnings[0], Warning::new(WarningType::MissingStoryData));

        Ok(())
    }

    #[test]
    fn a_test() {
        let input = r#":: A passage
This
That
The Other


:: Another passage
Foo
Bar
Baz


:: StoryTitle
Test Story


"#.to_string();
        let out = Story::from_string(input);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title = story.title.unwrap();
        assert_eq!(title, "Test Story");
    }
}
