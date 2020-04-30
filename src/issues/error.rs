use crate::ErrorType;
use crate::Position;
use crate::Positional;

/// An error with an owned [`ErrorType`] and [`Position`]
///
/// [`ErrorType`]: enum.ErrorType.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The type of error
    pub error_type: ErrorType,

    /// The location of the error
    pub position: Position,
}

impl Error {
    /// Creates a new `Error` with the given [`ErrorType`] and a default
    /// [`Position`]
    ///
    /// # Examples
    /// ```
    /// use tweep::{Error, ErrorType};
    /// # use tweep::{Position, Positional};
    /// let error = Error::new(ErrorType::EmptyName);
    /// # assert_eq!(error.error_type, ErrorType::EmptyName);
    /// # assert_eq!(error.get_position(), &Position::default());
    /// ```
    ///
    /// [`ErrorType`]: enum.ErrorType.html
    /// [`Position`]: enum.Position.html
    pub fn new(error_type: ErrorType) -> Self {
        Error {
            error_type,
            position: Position::StoryLevel,
        }
    }
}

impl Positional for Error {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.error_type, self.position)
    }
}
