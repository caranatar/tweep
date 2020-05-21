use crate::ErrorList;
use crate::FullContext;
use crate::Output;
use crate::Position;
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
///    TwineLink { target: "link".to_string(), position: Position::RowColumn(1, 43), #[cfg(feature = "full-context")] context_len: 8 },
/// #   TwineLink { target: "Another passage".to_string(), position: Position::RowColumn(2, 10), #[cfg(feature = "full-context")] context_len: 31 }]);
/// ```
///
/// [`Position`]: enum.Position.html
/// [`UnclosedLink`]: enum.WarningType.html#variant.UnclosedLink
/// [`WhitespaceInLink`]: enum.WarningType.html#variant.WhitespaceInLink
#[derive(Debug)]
pub struct TwineContent {
    /// The content of the passage
    pub content: String,

    /// The pid (Passage ID) of the passage
    pub pid: usize,

    /// A list of parsed links in this content
    links: Vec<TwineLink>,
}

impl TwineContent {
    /// Gets a [`Vec`] of all the links contained within this content
    ///
    /// [`Vec`]: std::Vec
    pub fn get_links(&self) -> &Vec<TwineLink> {
        &self.links
    }

    /// Parses a `TwineContent` out of the given context
    pub fn parse(context: FullContext) -> Output<Result<Self, ErrorList>> {
        let mut links = Vec::new();
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
                            Warning::new(
                                WarningType::UnclosedLink,
                                context.subcontext(
                                    Position::rel(row + 1, start + 1)
                                        ..=Position::rel(row + 1, line.len()),
                                ),
                            )
                        });
                        break;
                    }
                };
                let link_context = context.subcontext(
                    Position::rel(row + 1, start + 1)..=Position::rel(row + 1, end + 2),
                );
                let link_content = &line[start + 2..end];
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
                        Warning::new(WarningType::WhitespaceInLink, link_context.clone())
                    });
                }

                links.push(TwineLink {
                    target: linked_passage.to_string(),
                    context: link_context.clone(),
                });

                start = end;
            }
        }

        let mut content = context.get_contents().to_string();
        content.push('\n');
        Output::new(Ok(TwineContent {
            content,
            links,
            pid: 1,
        }))
        .with_warnings(warnings)
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
        let context = FullContext::from(None, input);
        let out = TwineContent::parse(context.clone());
        let (res, warnings) = out.take();
        assert_eq!(warnings.is_empty(), true);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        let expected_targets = vec!["foo", "bar", "baz", "qux"];
        let expected_lens = vec![7, 17, 18, 19];
        let expected_links: Vec<TwineLink> = (1 as usize..5)
            .map(|row| {
                TwineLink::new(
                    expected_targets[row - 1].to_string(),
                    context.subcontext(
                        Position::rel(row, 1)..=Position::rel(row, expected_lens[row - 1]),
                    ),
                )
            })
            .collect();
        assert_eq!(content.get_links(), &expected_links);
    }

    #[test]
    fn unclosed_link() {
        let context = FullContext::from(None, "blah [[unclosed\nlink]] blah blah\n\n".to_string());
        let out = TwineContent::parse(context.clone());
        let (res, warnings) = out.take();
        let expected = Warning::new(
            WarningType::UnclosedLink,
            context.subcontext(Position::rel(1, 6)..=Position::rel(1, 15)),
        );
        assert_eq!(warnings, vec![expected]);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        assert!(content.links.is_empty());
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
        let context = FullContext::from(None, input);
        let out = TwineContent::parse(context.clone());
        let (res, warnings) = out.take();
        let expected_lens = vec![8, 8, 13, 13, 15, 15, 16, 17];
        let expected_warnings: Vec<Warning> = (1 as usize..9)
            .map(|row| {
                Warning::new(
                    WarningType::WhitespaceInLink,
                    context.subcontext(
                        Position::rel(row, 1)..=Position::rel(row, expected_lens[row - 1]),
                    ),
                )
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
                    context.subcontext(
                        Position::rel(row, 1)..=Position::rel(row, expected_lens[row - 1]),
                    ),
                )
            })
            .collect();
        assert_eq!(content.get_links(), &expected_links);
    }
}
