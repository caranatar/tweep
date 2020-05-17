#[cfg(feature = "issue-context")]
use crate::Contextual;
use crate::ErrorList;
use crate::FullContext;
use crate::InternalTwineLink;
use crate::Output;
use crate::Position;
use crate::Positional;
use crate::TwineLink;
use crate::Warning;
use crate::WarningType;

/// The contents of a Twine passage.
///
/// Contains the content of the passage along with the [`Position`]. The
/// `get_links` method can be used to retrieve all Twine links that were parsed
/// out of the passage.
///
/// # Parse Errors
/// None
///
/// # Parse Warnings
/// * [`UnclosedLink`] - An unclosed Twine link such as `[[Passage Name``
/// * [`WhitespaceInLink`] - Errant whitespace in link such as `[[Display Text-> Passage Name]]`
///
/// # Notes
/// Currently, the supported formats for links are the following:
/// ```tweev3
/// [[Passge Name]]
/// [[Display Text|Passage Name]]
/// [[Display Text->Passage Name]]
/// [[Passage Name<-Display Text]]
/// ```
///
/// # Examples
/// ```
/// use tweep::{FullContext, Position, TwineContent, TwineLink};
/// let input = r#"This is a Twine content passage. It has a [[link]]
///And some [[other link->Another passage]]
///"#.to_string();
/// let out = TwineContent::parse(FullContext::from(None, input));
/// # assert!(!out.has_warnings());
/// # assert_eq!(out.get_output().as_ref().ok().unwrap().get_links(), vec![
///    TwineLink { target: "link".to_string(), position: Position::RowColumn(1, 43), #[cfg(feature = "issue-context")] context_len: 8 },
/// #   TwineLink { target: "Another passage".to_string(), position: Position::RowColumn(2, 10), #[cfg(feature = "issue-context")] context_len: 31 }]);
/// ```
///
/// [`Position`]: enum.Position.html
/// [`UnclosedLink`]: enum.WarningType.html#variant.UnclosedLink
/// [`WhitespaceInLink`]: enum.WarningType.html#variant.WhitespaceInLink
#[derive(Debug)]
pub struct TwineContent {
    /// The content of the passage
    pub content: String,

    /// The position of the passage
    pub position: Position,

    /// The pid (Passage ID) of the passage
    pub pid: usize,

    /// A list of parsed links in this content
    linked_passages: Vec<InternalTwineLink>,
}

impl TwineContent {
    /// Gets a [`Vec`] of all the links contained within this content
    ///
    /// [`Vec`]: std::Vec
    pub fn get_links(&self) -> Vec<TwineLink> {
        let mut links = Vec::new();
        for link in &self.linked_passages {
            links.push(
                TwineLink {
                    target: link.target.clone(),
                    position: self.position.clone(),
                    #[cfg(feature = "issue-context")]
                    context_len: link.context_len,
                }
                .with_offset_column(link.col_offset)
                .with_offset_row(link.row_offset),
            );
        }
        links
    }

    /// Parses a `TwineContent` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        let mut linked_passages = Vec::new();
        let mut warnings = Vec::new();
        for (row, line) in context.get_contents().split('\n').enumerate() {
            let mut start = 0;
            loop {
                start = match line[start..].find("[[") {
                    Some(x) => start + x,
                    None => break,
                };
                let end = match line[start..].find("]]") {
                    Some(x) => start + x,
                    None => {
                        warnings.push({
                            let warning = Warning::new(WarningType::UnclosedLink)
                                .with_column(start + 1)
                                .with_row(row + 1);
                            #[cfg(not(feature = "issue-context"))]
                            {
                                warning
                            }
                            #[cfg(feature = "issue-context")]
                            {
                                warning.with_context_len(line.len() - start)
                            }
                        });
                        break;
                    }
                };
                let link_content = &line[start + 2..end];
                #[cfg(feature = "issue-context")]
                let context_len = link_content.len() + 4;
                let linked_passage = if link_content.contains('|') {
                    // Link format: [[Link Text|Passage Name]]
                    let mut iter = link_content.split('|');
                    let _ = iter.next();
                    iter.next().unwrap()
                } else if link_content.contains("<-") {
                    // Link format: [[Passage Name<-Link Text]]
                    link_content.split("<-").next().unwrap()
                } else if link_content.contains("->") {
                    // Link format: [[Link Text->Passage Name]]
                    let mut iter = link_content.split("->");
                    let _ = iter.next();
                    iter.next().unwrap()
                } else {
                    // Link format: [[Passage Name]]
                    link_content
                };

                if linked_passage.starts_with(char::is_whitespace)
                    || linked_passage.ends_with(char::is_whitespace)
                {
                    warnings.push({
                        let warning = Warning::new(WarningType::WhitespaceInLink)
                            .with_column(start + 1)
                            .with_row(row + 1);
                        #[cfg(not(feature = "issue-context"))]
                        {
                            warning
                        }
                        #[cfg(feature = "issue-context")]
                        {
                            warning.with_context_len(context_len)
                        }
                    });
                }

                linked_passages.push(InternalTwineLink {
                    target: linked_passage.to_string(),
                    col_offset: start,
                    row_offset: row,
                    #[cfg(feature = "issue-context")]
                    context_len,
                });

                start = end;
            }
        }

        let mut content = context.get_contents().to_string();
        content.push('\n');
        Output::new(Ok(TwineContent {
            content,
            position: Position::default(),
            linked_passages,
            pid: 1,
        }))
        .with_warnings(warnings)
    }
}

impl Positional for TwineContent {
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
    fn a_test() {
        let input = "foo\nbar".to_string();
        let out = TwineContent::parse(FullContext::from(None, input));
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        assert_eq!(content.content, "foo\nbar\n");
    }

    #[test]
    fn links() {
        let input =
            "[[foo]]\n[[Pipe link|bar]]\n[[baz<-Left link]]\n[[Right link->qux]]\n".to_string();
        let out = TwineContent::parse(FullContext::from(None, input));
        let (res, warnings) = out.take();
        assert_eq!(warnings.is_empty(), true);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        let expected_targets = vec!["foo", "bar", "baz", "qux"];
        #[cfg(feature = "issue-context")]
        let expected_lens = vec![7, 17, 18, 19];
        let expected_links: Vec<TwineLink> = (1 as usize..5)
            .map(|row| {
                TwineLink::new(
                    expected_targets[row - 1].to_string(),
                    #[cfg(feature = "issue-context")]
                    expected_lens[row - 1],
                )
                .with_column(1)
                .with_row(row)
            })
            .collect();
        assert_eq!(content.get_links(), expected_links);
    }

    #[test]
    fn unclosed_link() {
        let input = "blah [[unclosed\nlink]] blah blah\n\n".to_string();
        let out = TwineContent::parse(FullContext::from(None, input));
        let (res, warnings) = out.take();
        let mut expected = Warning::new(WarningType::UnclosedLink);
        #[cfg(not(feature = "issue-context"))]
        {
            expected = expected.with_row(1).with_column(6);
        }
        #[cfg(feature = "issue-context")]
        {
            use crate::Contextual;
            expected = expected.with_row(1).with_column(6).with_context_len(10);
        }
        assert_eq!(warnings, vec![expected]);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        assert_eq!(content.linked_passages.is_empty(), true);
    }

    #[test]
    fn whitespace_in_link() {
        let input = r#"[[ foo]]
[[bar ]]
[[text|baz ]]
[[text| qux]]
[[quux <-text]]
[[ quuz<-text]]
[[text-> corge]]
[[text->grault ]]"#
            .to_string();
        let out = TwineContent::parse(FullContext::from(None, input));
        let (res, warnings) = out.take();
        #[cfg(feature = "issue-context")]
        let expected_lens = vec![8, 8, 13, 13, 15, 15, 16, 17];
        let expected_warnings: Vec<Warning> = (1 as usize..9)
            .map(|row| {
                let warning = Warning::new(WarningType::WhitespaceInLink)
                    .with_row(row)
                    .with_column(1);
                #[cfg(not(feature = "issue-context"))]
                {
                    warning
                }
                #[cfg(feature = "issue-context")]
                {
                    warning.with_context_len(expected_lens[row - 1])
                }
            })
            .collect();
        assert_eq!(warnings, expected_warnings);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        let expected_targets = vec![
            " foo", "bar ", "baz ", " qux", "quux ", " quuz", " corge", "grault ",
        ];
        let expected_links: Vec<TwineLink> = (1 as usize..9)
            .map(|row| {
                TwineLink::new(
                    expected_targets[row - 1].to_string(),
                    #[cfg(feature = "issue-context")]
                    expected_lens[row - 1],
                )
                .with_column(1)
                .with_row(row)
            })
            .collect();
        assert_eq!(content.get_links(), expected_links);
    }
}
