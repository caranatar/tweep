use crate::ErrorType;
use crate::FullContext;

/// An error with an owned [`ErrorType`] and [`Position`]
///
/// [`ErrorType`]: enum.ErrorType.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The type of error
    pub error_type: ErrorType,

    /// The context of the error
    pub context: Option<FullContext>,
}

impl Error {
    /// Creates a new `Error` with the given [`ErrorType`] and a default
    /// [`Position`]
    ///
    /// # Examples
    /// ```
    /// use tweep::{Error, ErrorType};
    /// # use tweep::{FullContext, Position, Positional};
    /// # let context = FullContext::from(None, "::".to_string());
    /// let error = Error::new(ErrorType::EmptyName, context);
    /// # assert_eq!(error.error_type, ErrorType::EmptyName);
    /// # assert_eq!(error.get_position(), &Position::default());
    /// ```
    ///
    /// [`ErrorType`]: enum.ErrorType.html
    /// [`Position`]: enum.Position.html
    pub fn new<T: Into<Option<FullContext>>>(error_type: ErrorType, context: T) -> Self {
        Error {
            error_type,
            context: context.into(),
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
        write!(f, "{} at {:?}", self.error_type, self.context)
    }
}
