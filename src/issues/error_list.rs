use crate::Error;
use crate::Positional;

/// Represents a list of [`Error`]s
///
/// [`Error`]: struct.Error.html
#[derive(Debug)]
pub struct ErrorList {
    /// The list of `Error`s
    pub errors: Vec<Error>,
}

impl ErrorList {
    /// Creates a new empty `ErrorList`
    pub fn new() -> Self {
        ErrorList { errors: Vec::new() }
    }

    /// Adds the given [`Error`] to the list
    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Returns `true` if the list is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Given two `Result`s with `ErrorList` as the `Err` type, returns:
    /// * `Ok(())` if both inputs are `Ok`
    /// * The `ErrorList` contained by the `Err` input if one input is `Err`
    /// * The `ErrorList` of `right` appended to the `ErrorList` of `left` if
    /// both inputs are `Err`
    pub fn merge<T,U>(left: &mut Result<T, ErrorList>,
                      right: &mut Result<U, ErrorList>) -> Result<(), ErrorList> {
        let mut errors = Vec::new();
        if left.is_err() {
            errors.append(&mut left.as_mut().err().unwrap().errors);
        }

        if right.is_err() {
            errors.append(&mut right.as_mut().err().unwrap().errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ErrorList { errors })
        }
    }
}

impl Positional for ErrorList {
    fn set_row(&mut self, row: usize) {
        for error in &mut self.errors {
            error.set_row(row);
        }
    }

    fn set_column(&mut self, col: usize) {
        for error in &mut self.errors {
            error.set_column(col);
        }
    }

    fn offset_column(&mut self, offset: usize) {
        for error in &mut self.errors {
            error.offset_column(offset);
        }
    }

    fn offset_row(&mut self, offset: usize) {
        for error in &mut self.errors {
            error.offset_row(offset);
        }
    }
}

impl std::error::Error for ErrorList {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for ErrorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = Ok(());
        for error in &self.errors {
            res = writeln!(f, "{}", error);
            if res.is_err() { return res; }
        }
        res
    }
}

impl std::convert::From<Error> for ErrorList {
    fn from(e: Error) -> ErrorList {
        let mut error_list = ErrorList::new();
        error_list.push(e);
        error_list
    }
}
