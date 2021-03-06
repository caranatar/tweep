use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Position;
use crate::Warning;
use crate::WarningKind;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The content of a special passage with the name StoryData that contains a
/// JSON blob with various information about the story.
///
/// # JSON fields
/// Fields as defined by the Twee 3 specification:
/// * ifid - Rquired. String. Interactive Fiction IDentifier v4 UUID
/// * format - String. Maps to <tw-storydata format>.
/// * format-version - String. Maps to <tw-storydata format-version>.
/// * start - String. Maps to <tw-passagedata name> of the node whose pid matches <tw-storydata startnode>.
/// * tag-colors - Object of tag(string):color(string) pairs. Pairs map to <tw-tag> nodes as <tw-tag name>:<tw-tag color>.
/// * zoom - Decimal. Maps to <tw-storydata zoom>.
///
/// # Parse Errors
/// None
///
/// # Parse Warnings
/// * [`JsonError`] - Error encountered while parsing the JSON content
#[derive(Debug, Serialize, Deserialize)]
pub struct StoryData {
    /// Interactive Fiction IDentifier v4 UUID
    pub ifid: String,

    /// The story format
    pub format: Option<String>,

    /// The version of the story format
    #[serde(rename = "format-version")]
    pub format_version: Option<String>,

    /// The starting passage
    pub start: Option<String>,

    /// Map of tag name to color name for coloring tags
    #[serde(rename = "tag-colors")]
    pub tag_colors: Option<HashMap<String, String>>,

    /// Zoom level for editing in Twine
    pub zoom: Option<f32>,
}

impl StoryData {
    /// Parses a `StoryData` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Option<Self>, ErrorList>> {
        let mut warnings = Vec::new();
        let res: serde_json::Result<StoryData> = serde_json::from_str(context.get_contents());

        let story_data = if res.is_ok() {
            Some(res.ok().unwrap())
        } else {
            let err = res.err().unwrap();
            // Get the error part of error string generated by serde
            let err_string = format!("{}", err).split(" at ").next().unwrap().to_string();
            warnings.push(Warning::new(
                WarningKind::JsonError(err_string),
                Some(context.subcontext(
                    Position::rel(err.line(), err.column())
                        ..=Position::rel(err.line(), err.column()),
                )),
            ));
            None
        };
        Output::new(Ok(story_data)).with_warnings(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let input = r#"{
	"ifid": "D674C58C-DEFA-4F70-B7A2-27742230C0FC",
	"format": "SugarCube",
	"format-version": "2.28.2",
	"start": "My Starting Passage",
	"tag-colors": {
		"bar": "green",
		"foo": "red",
		"qaz": "blue"
	},
	"zoom": 0.25
}
"#
        .to_string();
        let out = StoryData::parse(FullContext::from(None, input));
        assert!(!out.has_warnings());
        let (res, _) = out.take();
        assert!(res.is_ok());
        let data = res.ok().unwrap();
        let expected = if let Some(story_data) = data {
            assert_eq!(story_data.ifid, "D674C58C-DEFA-4F70-B7A2-27742230C0FC");
            assert_eq!(story_data.format, Some("SugarCube".to_string()));
            assert_eq!(story_data.format_version, Some("2.28.2".to_string()));
            assert_eq!(story_data.start, Some("My Starting Passage".to_string()));
            assert_eq!(story_data.zoom, Some(0.25));

            let expected = if let Some(tag_colors) = story_data.tag_colors {
                assert_eq!(tag_colors["bar"], "green");
                assert_eq!(tag_colors["foo"], "red");
                assert_eq!(tag_colors["qaz"], "blue");

                true
            } else {
                false
            };
            assert!(expected);

            true
        } else {
            false
        };
        assert!(expected);
    }

    #[test]
    fn test_malformed() {
        let input = r#"{
	"ifid": "D674C58C-DEFA-4F70-B7A2-27742230C0FC",
	"format": "SugarCube",
	"format-version": "2.28.2",
	"start": "My Starting Passage",
	"tag-colors": {
"#
        .to_string();
        let out = StoryData::parse(FullContext::from(None, input));
        assert!(out.has_warnings());
        let (res, warnings) = out.take();
        assert!(res.is_ok());
        let data = res.ok().unwrap();
        assert!(data.is_none());
        assert_eq!(warnings.len(), 1);
        assert!(
            if let WarningKind::JsonError(_) = &warnings[0].kind {
                true
            } else {
                false
            }
        );
    }
}
