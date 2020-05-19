#[cfg(feature = "issue-context")]
use crate::Contextual;
use crate::FullContext;
use crate::Position;
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
#[derive(Debug, Eq, PartialEq)]
pub struct Warning {
    /// The warning type
    pub warning_type: WarningType,

    pub context: Option<FullContext>,

    /// The location of the warning
    pub position: Position,

    /// The location referenced by this warning
    pub referent: Option<FullContext>,

    /// Line of context for Warning
    #[cfg(feature = "issue-context")]
    pub context_len: Option<usize>,
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
    pub fn new<T: Into<Option<FullContext>>>(warning_type: WarningType, context: T) -> Self {
        Warning {
            warning_type,
            context: context.into(),
            position: Position::StoryLevel,
            referent: None,
            #[cfg(feature = "issue-context")]
            context_len: None,
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
    pub fn get_referent(&self) -> Option<&FullContext> {
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
    pub fn set_referent(&mut self, referent: FullContext) {
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
    pub fn with_referent(mut self, referent: FullContext) -> Self {
        self.set_referent(referent);
        self
    }
}

#[cfg(feature = "warning-names")]
impl Warning {
    /// Gets a string representation of a `Warning`'s `WarningType` variant name
    ///
    /// Enabled with "warning-names" feature
    pub fn get_name(&self) -> &str {
        self.warning_type.get_name()
    }
}

#[cfg(feature = "issue-context")]
impl Contextual for Warning {
    fn get_context_len(&self) -> &Option<usize> {
        &self.context_len
    }

    fn mut_context_len(&mut self) -> &mut Option<usize> {
        &mut self.context_len
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cause = if self.has_referent() {
            let p: crate::PartialContext = self.get_referent().unwrap().clone().into();
            format!(", caused by: {}", p)
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
        let context = FullContext::from(None, "[[".to_string());
        let mut warning = Warning::new(WarningType::UnclosedLink, context);
        assert!(!warning.has_referent());
        assert!(warning.get_referent().is_none());

        let ref_context = FullContext::from(None, "foo bar".to_string());
        warning.set_referent(ref_context.clone());
        assert!(warning.has_referent());
        assert_eq!(warning.get_referent(), Some(&ref_context));
    }

    #[test]
    fn unchanged_referent() {
        let context = FullContext::from(None, "[[".to_string());
        let ref_context = FullContext::from(None, "foo bar".to_string());
        let warning = Warning::new(WarningType::UnclosedLink, context)
            .with_referent(ref_context.clone());
        // Prove changing the Warning's Position doesn't change the referent
        assert_eq!(warning.get_referent(), Some(&ref_context));
    }

    #[test]
    #[cfg(feature = "warning-names")]
    fn test_name() {
        let context = FullContext::from(None, "[[".to_string());
        let warning = Warning::new(WarningType::UnclosedLink, context);
        assert_eq!(warning.get_name(), "UnclosedLink");
    }
}
