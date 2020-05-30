use crate::Context;
use crate::WarningKind;

/// A warning with a [`WarningKind`], [`Position`], and optionally a reference
/// to another [`Position`]
///
/// # Examples
/// ```
/// use tweep::{FullContext, Warning, WarningKind};
/// # let context = FullContext::from(None, String::new());
/// # let referent = FullContext::from(None, String::new());
/// let warning = Warning::new(WarningKind::DuplicateStoryTitle, Some(context))
///     .with_referent(referent);
/// # assert!(warning.has_referent());
/// ```
///
/// [`WarningKind`]: enum.WarningKind.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Warning {
    /// The warning type
    pub kind: WarningKind,

    /// The context of this Warning
    pub context: Option<Context>,

    /// The location referenced by this warning
    pub referent: Option<Context>,
}

impl Warning {
    /// Creates a new `Warning` with a default `Position` and no referent
    ///
    /// # Examples
    /// ```
    /// use tweep::{FullContext, Warning, WarningKind};
    /// # let context = FullContext::from(None, String::new());
    /// let warning = Warning::new(WarningKind::MissingStartPassage, Some(context));
    /// # assert!(!warning.has_referent());
    /// ```
    pub fn new<T: Into<Context>>(kind: WarningKind, context: Option<T>) -> Self {
        Warning {
            kind,
            context: context.map(|c| c.into()),
            referent: None,
        }
    }

    /// Returns `true` if this `Warning` has a referent
    ///
    /// # Examples
    /// ```
    /// use tweep::{FullContext, Warning, WarningKind};
    /// # let context = FullContext::from(None, String::new());
    /// let mut warning = Warning::new(WarningKind::UnclosedLink, Some(context));
    /// assert!(!warning.has_referent());
    /// # let referent = FullContext::from(None, String::new());
    /// warning.set_referent(referent);
    /// assert!(warning.has_referent());
    /// ```
    pub fn has_referent(&self) -> bool {
        self.referent.is_some()
    }

    /// Gets the referent if one exists
    ///
    /// # Examples
    /// ```
    /// # use tweep::{Context, FullContext, Warning, WarningKind};
    /// # let context:Context = FullContext::from(None, String::new()).into();
    /// # let referent = context.clone();
    /// let warning = Warning::new(WarningKind::DuplicateStoryTitle, Some(context))
    ///     .with_referent(referent.clone());
    /// assert_eq!(warning.get_referent(), Some(&referent));
    /// ```
    pub fn get_referent(&self) -> Option<&Context> {
        self.referent.as_ref()
    }

    /// Sets the referent to the given `Position`
    ///
    /// # Examples
    /// ```
    /// # use tweep::{Context, FullContext, Warning, WarningKind};
    /// # let context:Context = FullContext::from(None, String::new()).into();
    /// # let referent = context.clone();
    /// let mut warning = Warning::new(WarningKind::DuplicateStoryTitle, Some(context));
    /// warning.set_referent(referent.clone());
    /// assert_eq!(warning.get_referent(), Some(&referent));
    /// ```
    pub fn set_referent<T: Into<Context>>(&mut self, referent: T) {
        self.referent = Some(referent.into());
    }

    /// Moves the object, sets the referent to the given `Position`, and returns
    /// the modified object
    ///
    /// # Examples
    /// ```
    /// # use tweep::{Context, FullContext, Warning, WarningKind};
    /// # let context:Context = FullContext::from(None, String::new()).into();
    /// # let referent = context.clone();
    /// let warning = Warning::new(WarningKind::DuplicateStoryTitle, Some(context))
    ///     .with_referent(referent.clone());
    /// assert_eq!(warning.get_referent(), Some(&referent));
    /// ```
    pub fn with_referent<T: Into<Context>>(mut self, referent: T) -> Self {
        self.set_referent(referent.into());
        self
    }
}

#[cfg(feature = "issue-names")]
impl Warning {
    /// Gets a string representation of a `Warning`'s `WarningKind` variant name
    ///
    /// Enabled with "issue-names" feature
    pub fn get_name(&self) -> &str {
        self.kind.get_name()
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
        write!(f, "{} at {:?}{}", self.kind, self.context, cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FullContext;

    #[test]
    fn incremental() {
        let context: Context = FullContext::from(None, "[[".to_string()).into();
        let mut warning = Warning::new(WarningKind::UnclosedLink, Some(context));
        assert!(!warning.has_referent());
        assert!(warning.get_referent().is_none());

        let ref_context = FullContext::from(None, "foo bar".to_string());
        warning.set_referent(ref_context.clone());
        assert!(warning.has_referent());
        assert_eq!(warning.get_referent(), Some(&ref_context.into()));
    }

    #[test]
    fn unchanged_referent() {
        let context = FullContext::from(None, "[[".to_string());
        let ref_context = FullContext::from(None, "foo bar".to_string());
        let warning = Warning::new(WarningKind::UnclosedLink, Some(context))
            .with_referent(ref_context.clone());
        // Prove changing the Warning's Position doesn't change the referent
        assert_eq!(warning.get_referent(), Some(&ref_context.into()));
    }

    #[test]
    #[cfg(feature = "issue-names")]
    fn test_name() {
        let context = FullContext::from(None, "[[".to_string());
        let warning = Warning::new(WarningKind::UnclosedLink, Some(context));
        assert_eq!(warning.get_name(), "UnclosedLink");
    }
}
