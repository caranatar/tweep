#[cfg(feature = "full-context")]
use crate::CodeMap;
use crate::Context;
#[cfg(feature = "full-context")]
use crate::ContextErrorList;
use crate::Error;
use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Passage;
use crate::PassageContent;
use crate::Position;
use crate::PositionKind;
use crate::Warning;
use crate::WarningKind;
#[cfg(feature = "full-context")]
use bimap::BiMap;
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[cfg(not(feature = "full-context"))]
type ParseOutput = Output<Result<StoryPassages, ErrorList>>;
#[cfg(feature = "full-context")]
type ParseOutput = Output<Result<StoryPassages, ContextErrorList>>;

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

    /// StoryMap for this story
    #[cfg(feature = "full-context")]
    pub code_map: CodeMap,
}

impl StoryPassages {
    /// Renumber pids, starting at the given number and counting up
    fn renumber_pids(&mut self, start: usize) {
        let mut pid = start;
        for passage in self.passages.values_mut() {
            if let PassageContent::Normal(twine) = &mut passage.content {
                twine.pid = pid;
            }

            pid += 1;
        }
    }

    #[cfg(feature = "full-context")]
    fn renumber_file_ids(&mut self, start: usize) {
        let mut new_id_file_map = BiMap::new();
        let mut new_contexts = HashMap::new();
        for (id, context) in self.code_map.contexts.drain() {
            let new_id = id + start;
            new_id_file_map.insert(new_id, context.get_file_name().clone().unwrap());
            new_contexts.insert(new_id, context);
        }
        self.code_map.id_file_map = new_id_file_map;
        self.code_map.contexts = new_contexts;
    }

    /// Parses an input `String` and returns the result or a list of errors,
    /// along with a list of any [`Warning`]s
    ///
    /// [`Warning`]: struct.Warning.html
    pub fn from_string(input: String) -> ParseOutput {
        let context = FullContext::from(None, input);
        StoryPassages::from_context(context)
    }

    pub(crate) fn from_context(context: FullContext) -> ParseOutput {
        let mut out = StoryPassages::parse(context);
        if out.is_ok() {
            out.mut_output().as_mut().ok().unwrap().renumber_pids(1);
        }
        out
    }

    /// Parses a `StoryPassages` from the given [`Path`]. If the given path is
    /// a file, parses that file and returns the `StoryPassages`. If it is a
    /// directory, it looks for any files with `.tw` or `.twee` extensions and
    /// parses them. Returns the parsed output or a list of errors, along with a
    /// list of any [`Warning`]s
    ///
    /// [`Path`]: std::path::Path
    /// [`Warning`]: struct.Warning.html
    pub fn from_path<P: AsRef<Path>>(input: P) -> ParseOutput {
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
    pub fn from_paths<P: AsRef<Path>>(input: &[P]) -> ParseOutput {
        let mut story = StoryPassages::default();
        let mut warnings = Vec::new();
        for path in input {
            let out = StoryPassages::from_path_internal(path);
            let (res, mut sub_warnings) = out.take();
            warnings.append(&mut sub_warnings);
            #[allow(unused_mut)]
            if let Err(mut e) = res {
                #[cfg(feature = "full-context")]
                {
                    story.renumber_file_ids(e.code_map.contexts.len());
                    e.code_map.contexts.extend(story.code_map.contexts);
                    for (id, file_name) in story.code_map.id_file_map.iter() {
                        e.code_map.id_file_map.insert(*id, file_name.clone());
                    }
                }
                return Output::new(Err(e)).with_warnings(warnings);
            }
            let sub_story = res.ok().unwrap();
            let mut merge_warnings = story.merge_from(sub_story);
            warnings.append(&mut merge_warnings);
        }

        let mut story_warnings = story.check();
        warnings.append(&mut story_warnings);

        Output::new(Ok(story)).with_warnings(warnings)
    }

    /// Does the heavy lifting for `from_path`. If given a file, reads its
    /// contents into a `String` and uses `from_context` to parse it. If given a
    /// directory, finds the twee files, recurses with each file, then assembles
    /// the outputs into a single output
    fn from_path_internal<P: AsRef<Path>>(input: P) -> ParseOutput {
        // Get the path
        let path: &Path = input.as_ref();

        // Convert path to string
        let path_string: String = path.to_string_lossy().to_owned().to_string();

        if path.is_file() {
            // If path is a file, get the file name part
            let file_name: String = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_owned()
                .to_string();

            // Open the file
            let file = File::open(path);

            if file.is_err() {
                // Check for errors, return Error if we can't open file
                let err_string = format!("{}", file.err().unwrap());
                return Output::new(Err(Error::new(
                    crate::ErrorKind::BadInputPath(path_string, err_string),
                    Some(FullContext::from(None, file_name)),
                )
                .into()));
            }

            // Get the file
            let mut file = file.ok().unwrap();

            // Slurp the file contents
            let mut contents = String::new();
            let res = file.read_to_string(&mut contents);

            if res.is_err() {
                // Return an error if we can't read the file
                let err_string = format!("{}", res.err().unwrap());
                return Output::new(Err(Error::new(
                    crate::ErrorKind::BadInputPath(path_string, err_string),
                    Some(FullContext::from(None, file_name)),
                )
                .into()));
            }

            // Create the object from the contents, add file name to Positions
            let context = FullContext::from(Some(file_name), contents);
            StoryPassages::from_context(context)
        } else if path.is_dir() {
            let dir = std::fs::read_dir(path);
            if dir.is_err() {
                let err_string = format!("{}", dir.err().unwrap());
                return Output::new(Err(Error::new::<Context>(
                    crate::ErrorKind::BadInputPath(path_string, err_string),
                    None,
                )
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
            Output::new(Err(Error::new::<Context>(
                crate::ErrorKind::BadInputPath(path_string, err_string),
                None,
            )
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

        other.renumber_pids(self.passages.len() + 1);

        #[cfg(feature = "full-context")]
        {
            other.renumber_file_ids(self.code_map.contexts.len());
            self.code_map.contexts.extend(other.code_map.contexts);
            for (id, file_name) in other.code_map.id_file_map.iter() {
                self.code_map.id_file_map.insert(*id, file_name.clone());
            }
        }

        match (&self.title, &other.title) {
            (None, Some(_)) => self.title = other.title,
            (Some(self_title), Some(other_title)) => {
                let mut warning = Warning::new(
                    WarningKind::DuplicateStoryTitle,
                    Some(other_title.context.clone()),
                );
                warning.set_referent(self_title.context.clone());
                warnings.push(warning)
            }
            _ => (),
        }

        match (&self.data, &other.data) {
            (None, Some(_)) => self.data = other.data,
            (Some(self_data), Some(other_data)) => {
                let mut warning = Warning::new(
                    WarningKind::DuplicateStoryData,
                    Some(other_data.context.clone()),
                );
                warning.set_referent(self_data.context.clone());
                warnings.push(warning);
            }
            _ => (),
        }

        for (name, passage) in other.passages.drain() {
            let entry = self.passages.entry(name.clone());
            use std::collections::hash_map::Entry::*;
            match entry {
                Vacant(_) => {
                    entry.or_insert(passage);
                },
                Occupied(v) => {
                    let warning = Warning::new(WarningKind::DuplicatePassage(name), Some(passage.context.clone())).with_referent(v.get().context.clone());
                    warnings.push(warning);
                }
            }
        }

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
    /// [`MissingStoryTitle`]: enum.WarningKind.html#variant.MissingStoryTitle
    /// [`MissingStoryData`]: enum.WarningKind.html#variant.MissingStoryData
    /// [`DeadLink`]: enum.WarningKind.html#variant.DeadLink
    /// [`MissingStartPassage`]: enum.WarningKind.html#variant.MissingStartPassage
    /// [`DeadStartPassage`]: enum.WarningKind.html#variant.DeadStartPassage
    pub fn check(&self) -> Vec<Warning> {
        let mut warnings = Vec::new();
        if self.title.is_none() {
            warnings.push(Warning::new::<Context>(
                WarningKind::MissingStoryTitle,
                None,
            ));
        }

        let mut missing_start = !self.passages.contains_key("Start");

        self.data
            .as_ref()
            .or_else(|| {
                // There is no StoryData, generate a warning
                warnings.push(Warning::new::<Context>(WarningKind::MissingStoryData, None));

                // Return None to prevent additional processing
                None
            })
            .and_then(|passage| {
                // There was an attempt to parse a StoryData passage
                if let PassageContent::StoryData(maybe_data) = &passage.content {
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
                                warnings.push(Warning::new(
                                    WarningKind::DeadStartPassage(start.clone()),
                                    Some(passage.context.clone()),
                                ));
                            }

                            // Return something
                            Some(())
                        })
                } else {
                    None
                }
            });

        if missing_start {
            warnings.push(Warning::new::<Context>(
                WarningKind::MissingStartPassage,
                None,
            ));
        }

        for passage in self.passages.values() {
            if let PassageContent::Normal(twine) = &passage.content {
                for link in twine.get_links() {
                    // Trim the target so that a whitespace warning and a dead
                    // link warning aren't both generated
                    if !self.passages.contains_key(link.target.trim()) {
                        warnings.push(Warning::new(
                            WarningKind::DeadLink(link.target.clone()),
                            Some(link.context.clone()),
                        ));
                    }
                }
            }
        }

        warnings
    }

    /// If a start passage is configured in the StoryData, return the name of
    /// that passage. If no start passage is configured, check for the presence
    /// of a passage called "Start". If that passage exists, return that name,
    /// otherwise return None
    pub fn get_start_passage_name(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|d| match &d.content {
                PassageContent::StoryData(story_data) => story_data.as_ref(),
                _ => None,
            })
            .and_then(|d| d.start.as_deref())
            .or_else(|| {
                if self.passages.contains_key("Start") {
                    Some("Start")
                } else {
                    None
                }
            })
    }

    pub(crate) fn parse(context: FullContext) -> ParseOutput {
        let contents = context.get_contents();

        #[cfg(feature = "full-context")]
        let mut code_map = CodeMap::default();

        // Story variables
        let mut title: Option<Passage> = None;
        let mut data: Option<Passage> = None;
        let mut passages:HashMap<String, Passage> = HashMap::new();
        let mut scripts = Vec::new();
        let mut stylesheets = Vec::new();

        // Running list of warnings
        let mut warnings = Vec::new();

        // Running list of errors
        let mut errors = Ok(());

        // Get an iterator to go through each line
        let mut iter = contents.split('\n').enumerate();
        // The first line must be a header, skip over it so we don't have an
        // empty slice
        iter.next();

        // The starting position of the current passage
        let mut start = Position::rel(1, 1);

        let end_line = context.get_end_position().line;
        while start.line <= end_line {
            let subcontext_start = start;
            let subcontext_end =
                if let Some((i, _)) = iter.find(|&(_, line)| line.trim_start().starts_with("::")) {
                    context.end_of_line(i, PositionKind::Relative)
                } else {
                    *context.get_end_position()
                };

            let next_line = subcontext_end.line + 1;
            let subcontext = context.subcontext(subcontext_start..=subcontext_end);
            // Parse the passage
            let (mut res, mut passage_warnings) = Passage::parse(subcontext).take();
            warnings.append(&mut passage_warnings);

            // Update the start position
            start = Position::rel(next_line, 1);

            // If there's an error, update the row before returning
            if res.is_err() {
                errors = ErrorList::merge(&mut errors, &mut res);
                continue;
            }

            let passage = res.ok().unwrap();

            // Handle passage types appropriately
            match &passage.content {
                PassageContent::Normal(_) => {
                    let name = &passage.header.name;
                    if passages.contains_key(name) {
                        warnings.push(Warning::new(WarningKind::DuplicatePassage(name.clone()), Some(passage.context.clone())).with_referent(passages.get(name).unwrap().context.clone()));
                    } else {
                        passages.insert(name.clone(), passage);
                    }
                }
                PassageContent::StoryTitle(_) => {
                    if let Some(existing) = &title {
                        let mut warning = Warning::new(
                            WarningKind::DuplicateStoryTitle,
                            Some(passage.context.clone()),
                        );
                        warning.set_referent(existing.context.clone());
                        warnings.push(warning);
                    } else {
                        title = Some(passage);
                    }
                }
                PassageContent::StoryData(_) => {
                    if let Some(existing) = &data {
                        let mut warning = Warning::new(
                            WarningKind::DuplicateStoryData,
                            Some(passage.context.clone()),
                        );
                        warning.set_referent(existing.context.clone());
                        warnings.push(warning);
                    } else {
                        data = Some(passage);
                    }
                }
                PassageContent::Script(_) => scripts.push(passage),
                PassageContent::Stylesheet(_) => stylesheets.push(passage),
            }
        }

        #[cfg(feature = "full-context")]
        code_map.add(context);
        match errors {
            Ok(_) => {
                let story = StoryPassages {
                    title,
                    data,
                    passages,
                    scripts,
                    stylesheets,
                    #[cfg(feature = "full-context")]
                    code_map,
                };
                Output::new(Ok(story))
            }
            Err(e) => {
                #[cfg(feature = "full-context")]
                let e = ContextErrorList {
                    error_list: e,
                    code_map,
                };
                Output::new(Err(e))
            }
        }
        .with_warnings(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let context = FullContext::from(None, input.clone());
        let out = StoryPassages::from_string(input);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
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
        write!(file, "{}", input.clone())?;

        let out = StoryPassages::from_path(file_path);
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(story.title.is_some(), true);
        let title_content = story.title.unwrap().content;
        let context = FullContext::from(Some("test.twee".to_string()), input);
        if let PassageContent::StoryTitle(title) = title_content {
            assert_eq!(title.title, "Test Story");
            assert_eq!(warnings[0], {
                let warning = Warning::new(
                    WarningKind::EscapedOpenSquare,
                    Some(context.subcontext(Position::rel(7, 5)..=Position::rel(7, 6))),
                );
                warning
            });
            assert_eq!(
                warnings[1],
                Warning::new::<Context>(WarningKind::MissingStoryData, None)
            );
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
        write!(file_one, "{}", input_one.clone())?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        write!(file_two, "{}", input_two.clone())?;

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
        write!(file_one, "{}", input_one.clone())?;
        let file_path_two = dir.path().join("test2.tw");
        let mut file_two = File::create(file_path_two.clone())?;
        write!(file_two, "{}", input_two.clone())?;

        let paths = vec![file_path_one, file_path_two];
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
            .any(|w| WarningKind::DuplicateStoryData == w.kind));
        assert!(warnings
            .iter()
            .any(|w| WarningKind::DuplicateStoryTitle == w.kind));

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
        let context = FullContext::from(None, input);
        let out = StoryPassages::from_context(context.clone());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0],
            Warning::new(
                WarningKind::DuplicateStoryData,
                Some(context.subcontext(Position::rel(15, 1)..=Position::abs(18, 1)))
            )
            .with_referent(story.data.as_ref().unwrap().context.clone())
        );

        assert_eq!(
            story
                .data
                .and_then(|passage| {
                    if let PassageContent::StoryData(data) = passage.content {
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
        let context = FullContext::from(None, input);
        let out = StoryPassages::from_context(context.clone());
        assert_eq!(out.has_warnings(), true);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0],
            Warning::new(
                WarningKind::DuplicateStoryTitle,
                Some(context.subcontext(Position::rel(15, 1)..=Position::abs(16, 25)))
            )
            .with_referent(story.title.as_ref().unwrap().context.clone())
        );
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
        let context = FullContext::from(None, input);
        let out = StoryPassages::from_context(context.clone());
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        #[allow(unused_mut)]
        let expected = vec![Warning::new(
            WarningKind::DeadLink("Dead link".to_string()),
            Some(context.subcontext(Position::rel(5, 23)..=Position::rel(5, 35))),
        )];
        assert_eq!(warnings, expected);
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
        assert_eq!(story.get_start_passage_name(), Some("Alt Start"));
    }

    #[test]
    fn empty_passage() {
        let input = r#":: Snoopy [dog peanuts]
Snoopy is a dog in the comic Peanuts.

::Blah

:: Foo[bar]

:: Charlie Brown [person peanuts] {"position":"600,400","size":"100,200"}
Charlie Brown is a person in the comic Peanuts

:: Styling [stylesheet]
body {font-size: 1.5em;}

:: StoryData
{
    "ifid": "2B68ECD6-348F-4CF5-96F8-549A512A8128",
    "format": "Harlowe",
    "formatVersion": "2.1.0",
    "zoom": 100
}"#
        .to_string();
        let context = FullContext::from(None, input);
        let out = StoryPassages::parse(context);
        assert_eq!(out.has_warnings(), false);
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
        let context = FullContext::from(None, input);
        let out = StoryPassages::from_context(context.clone());
        let (res, mut warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let story = res.ok().unwrap();
        let mut check_warnings = story.check();
        warnings.append(&mut check_warnings);
        assert_eq!(
            warnings,
            vec![Warning::new(
                WarningKind::DeadStartPassage("Alternate Start".to_string()),
                Some(context.subcontext(Position::rel(10, 1)..=Position::abs(14, 1)))
            )]
        );
        assert_eq!(story.get_start_passage_name(), Some("Alternate Start"));
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
        assert_eq!(
            warnings,
            vec![Warning::new::<Context>(WarningKind::MissingStoryTitle, None)]
        );
        assert_eq!(story.get_start_passage_name(), Some("Start"));
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
            vec![Warning::new::<Context>(WarningKind::MissingStartPassage, None)]
        );
        assert_eq!(story.get_start_passage_name(), None);
    }

    #[test]
    fn from_string_error() {
        let input = "".to_string();
        let out = StoryPassages::from_string(input);
        assert!(out.is_err());
    }
}
