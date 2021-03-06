#[cfg(feature = "full-context")]
use crate::CodeMap;
#[cfg(feature = "full-context")]
use crate::ContextErrorList;
#[cfg(not(feature = "full-context"))]
use crate::ErrorList;
use crate::Output;
use crate::PassageContent;
use crate::StoryData;
use crate::StoryPassages;
use crate::TwinePassage;
use std::collections::HashMap;
use std::path::Path;

/// A parsed Twee story
///
/// This is the primary interface for tweep. The provided utility functions
/// allow a Twee 3 story to be parsed from a `String`, a directory or file
/// `Path`, or a slice of string slices, representing the lines of input. The
/// output is an `Output<Result<Story, ErrorList>>` which is either the parsed
/// `Story` or an [`ErrorList`] if the parse failed, along with a list of any
/// [`Warning`]s generated during parsing. The fields in this struct provide
/// access to all necessary components of the parsed story.
///
/// # Parse Errors
/// * [`BadInputPath`] - The given `Path` cannot be used to parse a story
/// See [`Passage`] for other errors that can occur during parsing
///
/// # Parse Warnings
/// * [`DuplicateStoryTitle`] - More than one `StoryTitle` passage found
/// * [`DuplicateStoryData`] - More than one `StoryData` passage found
/// * [`MissingStoryTitle`] - No `StoryTitle` passage found
/// * [`MissingStoryData`] - No `StoryData` passage found
/// * [`DeadLink`] - Found a link to a non-existent passage
/// * [`MissingStartPassage`] - No `Start` passage found and no alternate
///   passage set in `StoryData`
/// * [`DeadStartPassage`] - Alternate start passage set in `StoryData`, but
///   no such passage found in parsing
/// See [`Passage`] for other warnings that can occur during parsing
///
///
/// # Examples
/// ```
/// use tweep::Story;
/// let input = r#":: StoryTitle
///RustDoc Sample Story
///
///:: StoryData
///{
///  "ifid": "D674C58C-DEFA-4F70-B7A2-27742230C0FC",
///  "format": "SugarCube",
///  "format-version": "2.28.2",
///  "start": "My Starting Passage",
///  "tag-colors": {
///    "tag1": "green",
///    "tag2": "red",
///    "tag3": "blue"
///  },
///  "zoom": 0.25
///}
///
///:: My Starting Passage [ tag1 tag2 ]
///This is the starting passage, specified by the start attribute of StoryData.
///Alternately, we could remove that attribute and rename the passage to Start.
///
///It has tags and links to:
///  [[Another passage]]
///  [[Here too!|Another passage]]
///  [[A third passage<-And a different passage]]
///
///:: Another passage {"position":"600,400","size":"100,200"}
///This passage has some metadata attached to it
///
///:: A third passage [tag3] { "position": "400,600" }
///This passage has both tags and metadata. The size attribute of the metadata
///isn't overridden, so it will be set to the default value.
///"#.to_string();
///
///// Parse the input into an Output<Result<Story, ErrorList>>
///let out = Story::from_string(input);
///assert!(!out.has_warnings());
///
///// Move the Result out of the Output
///let (res, _) = out.take();
///assert!(res.is_ok());
///
///// Get the Story object
///let story = res.ok().unwrap();
///
///// StoryTitle and StoryData contents are parsed into special fields
///assert_eq!(story.title.unwrap(), "RustDoc Sample Story");
///assert_eq!(story.data.unwrap().ifid, "D674C58C-DEFA-4F70-B7A2-27742230C0FC");
///
///// Other passages are parsed into a map, keyed by the passage name
///assert_eq!(story.passages["My Starting Passage"].tags(), &vec!["tag1", "tag2"]);
///let metadata = story.passages["A third passage"].metadata();
///assert_eq!(metadata["size"], "100,100");
///assert_eq!(metadata["position"], "400,600");
/// ```
///
/// [`DuplicateStoryTitle`]: enum.WarningKind.html#variant.DuplicateStoryTitle
/// [`DuplicateStoryData`]: enum.WarningKind.html#variant.DuplicateStoryData
/// [`MissingStoryTitle`]: enum.WarningKind.html#variant.MissingStoryTitle
/// [`MissingStoryData`]: enum.WarningKind.html#variant.MissingStoryData
/// [`DeadLink`]: enum.WarningKind.html#variant.DeadLink
/// [`MissingStartPassage`]: enum.WarningKind.html#variant.MissingStartPassage
/// [`DeadStartPassage`]: enum.WarningKind.html#variant.DeadStartPassage
/// [`BadInputPath`]: enum.ErrorKind.html#variant.BadInputPath
/// [`Passage`]: struct.Passage.html
#[derive(Default)]
pub struct Story {
    /// The story title
    pub title: Option<String>,

    /// The story data as defined by the specification
    pub data: Option<StoryData>,

    /// Map from passage name to `TwinePassage` for any non-special passages
    pub passages: HashMap<String, TwinePassage>,

    /// A list of the contents of any passages tagged with `script`
    pub scripts: Vec<String>,

    /// A list of the contents of any passages tagged with `stylesheet`
    pub stylesheets: Vec<String>,

    /// StoryMap for this story
    #[cfg(feature = "full-context")]
    pub code_map: CodeMap,
}

#[cfg(not(feature = "full-context"))]
type ParseOutput = Output<Result<Story, ErrorList>>;
#[cfg(feature = "full-context")]
type ParseOutput = Output<Result<Story, ContextErrorList>>;

impl Story {
    /// Parses an input `String` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_string(input: String) -> ParseOutput {
        StoryPassages::from_string(input).into_result()
    }

    /// Parses a `Story` from the given [`Path`]. If the given path is a file,
    /// parses that file and returns the `Story`. If it is a directory, it looks
    /// for any files with `.tw` or `.twee` extensions and parses them. Returns
    /// the parsed output or a list of errors, along with a list of any
    /// [`Warning`]s
    ///
    /// [`Path`]: std::path::Path
    /// [`Warning`]: struct.Warning.html
    pub fn from_path<P: AsRef<Path>>(input: P) -> ParseOutput {
        StoryPassages::from_path(input).into_result()
    }

    /// Parses a `Story` from the given [`Path`]s. See `from_path` for
    /// additional information on how directories are handled.
    ///
    /// [`Path`]: std::path::Path
    pub fn from_paths<P: AsRef<Path>>(input: &[P]) -> ParseOutput {
        StoryPassages::from_paths(input).into_result()
    }

    /// If a start passage is configured in the StoryData, return the name of
    /// that passage. If no start passage is configured, check for the presence
    /// of a passage called "Start". If that passage exists, return that name,
    /// otherwise return None
    pub fn get_start_passage_name(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|d| d.start.as_deref())
            .or_else(|| {
                if self.passages.contains_key("Start") {
                    Some("Start")
                } else {
                    None
                }
            })
    }
}

impl std::convert::From<StoryPassages> for Story {
    fn from(mut s: StoryPassages) -> Story {
        let title = match s.title {
            Some(c) => match c.content {
                PassageContent::StoryTitle(t) => Some(t.title),
                _ => panic!("Expected title to be StoryTitle"),
            },
            None => None,
        };

        let data = match s.data {
            Some(c) => match c.content {
                PassageContent::StoryData(d) => d,
                _ => panic!("Expected data to be StoryData"),
            },
            None => None,
        };

        let scripts = s
            .scripts
            .into_iter()
            .map(|p| match p.content {
                PassageContent::Script(script) => script.content,
                _ => panic!("Expected script to be Script"),
            })
            .collect();

        let stylesheets = s
            .stylesheets
            .into_iter()
            .map(|p| match p.content {
                PassageContent::Stylesheet(stylesheet) => stylesheet.content,
                _ => panic!("Expected stylesheet to be Stylesheet"),
            })
            .collect();

        let passages: HashMap<String, TwinePassage> =
            s.passages.drain().map(|(k, v)| (k, v.into())).collect();

        #[cfg(feature = "full-context")]
        let code_map = s.code_map;

        Story {
            title,
            data,
            passages,
            scripts,
            stylesheets,
            #[cfg(feature = "full-context")]
            code_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::Warning;
    use crate::WarningKind;
    use tempfile::tempdir;

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


"#
        .to_string();
        use crate::FullContext;
        use crate::Position;
        let out = Story::from_string(input.clone());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let context = FullContext::from(None, input);
        assert_eq!(warnings[0], {
            let warning = Warning::new(
                WarningKind::EscapedOpenSquare,
                Some(context.subcontext(Position::rel(7, 5)..=Position::rel(7, 6))),
            );
            warning
        });
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


"#
        .to_string();
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
        assert_eq!(
            warnings[0],
            Warning::new::<Context>(WarningKind::MissingStoryData, None)
        );

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


"#
        .to_string();
        let out = Story::from_string(input);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.get_start_passage_name(), None);
        assert_eq!(story.title.is_some(), true);
        let title = story.title.unwrap();
        assert_eq!(title, "Test Story");
    }

    #[test]
    fn dir_input() -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        let input_one = r#":: Start
At the start, link to [[A passage]]

:: A passage
This passage links to [[Another passage]]

:: StoryTitle
Test Story

:: Wa\{rning title one
blah blah
"#
        .to_string();

        let input_two = r#":: Another passage
Links back to [[Start]]

:: StoryData
{
"ifid": "ABC"
}

:: Warning titl\]e two
blah blah
"#
        .to_string();

        use std::io::Write;
        let dir = tempdir()?;
        let file_path_one = dir.path().join("test.twee");
        let mut file_one = File::create(file_path_one.clone())?;
        write!(file_one, "{}", input_one.clone())?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        write!(file_two, "{}", input_two.clone())?;

        let out = Story::from_path(dir.path());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 2);
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title, Some("Test Story".to_string()));
        assert_eq!(story.get_start_passage_name(), Some("Start"));

        use crate::FullContext;
        use crate::Position;
        let context = FullContext::from(Some("test.twee".to_string()), input_one);
        assert!(warnings.contains(&{
            let warning = Warning::new(
                WarningKind::EscapedOpenCurly,
                Some(context.subcontext(Position::rel(10, 6)..=Position::rel(10, 7))),
            );
            warning
        }));

        let context = FullContext::from(Some("test2.tw".to_string()), input_two);
        assert!(warnings.contains(&{
            let warning = Warning::new(
                WarningKind::EscapedCloseSquare,
                Some(context.subcontext(Position::rel(9, 16)..=Position::rel(9, 17))),
            );
            warning
        }));

        Ok(())
    }
}
