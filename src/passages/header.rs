use crate::issues::*;
use crate::FullContext;
use crate::Output;
use crate::Position;

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
/// [`LeadingWhitespace`]: enum.ErrorKind.html#variant.LeadingWhitespace
/// [`MissingSigil`]: enum.ErrorKind.html#variant.MissingSigil
/// [`MetadataBeforeTags`]: enum.ErrorKind.html#variant.MetadataBeforeTags
/// [`UnclosedTagBlock`]: enum.ErrorKind.html#variant.UnclosedTagBlock
/// [`UnescapedOpenCurly`]: enum.ErrorKind.html#variant.UnescapedOpenCurly
/// [`UnescapedCloseCurly`]: enum.ErrorKind.html#variant.UnescapedCloseCurly
/// [`UnescapedOpenSquare`]: enum.ErrorKind.html#variant.UnescapedOpenSquare
/// [`UnescapedCloseSquare`]: enum.ErrorKind.html#variant.UnescapedCloseSquare
/// [`EmptyName`]: enum.ErrorKind.html#variant.EmptyName
/// [`JsonError`]: enum.WarningKind.html#variant.JsonError
/// [`EscapedOpenCurly`]: enum.WarningKind.html#variant.EscapedOpenCurly
/// [`EscapedCloseCurly`]: enum.WarningKind.html#variant.EscapedCloseCurly
/// [`EscapedOpenSquare`]: enum.WarningKind.html#variant.EscapedOpenSquare
/// [`EscapedCloseSquare`]: enum.WarningKind.html#variant.EscapedCloseSquare
#[derive(Debug)]
pub struct PassageHeader {
    /// The name of the header. This can be a Twine passage name or a special name
    pub name: String,

    /// The list of comma separated tags
    pub tags: Vec<String>,

    /// A json object containing metadata for the passage
    pub metadata: serde_json::Map<String, serde_json::Value>,
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
                    Error::new(ErrorKind::LeadingWhitespace, Some(context.clone()))
                } else {
                    Error::new(ErrorKind::MissingSigil, Some(context.clone()))
                }
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
                let error = Error::new(ErrorKind::MetadataBeforeTags, Some(context.subcontext(Position::rel(1, pos+1)..)));
                errors.push(error);
            }

            let meta_context = context.subcontext(Position::rel(1, range.start)..=Position::rel(1, range.end));
            let res = parse_metadata(meta_context);
            if res.is_ok() {
                for (k, v) in res.ok().unwrap().iter() {
                    metadata.insert(k.to_string(), v.clone());
                }
            } else {
                warnings.push(res.err().unwrap());
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
                let error = Error::new(ErrorKind::UnclosedTagBlock, Some(context.subcontext(Position::rel(1, pos+1)..)));
                errors.push(error);
            }

            name_end_pos = std::cmp::min(name_end_pos, pos);
        }

        // Check for unescaped special characters in the name portion. This also
        // produces a list of warning locations for escaped chars in the name
        for (c, e, w) in vec![
            (
                "{",
                ErrorKind::UnescapedOpenCurly,
                WarningKind::EscapedOpenCurly,
            ),
            (
                "}",
                ErrorKind::UnescapedCloseCurly,
                WarningKind::EscapedCloseCurly,
            ),
            (
                "[",
                ErrorKind::UnescapedOpenSquare,
                WarningKind::EscapedOpenSquare,
            ),
            (
                "]",
                ErrorKind::UnescapedCloseSquare,
                WarningKind::EscapedCloseSquare,
            ),
        ] {
            // If there are unescaped special chars, return the error now. Pass
            // in 0 as the starting index because that way we don't have to
            // massage the character position of the error or warnings
            let indices = check_name(context.subcontext(..=Position::rel(1, name_end_pos)), c, e);
            if indices.is_err() {
                errors.push(indices.err().unwrap());
            } else {
                let indices = indices.ok().unwrap();

                // For any warning locations returned, add them to the warning list
                for idx in indices {
                    let warning = Warning::new(w.clone(), context.subcontext(Position::rel(1, idx + 1)..=Position::rel(1, idx+2)));
                    warnings.push(warning);
                }
            }
        }

        let name = if name_end_pos > 2 {
            input[2..name_end_pos].trim().replace("\\", "")
        } else {
            String::default()
        };
        if name.is_empty() {
            let error = Error::new(ErrorKind::EmptyName, Some(context.subcontext(Position::rel(1,3)..)));
            errors.push(error);
        }

        if errors.is_empty() {
            Output::new(Ok(PassageHeader {
                name,
                tags,
                metadata,
            }))
            .with_warnings(warnings)
        } else {
            Output::new(Err(errors))
        }
    }
}

/// Given metadata in `meta_str`, parses out the metadata object, or returns a
/// warning if the metadata can't be parsed
fn parse_metadata(context: FullContext) -> Result<serde_json::Map<String, serde_json::Value>, Warning> {
    let meta_str = context.get_contents();
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
        let warning = Warning::new(WarningKind::JsonError(err_string), Some(context.subcontext(Position::rel(1, col)..)));
        Err(warning)
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
fn check_name(context: FullContext, unescaped_str: &str, error: ErrorKind) -> Result<Vec<usize>, Error> {
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
        let err_range = Position::rel(1, unescaped[0] + 1)..=Position::rel(1, unescaped[0]+1);
        let error = Error::new(error, context.subcontext(err_range));
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sigil() {
        let context = FullContext::from(None, "An overgrown path".to_string());
        let expected = context.clone();
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::MissingSigil, Some(expected));
            error
        });
    }

    #[test]
    fn leading_whitespace() {
        let context = FullContext::from(None, " :: An overgrown path".to_string());
        let expected = context.clone();
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::LeadingWhitespace, Some(expected));
            error
        });
    }

    #[test]
    fn empty_name() {
        let context = FullContext::from(None, ":: [ tag1 tag2 ]".to_string());
        let expected = context.subcontext(Position::rel(1, 3)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::EmptyName, Some(expected));
            error
        });

        let context = FullContext::from(None, ":: \t".to_string());
        let expected = context.subcontext(Position::rel(1, 3)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::EmptyName, Some(expected));
            error
        });
    }

    #[test]
    fn metadata_before_tags() {
        let context = FullContext::from(
            None,
            ":: An overgrown path { \"size\": \"5,5\" } [ tag ]".to_string(),
        );
        let expected = context.subcontext(Position::rel(1, 22)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::MetadataBeforeTags, Some(expected));
            error
        });
    }

    #[test]
    fn unescaped_chars() {
        for (c, e) in vec![
            ("{", ErrorKind::UnescapedOpenCurly),
            ("}", ErrorKind::UnescapedCloseCurly),
            ("[", ErrorKind::UnescapedOpenSquare),
            ("]", ErrorKind::UnescapedCloseSquare),
        ] {
            let context = FullContext::from(
                None,
                format!(":: {}An overgrown path [tag] {{ \"size\": \"5,5\" }}", c),
            );
            let sub = context.clone(); // copy context, essentially
            
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);
            let errors = res.err().unwrap().errors;
            assert!(errors.iter().any(|a| {
                let sub = sub.subcontext(Position::rel(1, 4)..=Position::rel(1, 4));
                let expected = Error::new(e.clone(), sub);
                *a == expected
            }));

            let input = format!(
                ":: {}\\{}An overgrown path [tag] {{ \"size\": \"5,5\" }}",
                c, c
            );
            let context = FullContext::from(None, input.clone());
            let sub = context.clone();
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);
            assert!(res.err().unwrap().errors.iter().any(|a| {
                let sub = sub.subcontext(Position::rel(1,4)..=Position::rel(1,4));
                let expected = Error::new(e.clone(), sub);
                *a == expected
            }));
            let input = format!(
                ":: \\{}{}An overgrown path [tag] {{ \"size\": \"5,5\" }}",
                c, c
            );
            let context = FullContext::from(None, input.clone());
            let sub = context.clone();
            let out = PassageHeader::parse(context);
            let (res, _) = out.take();
            assert_eq!(res.is_err(), true);            
            assert!(res.err().unwrap().errors.iter().any(|a| {
                let sub = sub.subcontext(Position::rel(1,6)..=Position::rel(1,6));
                let expected = Error::new(e.clone(), sub);
                *a == expected
            }));
        }
    }

    #[test]
    fn unclosed_tags() {
        let context = FullContext::from(None, ":: An overgrown path [ tag1 tag2".to_string());
        let expected = context.subcontext(Position::rel(1, 22)..);
        let out = PassageHeader::parse(context);
        let (res, _) = out.take();
        assert_eq!(res.is_err(), true);
        assert_eq!(res.err().unwrap().errors[0], {
            let error = Error::new(ErrorKind::UnclosedTagBlock, Some(expected));
            error
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
            if let WarningKind::JsonError(_) = warnings[0].kind {
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
        let expected = if let WarningKind::JsonError(_) = warnings[0].kind {
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
        assert_eq!(warnings[1].kind, WarningKind::EscapedOpenSquare);
        assert_eq!(warnings[0].kind, WarningKind::EscapedCloseCurly);

        let context = FullContext::from(None, ":: An over\\{grown\\] pa\\th[ tag ]".to_string());
        let out = PassageHeader::parse(context);
        let (res, warnings) = out.take();
        assert_eq!(res.is_ok(), true);
        let ph = res.ok().unwrap();
        assert_eq!(ph.name, "An over{grown] path");
        assert_eq!(ph.tags.len(), 1);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].kind, WarningKind::EscapedOpenCurly);
        assert_eq!(warnings[1].kind, WarningKind::EscapedCloseSquare);
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
