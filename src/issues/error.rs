use crate::ErrorKind;
use crate::Context;

/// An error with an owned [`ErrorKind`] and [`Position`]
///
/// [`ErrorKind`]: enum.ErrorKind.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The type of error
    pub kind: ErrorKind,

    /// The context of the error
    pub context: Option<Context>,
}

impl Error {
    /// Creates a new `Error` with the given [`ErrorKind`] and a default
    /// [`Position`]
    ///
    /// # Examples
    /// ```
    /// use tweep::{Error, ErrorKind};
    /// # use tweep::{FullContext};
    /// # let context = FullContext::from(None, "::".to_string());
    /// let error = Error::new(ErrorKind::EmptyName, Some(context));
    /// # assert_eq!(error.kind, ErrorKind::EmptyName);
    /// ```
    ///
    /// [`ErrorKind`]: enum.ErrorKind.html
    /// [`Position`]: struct.Position.html
    pub fn new<T: Into<Context>>(kind: ErrorKind, context: Option<T>) -> Self {
        Error {
            kind,
            context: context.map(|t| t.into()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.kind, self.context)
    }
}
