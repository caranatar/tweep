use crate::issues::*;
#[cfg(feature = "issue-context")]
use crate::Contextual;
use crate::FullContext;
use crate::Output;
use crate::Position;
use crate::Positional;
use crate::ContextPosition;

use std::ops::Range;

use serde_json::json;

/// A passage header, along with associated [`Position`], tags, and metadata
///
/// # Parse Errors
/// * [`LeadingWhitespace`] - Whitespace before the `::` sigil on a header line
/// * [`MissingSigil`] - No `::` sigil at the beginning of the header line
/// * [`MetadataBeforeTags`] - Metadata and tags present but in wrong order
/// * [`UnclosedTagBlock`] - Tag block present but unclosed
/// * [`UnescapedOpenCurly`] - Unescaped `{` character in passage name
/// * [`UnescapedCloseCurly`] - Unescaped `}` character in passage name
/// * [`UnescapedOpenSquare`] - Unescaped `[` character in passage name
/// * [`UnescapedCloseSquare`] - Unescaped `]` character in passage name
/// * [`EmptyName`] - No passage name present in header line
///
/// # Parse Warnings
/// * [`JsonError`] - Error encountered when parsing metadata
/// * [`EscapedOpenCurly`] - `\{` present in passage name
/// * [`EscapedCloseCurly`] - `\}` present in passage name
/// * [`EscapedOpenSquare`] - `\[` present in passage name
/// * [`EscapedCloseSquare`] - `\]` present in passage name
///
/// # Examples
/// ```
/// use tweep::{FullContext, PassageHeader};
/// let input = r#":: A passage [ tag1 tag2 ] { "position": "5,5" }"#;
/// let context = FullContext::from(None, input.to_string());
/// let header = PassageHeader::parse(context);
/// # assert!(!header.has_warnings());
/// # let (header, _) = header.take();
/// # let header = header.ok().unwrap();
/// # assert_eq!(header.name, "A passage");
/// ```
///
/// [`Position`]: enum.Position.html
/// [`LeadingWhitespace`]: enum.ErrorType.html#variant.LeadingWhitespace
/// [`MissingSigil`]: enum.ErrorType.html#variant.MissingSigil
/// [`MetadataBeforeTags`]: enum.ErrorType.html#variant.MetadataBeforeTags
/// [`UnclosedTagBlock`]: enum.ErrorType.html#variant.UnclosedTagBlock
/// [`UnescapedOpenCurly`]: enum.ErrorType.html#variant.UnescapedOpenCurly
/// [`UnescapedCloseCurly`]: enum.ErrorType.html#variant.UnescapedCloseCurly
/// [`UnescapedOpenSquare`]: enum.ErrorType.html#variant.UnescapedOpenSquare
/// [`UnescapedCloseSquare`]: enum.ErrorType.html#variant.UnescapedCloseSquare
/// [`EmptyName`]: enum.ErrorType.html#variant.EmptyName
/// [`JsonError`]: enum.WarningType.html#variant.JsonError
/// [`EscapedOpenCurly`]: enum.WarningType.html#variant.EscapedOpenCurly
/// [`EscapedCloseCurly`]: enum.WarningType.html#variant.EscapedCloseCurly
/// [`EscapedOpenSquare`]: enum.WarningType.html#variant.EscapedOpenSquare
/// [`EscapedCloseSquare`]: enum.WarningType.html#variant.EscapedCloseSquare
#[derive(Debug)]
pub struct PassageHeader {
    /// The name of the header. This can be a Twine passage name or a special name
    pub name: String,

    /// The list of comma separated tags
    pub tags: Vec<String>,

    /// A json object containing metadata for the passage
    pub metadata: serde_json::Map<String, serde_json::Value>,

    /// The position of the header
    pub position: Position,
}

impl PassageHeader {
    /// Returns `true` if this header is tagged with `str`
    ///
    /// # Examples
    /// ```
    /// use tweep::{FullContext, PassageHeader};
    /// let context = FullContext::from(None, ":: A passage [ foo bar ]".to_string());
    /// let out = PassageHeader::parse(context);
    /// assert!(out.get_output().as_ref().ok().unwrap().has_tag("foo"));
    /// ```
    pub fn has_tag(&self, tag: &str) -> bool {
        let tag = tag.to_string();
        self.tags.contains(&tag)
    }

    /// Parses a `PassageHeader` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        let mut warnings = Vec::new();
        let mut errors = ErrorList::default();
        let input = context.get_contents();

        // Check for sigil
        if !input.starts_with("::") {
            // If the sigil is not present, check for leading whitespace
            let trimmed = input.trim_start();

            // Generate appropriate error
            errors.push(
                if trimmed.starts_with("::") {
                    let err = Error::new(ErrorType::LeadingWhitespace, context.subcontext(..));
                    #[cfg(not(feature = "issue-context"))]
                    {
                        err
                    }
                    #[cfg(feature = "issue-context")]
                    {
                        err.with_context_len(input.len() - trimmed.len())
                    }
                } else {
                    let err = Error::new(ErrorType::MissingSigil, context.subcontext(..));
                    #[cfg(not(feature = "issue-context"))]
                    {
                        err
                    }
                    #[cfg(feature = "issue-context")]
                    {
                        err.with_context_len(1)
                    }
                }
                .with_column(1),
            );
        }

        // Check for metadata
        let mut name_end_pos = input.len();

        // Default metadata
        let metadata = json!({ "position": "10,10", "size":"100,100" });
        let mut metadata = if let serde_json::Value::Object(map) = metadata {
            map
        } else {
            panic!("Unreachable: Failed to extract map from JSON object");
        };

        if let Some(range) = guess_metadata_range(input) {
            let pos = range.start;
            name_end_pos = pos;

            if find_last_unescaped(&input[range.end..], "[").is_some() {
                let error = Error::new(ErrorType::MetadataBeforeTags, context.subcontext(ContextPosition::new(1, pos+1)..)).with_column(pos + 1);
                #[cfg(not(feature = "issue-context"))]
                {
                    errors.push(error);
                }
                #[cfg(feature = "issue-context")]
                {
                    errors.push(error.with_context_len(range.end - pos));
                }
            }

            let meta_str = &input[range];
            let res = parse_metadata(meta_str);
            if res.is_ok() {
                for (k, v) in res.ok().unwrap().iter() {
                    metadata.insert(k.to_string(), v.clone());
                }
            } else {
                warnings.push(res.err().unwrap().with_offset_column(pos));
            }
        }

        // Check for tags
        let mut tags: Vec<String> = Vec::new();
        if let Some(pos) = find_last_unescaped(&input[..name_end_pos], "[") {
            let end_pos = find_last_unescaped(&input[pos + 1..name_end_pos], "]");

            if let Some(p) = end_pos {
                tags = input[pos + 1..pos + 1 + p]
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else {
                let error = Error::new(ErrorType::UnclosedTagBlock, context.subcontext(ContextPosition::new(1, pos+1)..)).with_column(pos + 1);
                #[cfg(not(feature = "issue-context"))]
                {
                    errors.push(error);
                }
                #[cfg(feature = "issue-context")]
                {
                    errors.push(error.with_context_len(name_end_pos - pos));
                }
            }

            name_end_pos = std::cmp::min(name_end_pos, pos);
        }

        // Check for unescaped special characters in the name portion. This also
        // produces a list of warning locations for escaped chars in the name
        for (c, e, w) in vec![
            (
                "{",
                ErrorType::UnescapedOpenCurly,
                WarningType::EscapedOpenCurly,
            ),
            (
                "}",
                ErrorType::UnescapedCloseCurly,
                WarningType::EscapedCloseCurly,
            ),
            (
                "[",
                ErrorType::UnescapedOpenSquare,
                WarningType::EscapedOpenSquare,
            ),
            (
                "]",
                ErrorType::UnescapedCloseSquare,
                WarningType::EscapedCloseSquare,
            ),
        ] {
            // If there are unescaped special chars, return the error now. Pass
            // in 0 as the starting index because that way we don't have to
            // massage the character position of the error or warnings
            let indices = check_name(context.subcontext(ContextPosition::new(1,1)..=ContextPosition::new(1, name_end_pos)), c, e);
            if indices.is_err() {
                errors.push(indices.err().unwrap());
            } else {
                let indices = indices.ok().unwrap();

                // For any warning locations returned, add them to the warning list
                for idx in indices {
                    let warning = Warning::new(w.clone()).with_column(idx + 1);
                    #[cfg(not(feature = "issue-context"))]
                    {
                        warnings.push(warning);
                    }
                    #[cfg(feature = "issue-context")]
                    {
                        warnings.push(warning.with_context_len(2));
                    }
                }
            }
        }

        let name = if name_end_pos > 2 {
            input[2..name_end_pos].trim().replace("\\", "")
        } else {
            String::default()
        };
        if name.is_empty() {
            let error = Error::new(ErrorType::EmptyName, context.subcontext(ContextPosition::new(1,3)..)).with_column(3);
            #[cfg(not(feature = "issue-context"))]
            {
                errors.push(error);
            }
            #[cfg(feature = "issue-context")]
            {
                errors.push(error.with_context_len(1));
            }
        }

        if errors.is_empty() {
            Output::new(Ok(PassageHeader {
                name,
                tags,
                metadata,
                position: Position::default(),
            }))
            .with_warnings(warnings)
        } else {
            Output::new(Err(errors))
        }
    }
}

/// Given metadata in `meta_str`, parses out the metadata object, or returns a
/// warning if the metadata can't be parsed
fn parse_metadata(meta_str: &str) -> Result<serde_json::Map<String, serde_json::Value>, Warning> {
    let res = serde_json::from_str(meta_str);
    if res.is_ok() {
        use serde_json::Value;
        let tmp_meta: Value = res.ok().unwrap();
        if let Value::Object(map) = tmp_meta {
            Ok(map)
        } else {
            // shouldn't be possible?
            panic!("found a metadata object but it isn't an object?");
        }
    } else {
        let err = res.err().unwrap();
        let col = err.column();
        // Get the error part of error string generated by serde
        let err_string = format!("{}", err).split(" at ").next().unwrap().to_string();
        let warning = Warning::new(WarningType::JsonError(err_string)).with_column(col);
        #[cfg(not(feature = "issue-context"))]
        {
            Err(warning)
        }
        #[cfg(feature = "issue-context")]
        {
            Err(warning.with_context_len(meta_str.len() - col + 1))
        }
    }
}

impl Positional for PassageHeader {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}

/// Finds the last unescaped string `s` in the input string `input`
fn find_last_unescaped(input: &str, s: &str) -> Option<usize> {
    // Check for last 's'
    input.rfind(s).and_then(|pos| {
        let escaped_str = format!("\\{}", s);
        // Find last escaped 's' or use input length
        let escaped_pos = input.rfind(&escaped_str).unwrap_or_else(|| input.len());

        // If the position of the escaped and unescaped locations don't match
        // then we found an unescaped 's'
        if pos != (escaped_pos + 1) {
            Some(pos)
        } else {
            None
        }
    })
}

/// Finds all unescaped occurrences of the string `s` in input string `input`
fn find_all_unescaped(input: &str, s: &str) -> Vec<usize> {
    let esc_s = format!("\\{}", s);
    let escaped: Vec<usize> = input.match_indices(&esc_s).map(|(i, _)| i + 1).collect();
    let unescaped: Vec<usize> = input
        .match_indices(s)
        .filter(|(i, _)| !escaped.contains(i))
        .map(|(i, _)| i)
        .collect();

    unescaped
}

/// Given a header string, tries to guess what the best range is representing
/// the metadata within the header, if present. Returns `None` if no metadata is
/// found. If it's found, it returns the range
///
/// Code-chan... ganbarre
fn guess_metadata_range(input: &str) -> Option<Range<usize>> {
    let opens = find_all_unescaped(input, "{");
    let closes = find_all_unescaped(input, "}");

    if opens.is_empty() {
        None
    } else if closes.is_empty() {
        Some(opens[opens.len() - 1]..input.len())
    } else if opens.len() > closes.len() {
        let diff = opens.len() - closes.len();
        Some(opens[diff]..(closes[closes.len() - 1] + 1))
    } else {
        Some(opens[0]..(closes[closes.len() - 1] + 1))
    }
}

/// Checks the name of a passage (`input`) for validity. If the name contains
/// any of the unescaped special character (`str`), return the error `error`. If
/// the name contains any instances of that character but escaped, return a list
/// of locations in the name where the escaped character is found so that
/// warnings can be generated
fn check_name<'a>(context: FullContext<'a>, unescaped_str: &str, error: ErrorType) -> Result<Vec<usize>, Error<'a>> {
    let escaped_str = format!("\\{}", unescaped_str);
    let input = context.get_contents();

    let escaped: Vec<usize> = input.match_indices(&escaped_str).map(|(i, _)| i).collect();
    let unescaped: Vec<usize> = input
        .match_indices(unescaped_str)
        .map(|(i, _)| i)
        .filter(|i| *i == 0 || !escaped.contains(&(i - 1)))
        .collect();

    if unescaped.is_empty() {
        Ok(escaped)
    } else {
        let err_range = ContextPosition::new(1, unescaped[0] + 1)..=ContextPosition::new(1, unescaped[0]+1);
        let error = Error::new(error, context.subcontext(err_range)).with_column(unescaped[0] + 1);
        #[cfg(not(feature = "issue-context"))]
        {
            Err(error)
        }
        #[cfg(feature = "issue-context")]
        {
            Err(error.with_context_len(1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sigil() {
        let context = FullContext::from(None, "An overgrown path".to_string());
        let expected = context.subcontext(..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::MissingSigil, expected).with_column(1);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(1)
            }
        });
    }

    #[test]
    fn leading_whitespace() {
        let context = FullContext::from(None, " :: An overgrown path".to_string());
        let expected = context.subcontext(..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::LeadingWhitespace, expected).with_column(1);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(1)
            }
        });
    }

    #[test]
    fn empty_name() {
        let context = FullContext::from(None, ":: [ tag1 tag2 ]".to_string());
        let expected = context.subcontext(ContextPosition::new(1, 3)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::EmptyName, expected).with_column(3);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(1)
            }
        });

        let context = FullContext::from(None, ":: \t".to_string());
        let expected = context.subcontext(ContextPosition::new(1, 3)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::EmptyName, expected).with_column(3);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(1)
            }
        });
    }

    #[test]
    fn metadata_before_tags() {
        let context = FullContext::from(
            None,
            ":: An overgrown path { \"size\": \"5,5\" } [ tag ]".to_string(),
        );
        let expected = context.subcontext(ContextPosition::new(1, 22)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::MetadataBeforeTags, expected).with_column(22);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(17)
            }
        });
    }

    #[test]
    fn unescaped_chars() {
        for (c, e) in vec![
            ("{", ErrorType::UnescapedOpenCurly),
            ("}", ErrorType::UnescapedCloseCurly),
            ("[", ErrorType::UnescapedOpenSquare),
            ("]", ErrorType::UnescapedCloseSquare),
        ] {
            let context = FullContext::from(
                None,
                format!(":: {}An overgrown path [tag] {{ \"size\": \"5,5\" }}", c),
            );
            let sub = context.subcontext(..); // copy context, essentially
            
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);
            let errors = res.err().unwrap().errors;
            assert!(errors.iter().any(|a| {
                let sub = sub.subcontext(ContextPosition::new(1, 4)..=ContextPosition::new(1, 4));
                let expected = Error::new(e.clone(), sub).with_column(4);
                #[cfg(not(feature = "issue-context"))]
                {
                    *a == expected
                }
                #[cfg(feature = "issue-context")]
                {
                    *a == expected.with_context_len(1)
                }
            }));

            let input = format!(
                ":: {}\\{}An overgrown path [tag] {{ \"size\": \"5,5\" }}",
                c, c
            );
            let context = FullContext::from(None, input.clone());
            let sub = context.subcontext(..);
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);
            assert!(res.err().unwrap().errors.iter().any(|a| {
                let sub = sub.subcontext(ContextPosition::new(1,4)..=ContextPosition::new(1,4));
                let expected = Error::new(e.clone(), sub).with_column(4);
                #[cfg(not(feature = "issue-context"))]
                {
                    *a == expected
                }
                #[cfg(feature = "issue-context")]
                {
                    *a == expected.with_context_len(1)
                }
            }));
            let input = format!(
                ":: \\{}{}An overgrown path [tag] {{ \"size\": \"5,5\" }}",
                c, c
            );
            let context = FullContext::from(None, input.clone());
            let sub = context.subcontext(..);
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);            
            assert!(res.err().unwrap().errors.iter().any(|a| {
                let sub = sub.subcontext(ContextPosition::new(1,6)..=ContextPosition::new(1,6));
                let expected = Error::new(e.clone(), sub).with_column(6);
                #[cfg(not(feature = "issue-context"))]
                {
                    *a == expected
                }
                #[cfg(feature = "issue-context")]
                {
                    *a == expected.with_context_len(1)
                }
            }));
        }
    }

    #[test]
    fn unclosed_tags() {
        let context = FullContext::from(None, ":: An overgrown path [ tag1 tag2".to_string());
        let expected = context.subcontext(ContextPosition::new(1, 22)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorType::UnclosedTagBlock, expected).with_column(22);
            #[cfg(not(feature = "issue-context"))]
            {
                error
            }
            #[cfg(feature = "issue-context")]
            {
                error.with_context_len(11)
            }
        });
    }

    #[test]
    fn unclosed_metadata() {
        let context =
            FullContext::from(None, ":: An overgrown path { \"foo\": \"bar\"".to_string());
        let out = PassageHeader::parse(context);
        let (res, warnings) = out.take();
        assert_eq!(res.is_err(), false);
        assert!(
            if let WarningType::JsonError(_) = warnings[0].warning_type {
                true
            } else {
                false
            }
        )
    }

    #[test]
    fn tags() {
        let context = FullContext::from(
            None,
            ":: An overgrown path [tag1 tag2 tag3   tag4   ]".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.tags.len(), 4);
        assert_eq!(ph.tags, vec!["tag1", "tag2", "tag3", "tag4"]);
        assert_eq!(ph.has_tag("tag1"), true);
        assert_eq!(ph.has_tag("tag5"), false);

        let context = FullContext::from(None, ":: An overgrown path []".to_string());
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.tags.len(), 0);
        assert_eq!(ph.has_tag("tag1"), false);

        let context = FullContext::from(
            None,
            ":: An overgrown path [              \t          ]".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.tags.len(), 0);
    }

    #[test]
    fn metadata() {
        let context = FullContext::from(None, ":: Title {\"foo\":\"bar\"}".to_string());
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "100,100");
        assert_eq!(meta["position"], "10,10");
        assert_eq!(meta["foo"], "bar");

        let context = FullContext::from(None, ":: Title {\"size\":\"23,23\"}".to_string());
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "23,23");
        assert_eq!(meta["position"], "10,10");

        let context = FullContext::from(None, ":: Title { \"position\":\"5,5\" }".to_string());
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "100,100");
        assert_eq!(meta["position"], "5,5");

        let context = FullContext::from(
            None,
            ":: Title {\"size\":\"23,23\", \"position\":\"5,5\"}".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "23,23");
        assert_eq!(meta["position"], "5,5");
    }

    #[test]
    fn multilevel_metadata() {
        let context = FullContext::from(
            None,
            ":: Title {\"size\": \"23,23\", \"foo\": { \"bar\": 5 } }".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "23,23");
        assert_eq!(meta["position"], "10,10");
        assert_eq!(meta["foo"]["bar"], 5);
    }

    #[test]
    fn malformed_metadata() {
        let context = FullContext::from(None, ":: Title {\"size\":\"23, }".to_string());
        let out = PassageHeader::parse(context);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "100,100");
        assert_eq!(meta["position"], "10,10");

        assert_eq!(warnings.len(), 1);
        let expected = if let WarningType::JsonError(_) = warnings[0].warning_type {
            true
        } else {
            false
        };
        assert_eq!(expected, true);
    }

    #[test]
    fn escaped_chars() {
        let context = FullContext::from(None, ":: An over\\[grown\\} pa\\th[ tag ]".to_string());
        let out = PassageHeader::parse(context);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.name, "An over[grown} path");
        assert_eq!(ph.tags.len(), 1);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[1].warning_type, WarningType::EscapedOpenSquare);
        assert_eq!(warnings[0].warning_type, WarningType::EscapedCloseCurly);

        let context = FullContext::from(None, ":: An over\\{grown\\] pa\\th[ tag ]".to_string());
        let out = PassageHeader::parse(context);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.name, "An over{grown] path");
        assert_eq!(ph.tags.len(), 1);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].warning_type, WarningType::EscapedOpenCurly);
        assert_eq!(warnings[1].warning_type, WarningType::EscapedCloseSquare);
    }

    #[test]
    fn tags_and_metadata() {
        let context = FullContext::from(
            None,
            ":: An overgrown path [ tag ] { \"size\": \"5,5\" }".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.name, "An overgrown path");
        assert_eq!(ph.tags.len(), 1);
        assert_eq!(ph.tags, vec!["tag"]);
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "5,5");
        assert_eq!(meta["position"], "10,10");
    }

    #[test]
    fn metadata_with_array() {
        let context = FullContext::from(
            None,
            ":: An overgrown path { \"size\": \"5,5\", \"foo\":[2,3] }".to_string(),
        );
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.name, "An overgrown path");
        let meta = &ph.metadata;
        assert_eq!(meta["size"], "5,5");
        assert_eq!(meta["position"], "10,10");
        assert_eq!(ph.tags.len(), 0);
    }

    #[test]
    fn empty_tags() {
        let context = FullContext::from(None, ":: An overgrown path []".to_string());
        let out = PassageHeader::parse(context);
        assert_eq!(out.has_warnings(), false);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.tags.len(), 0);
    }
}
