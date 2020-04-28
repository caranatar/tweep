use crate::ErrorType;
use crate::Position;
use crate::Positional;

/// Represents an error with an [`ErrorType`] and [`Position`]
///
/// [`ErrorType`]: enum.ErrorType.html
/// [`Position`]: enum.Position.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    error_type: ErrorType,
    position: Position,
}

impl Error {
    /// Creates a new `Error` with the given [`ErrorType`] and a default
    /// [`Position`]
    ///
    /// [`ErrorType`]: enum.ErrorType.html
    /// [`Position`]: enum.Position.html
    pub fn new(error_type: ErrorType) -> Self {
        Error { error_type, position: Position::StoryLevel }
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
