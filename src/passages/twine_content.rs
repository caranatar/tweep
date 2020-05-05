use crate::ErrorList;
use crate::InternalTwineLink;
use crate::Output;
use crate::Parser;
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
/// use tweep::{Parser, Position, TwineContent, TwineLink};
/// let input:Vec<&str> = r#"This is a Twine content passage. It has a [[link]]
///And some [[other link->Another passage]]
///"#.split('\n').collect();
/// let out = TwineContent::parse(&input);
/// # assert!(!out.has_warnings());
/// assert_eq!(out.get_output().as_ref().ok().unwrap().get_links(), vec![
///    TwineLink { target: "link".to_string(), position: Position::RowColumn(1, 45) },
///    TwineLink { target: "Another passage".to_string(), position: Position::RowColumn(2, 12) }]);
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
                }
                .with_offset_column(link.col_offset)
                .with_offset_row(link.row_offset),
            );
        }
        links
    }
}

impl<'a> Parser<'a> for TwineContent {
    type Output = Output<Result<Self, ErrorList>>;
    type Input = [&'a str];

    fn parse(input: &'a Self::Input) -> Self::Output {
        let mut linked_passages = Vec::new();
        let mut warnings = Vec::new();
        for (row, line) in input.iter().enumerate() {
            let mut start = 0;
            loop {
                start = match line[start..].find("[[") {
                    Some(x) => start + x,
                    None => break,
                };
                let end = match line[start..].find("]]") {
                    Some(x) => start + x,
                    None => {
                        warnings.push(
                            Warning::new(WarningType::UnclosedLink)
                                .with_column(start+1)
                                .with_row(row+1),
                        );
                        break;
                    }
                };
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
                    warnings.push(
                        Warning::new(WarningType::WhitespaceInLink)
                            .with_column(start + 3)
                            .with_row(row + 1),
                    );
                }

                linked_passages.push(InternalTwineLink {
                    target: linked_passage.to_string(),
                    col_offset: (start + 2),
                    row_offset: row,
                });

                start = end;
            }
        }

        let mut content = input.join("\n");
        content.push('\n');
        Output::new(Ok(TwineContent {
            content,
            position: Position::default(),
            linked_passages,
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
        let x = "foo";
        let y = "bar";
        let v = vec![x, y];
        let out = TwineContent::parse(&v);
        let (res, _) = out.take();
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        assert_eq!(content.content, "foo\nbar\n");
    }

    #[test]
    fn links() {
        let input: Vec<&str> =
            "[[foo]]\n[[Pipe link|bar]]\n[[baz<-Left link]]\n[[Right link->qux]]\n"
                .split("\n")
                .collect();
        let out = TwineContent::parse(&input);
        let (res, warnings) = out.take();
        assert_eq!(warnings.is_empty(), true);
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        let expected_targets = vec!["foo", "bar", "baz", "qux"];
        let expected_links: Vec<TwineLink> = (1 as usize..5)
            .map(|row| {
                TwineLink::new(expected_targets[row-1].to_string())
                    .with_column(3)
                    .with_row(row)
            })
            .collect();
        assert_eq!(content.get_links(), expected_links);
    }

    #[test]
    fn unclosed_link() {
        let input: Vec<&str> = "blah [[unclosed\nlink]] blah blah\n\n"
            .split("\n")
            .collect();
        let out = TwineContent::parse(&input);
        let (res, warnings) = out.take();
        assert_eq!(
            warnings,
            vec![Warning::new(WarningType::UnclosedLink)
                .with_row(1)
                .with_column(6)]
        );
        assert_eq!(res.is_ok(), true);
        let content = res.ok().unwrap();
        assert_eq!(content.linked_passages.is_empty(), true);
    }

    #[test]
    fn whitespace_in_link() {
        let input = vec![
            "[[ foo]]",
            "[[bar ]]",
            "[[text|baz ]]",
            "[[text| qux]]",
            "[[quux <-text]]",
            "[[ quuz<-text]]",
            "[[text-> corge]]",
            "[[text->grault ]]",
        ];
        let out = TwineContent::parse(&input);
        let (res, warnings) = out.take();
        let expected_warnings: Vec<Warning> = (1 as usize..9)
            .map(|row| {
                Warning::new(WarningType::WhitespaceInLink)
                    .with_row(row)
                    .with_column(3)
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
                TwineLink::new(expected_targets[row-1].to_string())
                    .with_column(3)
                    .with_row(row)
            })
            .collect();
        assert_eq!(content.get_links(), expected_links);
    }
}
