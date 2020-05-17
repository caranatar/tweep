use crate::ErrorList;
use crate::Output;
use crate::PassageContent;
use crate::PassageHeader;
use crate::Positional;
use crate::ScriptContent;
use crate::StoryData;
use crate::StoryTitle;
use crate::StylesheetContent;
use crate::TwineContent;
use crate::FullContext;
use crate::ContextPosition;

/// A complete Twee passage, including header and content
///
/// # Parse Errors
/// See [`PassageHeader`] and possible [`PassageContent`] variants
///
/// # Parse Warnings
/// See [`PassageHeader`] and possible [`PassageContent`] variants
///
/// [`PassageHeader`]: struct.PassageHeader.html
/// [`PassageContent`]: enum.PassageContent.html
pub struct Passage {
    /// The header
    pub header: PassageHeader,

    /// The content
    pub content: PassageContent,
}

impl Passage {
    /// Creates a new `Output<Result<Passage, ErrorList>>` from the parse output
    /// of a `PassageHeader` and a `PassageContent`, along with a composed list
    /// of `Warning`s from both
    ///
    /// If either `header` or `content` contain errors, the result of this will
    /// be a list of errors.
    ///
    /// # Examples
    /// ```
    /// # use tweep::{FullContext, Passage, PassageHeader, PassageContent, StoryTitle};
    /// let context = FullContext::from(None, ":: StoryTitle".to_string());
    /// let header = PassageHeader::parse(context);
    /// let context = FullContext::from(None, "A title".to_string());
    /// let content = StoryTitle::parse(context);
    /// let passage = Passage::new(header, content.into_result());
    /// assert!(passage.is_ok());
    /// ```
    pub fn new(
        header: Output<Result<PassageHeader, ErrorList>>,
        content: Output<Result<PassageContent, ErrorList>>,
    ) -> Output<Result<Self, ErrorList>> {
        // Move out the header and its associated warnings
        let (mut header_res, mut warnings) = header.take();

        // Move out the content and its associated warnings
        let (mut content_res, mut content_warnings) = content.take();

        // Consolidate the warnings
        warnings.append(&mut content_warnings);

        // Consolidate the Errors if there are any
        let possible_errors = ErrorList::merge(&mut header_res, &mut content_res);

        // Create and return the completed Output
        Output::new(match possible_errors {
            Err(e) => Err(e),
            Ok(_) => {
                let header = header_res.ok().unwrap();
                let content = content_res.ok().unwrap();
                Ok(Passage { header, content })
            }
        })
        .with_warnings(warnings)
    }

    /// Returns a reference to the metadata contained by the `header` field
    pub fn metadata(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.header.metadata
    }

    /// Returns a reference to the list of tags contained by the `header` field
    pub fn tags(&self) -> &Vec<String> {
        &self.header.tags
    }

    pub(crate) fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        let header_context = context.subcontext(context.line_range(1));
        // Parse the first line as the header
        let mut header = PassageHeader::parse(header_context);
        header.set_row(1);

        // Since we can't know how to parse the passage contents if we don't know
        // the passage type from the header, we can't continue
        if header.is_err() {
            return header.into_err();
        }

        // Get a reference to the result, convert it into a Result of references
        // get the Ok side and unwrap it, getting a reference to the header
        let header_ref = header.get_output().as_ref().ok().unwrap();

        // Find the position of the last non-empty line
        let mut new_iter = context.get_contents().split('\n');
        new_iter.rfind(|&x| !x.is_empty());
        let len = new_iter.fold(0, |acc, _| acc + 1);

        // Create the content's context
        let content_context = context.subcontext(ContextPosition::new(2,1)..=context.end_of_line(len+1));

        // Parse the content based on the type indicated by the header
        let content: Output<Result<PassageContent, ErrorList>>;
        content = if header_ref.name == "StoryTitle" {
            StoryTitle::parse(content_context).into_result()
        } else if header_ref.name == "StoryData" {
            StoryData::parse(content_context).into_result()
        } else if header_ref.has_tag("script") {
            ScriptContent::parse(content_context).into_result()
        } else if header_ref.has_tag("stylesheet") {
            StylesheetContent::parse(content_context).into_result()
        } else {
            TwineContent::parse(content_context).into_result()
        };

        // Assemble and return the output
        Self::new(header, content.with_offset_row(1))
    }
}

impl Positional for Passage {
    fn set_row(&mut self, row: usize) {
        self.header.set_row(row);
        self.content.set_row(row);
    }

    fn set_column(&mut self, col: usize) {
        self.header.set_column(col);
        self.content.set_column(col);
    }

    fn offset_column(&mut self, offset: usize) {
        self.header.offset_column(offset);
        self.content.offset_column(offset);
    }

    fn offset_row(&mut self, offset: usize) {
        self.header.offset_row(offset);
        self.content.offset_row(offset);
    }

    fn set_file(&mut self, file: String) {
        self.header.set_file(file.clone());
        self.content.set_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn story_title_subtest(input: String, expected_title: &str) {
        let context = FullContext::from(None, input);
        let out = Passage::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let passage = res.ok().unwrap();
        let content = passage.content;
        let expected = if let PassageContent::StoryTitle(story_title) = content {
            assert_eq!(story_title.title, expected_title);
            true
        } else {
            false
        };
        assert_eq!(expected, true);
    }

    #[test]
    fn one_line_story_title() {
        let input = ":: StoryTitle\nOne line story title\n\n".to_string();
        story_title_subtest(input, "One line story title");
    }

    #[test]
    fn multi_line_story_title() {
        let input = "::StoryTitle\nMulti\nLine\nTitle".to_string();
        story_title_subtest(input, "Multi\nLine\nTitle")
    }
    
    #[test]
    fn script_passage() {
        let input = ":: Script Passage [script]\nfoo\nbar".to_string();
        let context = FullContext::from(None, input);
        let out = Passage::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let passage = res.ok().unwrap();
        assert_eq!(passage.tags(), &vec!["script".to_string()]);
        let content = passage.content;
        let expected = if let PassageContent::Script(script) = content {
            assert_eq!(passage.header.name, "Script Passage");
            assert_eq!(script.content, "foo\nbar");
            true
        } else {
            false
        };
        assert_eq!(expected, true);
    }

    #[test]
    fn stylesheet_passage() {
        let input = ":: Style Passage [stylesheet]\nfoo\nbar".to_string();
        let context = FullContext::from(None, input);
        let out = Passage::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let passage = res.ok().unwrap();
        assert_eq!(passage.metadata()["position"], "10,10");
        assert_eq!(passage.metadata()["size"], "100,100");
        let content = passage.content;
        let expected = if let PassageContent::Stylesheet(stylesheet) = content {
            assert_eq!(passage.header.name, "Style Passage");
            assert_eq!(stylesheet.content, "foo\nbar");
            true
        } else {
            false
        };
        assert_eq!(expected, true);
    }

    #[test]
    fn a_test() {
        let input_string = r#":: An overgrown path[tag  tag2 ]
This
That



"#.to_string();
        let context = FullContext::from(None, input_string);
        let out = Passage::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let passage = res.ok().unwrap();
        let content = passage.content;
        let expected = if let PassageContent::Normal(normal) = content {
            assert_eq!(passage.header.name, "An overgrown path");
            assert_eq!(normal.content, "This\nThat\n");
            true
        } else {
            false
        };
        assert_eq!(expected, true);
    }
}
