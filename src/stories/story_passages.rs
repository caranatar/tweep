use crate::Error;
use crate::ErrorList;
use crate::Output;
use crate::Parser;
use crate::Passage;
use crate::PassageContent;
use crate::Positional;
use crate::Warning;
use crate::WarningType;
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// A parsed Twee story, that stores the full [`Passage`] object of each field
///
/// For more information, see the [`Story`] struct.
///
/// [`Passage`]: struct.Passage.html
/// [`Story`]: struct.Story.html
#[derive(Default)]
pub struct StoryPassages {
    /// `StoryTitle` passage
    pub title: Option<Passage>,

    /// `StoryData` passage
    pub data: Option<Passage>,

    /// Map from passage name to `Passage` for any non-special passages
    pub passages: HashMap<String, Passage>,

    /// List of passages tagged with `script`
    pub scripts: Vec<Passage>,

    /// List of passages tagged with `stylesheet`
    pub stylesheets: Vec<Passage>,
}

impl StoryPassages {
    /// Parses an input `String` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_string(input: String) -> Output<Result<Self, ErrorList>> {
        let slice: Vec<&str> = input.split('\n').collect();
        StoryPassages::parse(&slice)
    }

    /// Parses an input `&[&str]` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_slice(input: &[&str]) -> Output<Result<Self, ErrorList>> {
        StoryPassages::parse(input)
    }

    /// Parses a `StoryPassages` from the given [`Path`]. If the given path is
    /// a file, parses that file and returns the `StoryPassages`. If it is a
    /// directory, it looks for any files with `.tw` or `.twee` extensions and
    /// parses them. Returns the parsed output or a list of errors, along with a
    /// list of any [`Warning`]s
    ///
    /// [`Path`]: std::path::Path
    /// [`Warning`]: struct.Warning.html
    pub fn from_path<P: AsRef<Path>>(input: P) -> Output<Result<Self, ErrorList>> {
        let out = StoryPassages::from_path_internal(input);
        let (mut res, mut warnings) = out.take();
        if res.is_ok() {
            let story = res.ok().unwrap();
            let mut story_warnings = story.check();
            warnings.append(&mut story_warnings);
            res = Ok(story);
        }
        Output::new(res).with_warnings(warnings)
    }

    /// Parses a `StoryPassages` from the given [`Path`]s. See `from_path` for
    /// additional information on how directories are handled.
    ///
    /// [`Path`]: std::path::Path
    pub fn from_paths<P: AsRef<Path>>(input: &[P]) -> Output<Result<Self, ErrorList>> {
        let mut story = StoryPassages::default();
        let mut warnings = Vec::new();
        for path in input {
            let out = StoryPassages::from_path_internal(path);
            let (res, mut sub_warnings) = out.take();
            if res.is_err() {
                return Output::new(res).with_warnings(warnings);
            }
            let sub_story = res.ok().unwrap();
            let mut merge_warnings = story.merge_from(sub_story);
            warnings.append(&mut sub_warnings);
            warnings.append(&mut merge_warnings);
        }
        
        let mut story_warnings = story.check();
        warnings.append(&mut story_warnings);
        
        Output::new(Ok(story)).with_warnings(warnings)
    }

    /// Does the heavy lifting for `from_path`. If given a file, reads its
    /// contents into a `String` and uses `from_string` to parse it. If given a
    /// directory, finds the twee files, recurses with each file, then assembles
    /// the outputs into a single output
    fn from_path_internal<P: AsRef<Path>>(input: P) -> Output<Result<Self, ErrorList>> {
        let path: &Path = input.as_ref();
        let path_string: String = path.to_string_lossy().to_owned().to_string();
        if path.is_file() {
            let file_name: String = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_owned()
                .to_string();
            let file = File::open(path);
            if file.is_err() {
                let err_string = format!("{}", file.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(
                    path_string,
                    err_string,
                ))
                .into()));
            }
            let mut file = file.ok().unwrap();
            let mut contents = String::new();
            let res = file.read_to_string(&mut contents);
            if res.is_err() {
                let err_string = format!("{}", res.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(
                    path_string,
                    err_string,
                ))
                .into()));
            }
            StoryPassages::from_string(contents).with_file(file_name)
        } else if path.is_dir() {
            let dir = std::fs::read_dir(path);
            if dir.is_err() {
                let err_string = format!("{}", dir.err().unwrap());
                return Output::new(Err(Error::new(crate::ErrorType::BadInputPath(
                    path_string,
                    err_string,
                ))
                .into()));
            }
            let dir = dir.ok().unwrap();
            let mut story = StoryPassages::default();
            let mut warnings = Vec::new();
            for entry in dir {
                if entry.is_err() {
                    continue;
                }
                let file_path = entry.ok().unwrap().path();
                let extension = file_path.extension();
                if extension.is_none() {
                    continue;
                }
                let extension = extension.unwrap().to_string_lossy();
                if !((extension == "tw" || extension == "twee") && file_path.is_file()) {
                    continue;
                }
                let out = StoryPassages::from_path_internal(file_path);
                let (res, mut sub_warnings) = out.take();
                if res.is_err() {
                    return Output::new(res).with_warnings(warnings);
                }
                let sub_story = res.ok().unwrap();
                let mut merge_warnings = story.merge_from(sub_story);
                warnings.append(&mut sub_warnings);
                warnings.append(&mut merge_warnings);
            }
            Output::new(Ok(story)).with_warnings(warnings)
        } else {
            let err_string = "Path is not a file or directory".to_string();
            Output::new(Err(Error::new(crate::ErrorType::BadInputPath(
                path_string,
                err_string,
            ))
            .into()))
        }
    }

    /// Merges the given `StoryPassages` into this one, producing a possible
    /// list of [`Warning`]s in the process.
    ///
    /// # Warnings
    /// Produces a warning if a duplicate `StoryTitle` or `StoryData` is found.
    /// The duplicate is ignored and the existing one is kept.
    pub fn merge_from(&mut self, mut other: Self) -> Vec<Warning> {
        let mut warnings = Vec::new();

        match (&self.title, &other.title) {
            (None, Some(_)) => self.title = other.title,
            (Some(_), Some(_)) => {
                let mut warning = Warning::new(WarningType::DuplicateStoryTitle);
                *warning.mut_position() = other.title.unwrap().header.get_position().clone();
                warning.set_referent(self.title.as_ref().unwrap().header.get_position().clone());
                warnings.push(warning)
            }
            _ => (),
        }

        match (&self.data, &other.data) {
            (None, Some(_)) => self.data = other.data,
            (Some(_), Some(_)) => {
                let mut warning = Warning::new(WarningType::DuplicateStoryData);
                *warning.mut_position() = other.data.unwrap().header.get_position().clone();
                warning.set_referent(self.data.as_ref().unwrap().header.get_position().clone());
                warnings.push(warning);
            }
            _ => (),
        }

        self.passages.extend(other.passages);
        self.scripts.append(&mut other.scripts);
        self.stylesheets.append(&mut other.stylesheets);

        warnings
    }

    /// Performs a set of post-parse checks and returns a list of any warnings
    ///
    /// # Warnings
    /// * [`MissingStoryTitle`] - No `StoryTitle` passage found
    /// * [`MissingStoryData`] - No `StoryData` passage found
    /// * [`DeadLink`] - Found a link to a non-existent passage
    /// * [`MissingStartPassage`] - No `Start` passage found and no alternate
    ///   passage set in `StoryData`
    /// * [`DeadStartPassage`] - Alternate start passage set in `StoryData`, but
    ///   no such passage found in parsing
    ///
    /// [`MissingStoryTitle`]: enum.WarningType.html#variant.MissingStoryTitle
    /// [`MissingStoryData`]: enum.WarningType.html#variant.MissingStoryData
    /// [`DeadLink`]: enum.WarningType.html#variant.DeadLink
    /// [`MissingStartPassage`]: enum.WarningType.html#variant.MissingStartPassage
    /// [`DeadStartPassage`]: enum.WarningType.html#variant.DeadStartPassage
    pub fn check(&self) -> Vec<Warning> {
        let mut warnings = Vec::new();
        if self.title.is_none() {
            warnings.push(Warning::new(WarningType::MissingStoryTitle));
        }

        let mut missing_start = !self.passages.contains_key("Start");

        self.data
            .as_ref()
            .or_else(|| {
                // There is no StoryData, generate a warning
                warnings.push(Warning::new(WarningType::MissingStoryData));

                // Return None to prevent additional processing
                None
            })
            .and_then(|passage| {
                // There was an attempt to parse a StoryData passage
                if let PassageContent::StoryData(maybe_data, _) = &passage.content {
                    maybe_data
                        .as_ref()
                        // If there is parsed StoryData, get the start field
                        .and_then(|data| data.start.as_ref())
                        // If there is a start field
                        .and_then(|start| {
                            // Even if the start field is a dead link, it's not
                            // missing a start passage
                            missing_start = false;

                            // Check if the configured start passage exists
                            if !self.passages.contains_key(start) {
                                // There is an alternate start passage specified,
                                // but it does not exist
                                warnings.push(Warning {
                                    warning_type: WarningType::DeadStartPassage(start.clone()),
                                    position: passage.header.position.clone(),
                                    referent: None,
                                });
                            }

                            // Return something
                            Some(())
                        })
                } else {
                    None
                }
            });

        if missing_start {
            warnings.push(Warning::new(WarningType::MissingStartPassage));
        }

        for passage in self.passages.values() {
            if let PassageContent::Normal(twine) = &passage.content {
                for link in twine.get_links() {
                    // Trim the target so that a whitespace warning and a dead
                    // link warning aren't both generated
                    if !self.passages.contains_key(link.target.trim()) {
                        warnings.push(Warning {
                            warning_type: WarningType::DeadLink(link.target.clone()),
                            position: link.position.clone(),
                            referent: None,
                        });
                    }
                }
            }
        }

        warnings
    }
}

impl<'a> Parser<'a> for StoryPassages {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        // The iterator we'll use to walk through the input
        let mut iter = input.iter();
        // The first line must be a header, skip over it so we don't have an
        // empty slice
        iter.next();
        // The starting index of the next passage
        let mut start = 0;

        // Story variables
        let mut title: Option<Passage> = None;
        let mut data: Option<Passage> = None;
        let mut passages = HashMap::new();
        let mut scripts = Vec::new();
        let mut stylesheets = Vec::new();

        // Running list of warnings
        let mut warnings = Vec::new();

        // Running list of errors
        let mut errors = Ok(());

        while start < input.len() {
            // Find the start of the next passage using the sigil (::)
            let pos = iter.position(|&x| x.trim_start().starts_with("::"));

            let pos = if let Some(p) = pos {
                start + p + 1
            } else {
                input.len()
            };
            let passage_input = &input[start..pos];

            // Parse the passage
            let (mut res, mut passage_warnings) =
                Passage::parse(passage_input).with_offset_row(start).take();
            warnings.append(&mut passage_warnings);
            start = pos;

            // If there's an error, update the row before returning
            if res.is_err() {
                errors = ErrorList::merge(&mut errors, &mut res);
                continue;
            }

            let passage = res.ok().unwrap();

            // Handle passage types appropriately
            match &passage.content {
                PassageContent::Normal(_) => {
                    passages.insert(passage.header.name.clone(), passage);
                }
                PassageContent::StoryTitle(_) => {
                    if let Some(existing) = &title {
                        let warning = Warning {
                            warning_type: WarningType::DuplicateStoryTitle,
                            position: passage.header.position.clone(),
                            referent: Some(existing.header.position.clone()),
                        };
                        warnings.push(warning);
                    } else {
                        title = Some(passage);
                    }
                }
                PassageContent::StoryData(_, _) => {
                    if let Some(existing) = &data {
                        let warning = Warning {
                            warning_type: WarningType::DuplicateStoryData,
                            position: passage.header.position.clone(),
                            referent: Some(existing.header.position.clone()),
                        };
                        warnings.push(warning);
                    } else {
                        data = Some(passage);
                    }
                }
                PassageContent::Script(_) => scripts.push(passage),
                PassageContent::Stylesheet(_) => stylesheets.push(passage),
            }
        }

        let story = StoryPassages {
            title,
            data,
            passages,
            scripts,
            stylesheets,
        };
        Output::new(Ok(story)).with_warnings(warnings)
    }
}

impl Positional for StoryPassages {
    fn set_file(&mut self, file: String) {
        if self.title.is_some() {
            self.title.as_mut().unwrap().set_file(file.clone());
        }

        if self.data.is_some() {
            self.data.as_mut().unwrap().set_file(file.clone());
        }

        for passage in self.passages.values_mut() {
            passage.set_file(file.clone());
        }

        for script in &mut self.scripts {
            script.set_file(file.clone());
        }

        for style in &mut self.stylesheets {
            style.set_file(file.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Warning;
    use crate::WarningType;
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
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        assert_eq!(
            warnings[0],
            Warning::new(WarningType::EscapedOpenSquare)
                .with_row(7)
                .with_column(5)
        );
    }

    #[test]
    fn file_input() -> Result<(), Box<dyn std::error::Error>> {
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
        use std::io::Write;
        let dir = tempdir()?;
        let file_path = dir.path().join("test.twee");
        let mut file = File::create(file_path.clone())?;
        writeln!(file, "{}", input)?;

        let out = StoryPassages::from_path(file_path);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
            assert_eq!(
                warnings[0],
                Warning::new(WarningType::EscapedOpenSquare)
                    .with_row(7)
                    .with_column(5)
                    .with_file("test.twee".to_string())
            );
            assert_eq!(warnings[1], Warning::new(WarningType::MissingStoryData));
        } else {
            panic!("Expected StoryTitle");
        }

        Ok(())
    }

    #[test]
    fn dir_input() -> Result<(), Box<dyn std::error::Error>> {
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
        writeln!(file_one, "{}", input_one)?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        writeln!(file_two, "{}", input_two)?;

        let out = StoryPassages::from_path(dir.path());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 2);
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
        } else {
            panic!("Expected StoryTitle");
        }

        assert!(warnings.contains(
            &Warning::new(WarningType::EscapedOpenCurly)
                .with_column(6)
                .with_row(10)
                .with_file("test.twee".to_string())
        ));

        assert!(warnings.contains(
            &Warning::new(WarningType::EscapedCloseSquare)
                .with_column(16)
                .with_row(9)
                .with_file("test2.tw".to_string())
        ));

        Ok(())
    }

    #[test]
    fn multi_path() -> Result<(), Box<dyn std::error::Error>> {
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
        writeln!(file_one, "{}", input_one)?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        writeln!(file_two, "{}", input_two)?;

        let paths = vec![ file_path_one, file_path_two ];
        let out = StoryPassages::from_paths(&paths);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 2);
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
        } else {
            panic!("Expected StoryTitle");
        }

        assert!(warnings.contains(
            &Warning::new(WarningType::EscapedOpenCurly)
                .with_column(6)
                .with_row(10)
                .with_file("test.twee".to_string())
        ));

        assert!(warnings.contains(
            &Warning::new(WarningType::EscapedCloseSquare)
                .with_column(16)
                .with_row(9)
                .with_file("test2.tw".to_string())
        ));

        Ok(())
    }

    #[test]
    fn dir_input_duplicates() -> Result<(), Box<dyn std::error::Error>> {
        let input_one = r#":: Start
At the start, link to [[A passage]]

:: A passage
This passage links to [[Another passage]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "DEF"
}
"#
        .to_string();

        let input_two = r#":: Another passage
Links back to [[Start]]

:: StoryData
{
"ifid": "ABC"
}

:: StoryTitle
A Test Story
"#
        .to_string();

        use std::io::Write;
        let dir = tempdir()?;
        let file_path_one = dir.path().join("test.twee");
        let mut file_one = File::create(file_path_one.clone())?;
        writeln!(file_one, "{}", input_one)?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        writeln!(file_two, "{}", input_two)?;

        let out = StoryPassages::from_path(dir.path());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 2);

        // We can't know the parse order, so we can't know anything other than
        // the type of warnings we expect
        assert!(warnings
            .iter()
            .any(|w| WarningType::DuplicateStoryData == w.warning_type));
        assert!(warnings
            .iter()
            .any(|w| WarningType::DuplicateStoryTitle == w.warning_type));

        assert_eq!(res.is_ok(), true);

        Ok(())
    }

    #[test]
    fn duplicate_story_data() {
        let input = r#":: A passage
blah whatever

:: StoryData
{
"ifid": "ABC"
}

:: StoryTitle
Test Story

:: Start
Link to [[A passage]]

:: StoryData
{
"ifid": "DEF"
}
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0],
            Warning::new(WarningType::DuplicateStoryData)
                .with_column(1)
                .with_row(15)
                .with_referent(crate::Position::RowColumn(4, 1))
        );
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(
            story
                .data
                .and_then(|passage| {
                    if let PassageContent::StoryData(data, _) = passage.content {
                        data
                    } else {
                        None
                    }
                })
                .and_then(|data| Some(data.ifid)),
            Some("ABC".to_string())
        );
    }

    #[test]
    fn duplicate_story_title() {
        let input = r#":: A passage
blah whatever

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "ABC"
}

:: Start
Link to [[A passage]]

:: StoryTitle
Discarded Duplicate Title
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0],
            Warning::new(WarningType::DuplicateStoryTitle)
                .with_column(1)
                .with_row(15)
                .with_referent(crate::Position::RowColumn(4, 1))
        );
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
        } else {
            panic!("Expected StoryTitle");
        }
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
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
        } else {
            panic!("Expected StoryTitle");
        }
    }

    #[test]
    fn dead_link() {
        let input = r#":: Start
This passage links to [[Another passage]]

:: Another passage
This has dead link to [[Dead link]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "abc"
}
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(
            warnings,
            vec![Warning::new(WarningType::DeadLink("Dead link".to_string()))
                .with_row(5)
                .with_column(25)]
        );
    }

    #[test]
    fn alt_start() {
        let input = r#":: Alt Start
This passage links to [[Another passage]]

:: Another passage
This links back to [[Alt Start]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "abc",
"start": "Alt Start"
}
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert!(warnings.is_empty());
    }

    #[test]
    fn dead_start() {
        let input = r#":: Alt Start
This passage links to [[Another passage]]

:: Another passage
This links back to [[Alt Start]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "abc",
"start": "Alternate Start"
}
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(
            warnings,
            vec![
                Warning::new(WarningType::DeadStartPassage("Alternate Start".to_string()))
                    .with_row(10)
                    .with_column(1)
            ]
        );
    }

    #[test]
    fn missing_title() {
        let input = r#":: Start
blah blah

::StoryData
{"ifid": "ABC"}"#
            .to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(warnings, vec![Warning::new(WarningType::MissingStoryTitle)]);
    }

    #[test]
    fn missing_start() {
        let input = r#":: Alt Start
This passage links to [[Another passage]]

:: Another passage
This links back to [[Alt Start]]

:: StoryTitle
Test Story

:: StoryData
{
"ifid": "abc"
}
"#
        .to_string();
        let out = StoryPassages::from_string(input);
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(
            warnings,
            vec![Warning::new(WarningType::MissingStartPassage)]
        );
    }
}
