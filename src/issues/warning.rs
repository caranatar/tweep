use crate::Position;
use crate::Positional;
use crate::WarningType;

/// A warning with a [`WarningType`], [`Position`], and optionally a reference
/// to another [`Position`]
///
/// # Examples
/// ```
/// use tweep::{Position, Warning, WarningType};
/// let warning = Warning::new(WarningType::DuplicateStoryTitle)
///     .with_referent(Position::RowColumn(5, 0));
/// # assert!(warning.has_referent());
/// ```
///
/// [`WarningType`]: enum.WarningType.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Warning {
    /// The warning type
    pub warning_type: WarningType,

    /// The location of the warning
    pub position: Position,

    /// The location referenced by this warning
    pub referent: Option<Position>,
}

impl Warning {
    /// Creates a new `Warning` with a default `Position` and no referent
    ///
    /// # Examples
    /// ```
    /// use tweep::{Warning, WarningType};
    /// let warning = Warning::new(WarningType::MissingStartPassage);
    /// # assert!(!warning.has_referent());
    /// # assert_eq!(warning.position, tweep::Position::StoryLevel);
    /// ```
    pub fn new(warning_type: WarningType) -> Self {
        Warning {
            warning_type,
            position: Position::StoryLevel,
            referent: None,
        }
    }

    /// Returns `true` if this `Warning` has a referent
    ///
    /// # Examples
    /// ```
    /// use tweep::{Position, Warning, WarningType};
    /// let mut warning = Warning::new(WarningType::UnclosedLink);
    /// assert!(!warning.has_referent());
    /// warning.set_referent(Position::RowColumn(23, 5));
    /// assert!(warning.has_referent());
    /// ```
    pub fn has_referent(&self) -> bool {
        self.referent.is_some()
    }

    /// Gets the referent if one exists
    ///
    /// # Examples
    /// ```
    /// use tweep::{Position, Warning, WarningType};
    /// let warning = Warning::new(WarningType::DuplicateStoryTitle)
    ///     .with_referent(Position::RowColumn(5, 0));
    /// assert_eq!(warning.get_referent(), Some(&Position::RowColumn(5, 0)));
    /// ```
    pub fn get_referent(&self) -> Option<&Position> {
        self.referent.as_ref()
    }

    /// Sets the referent to the given `Position`
    ///
    /// # Examples
    /// ```
    /// use tweep::{Position, Warning, WarningType};
    /// let mut warning = Warning::new(WarningType::UnclosedLink);
    /// assert!(!warning.has_referent());
    /// warning.set_referent(Position::RowColumn(23, 5));
    /// assert!(warning.has_referent());
    /// assert_eq!(warning.get_referent(), Some(&Position::RowColumn(23, 5)));
    /// ```
    pub fn set_referent(&mut self, referent: Position) {
        self.referent = Some(referent);
    }

    /// Moves the object, sets the referent to the given `Position`, and returns
    /// the modified object
    ///
    /// # Examples
    /// ```
    /// use tweep::{Position, Warning, WarningType};
    /// let warning = Warning::new(WarningType::DuplicateStoryTitle)
    ///     .with_referent(Position::RowColumn(5, 0));
    /// # assert_eq!(warning.get_referent(), Some(&Position::RowColumn(5, 0)));
    /// ```
    pub fn with_referent(mut self, referent: Position) -> Self {
        self.set_referent(referent);
        self
    }
}

impl Positional for Warning {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cause = if self.has_referent() {
            format!(", caused by: {}", self.get_referent().unwrap())
        } else {
            String::new()
        };
        write!(f, "{} at {}{}", self.warning_type, self.position, cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental() {
        let mut warning = Warning::new(WarningType::UnclosedLink);
        assert!(!warning.has_referent());
        assert!(warning.get_referent().is_none());
        assert_eq!(warning.get_position(), &Position::StoryLevel);

        warning.set_referent(Position::RowColumn(5, 23));
        assert!(warning.has_referent());
        assert_eq!(warning.get_referent(), Some(&Position::RowColumn(5, 23)));
        assert_eq!(warning.get_position(), &Position::StoryLevel);
    }

    #[test]
    fn unchanged_referent() {
        let mut warning =
            Warning::new(WarningType::UnclosedLink).with_referent(Position::RowColumn(23, 5));
        // Prove changing the Warning's Position doesn't change the referent
        warning.set_column(10);
        warning.set_row(20);
        assert_eq!(warning.get_referent(), Some(&Position::RowColumn(23, 5)));
        assert_eq!(warning.get_position(), &Position::RowColumn(20, 10));
    }
}
