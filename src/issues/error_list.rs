use crate::Error;
use crate::Positional;

/// A wrapper type for a list of [`Error`]s
///
/// [`Error`]: struct.Error.html
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ErrorList {
    /// The list of `Error`s
    pub errors: Vec<Error>,
}

impl ErrorList {
    /// Creates a new `ErrorList`
    ///
    /// # Examples
    /// ```
    /// use tweep::ErrorList;
    /// let errors = ErrorList::new();
    /// # assert!(errors.is_empty());
    /// ```
    pub fn new() -> Self {
        ErrorList::default()
    }

    /// Adds the given [`Error`] to the list
    ///
    /// # Examples
    /// ```
    /// use tweep::{Error, ErrorList, ErrorType};
    /// let mut errors = ErrorList::default();
    /// errors.push(Error::new(ErrorType::EmptyName));
    /// # assert_eq!(errors.errors, vec![ Error::new(ErrorType::EmptyName) ]);
    /// ```
    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Returns `true` if the list is empty
    ///
    /// # Examples
    /// ```
    /// use tweep::ErrorList;
    /// let errors = ErrorList::new();
    /// assert!(errors.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Given two `Result`s with `ErrorList` as the `Err` type, returns:
    /// * `Ok(())` if both inputs are `Ok`
    /// * The `ErrorList` contained by the `Err` input if one input is `Err`
    /// * The `ErrorList` of `right` appended to the `ErrorList` of `left` if
    /// both inputs are `Err`
    ///
    /// Note that `T` and `U` do not need to have any relation to each other.
    ///
    /// # Examples
    /// When given two `Ok` variant inputs, returns `Ok(())`:
    /// ```
    /// use tweep::ErrorList;
    /// let mut left:Result<u8, ErrorList> = Ok(5);
    /// let mut right:Result<&str, ErrorList> = Ok("foo");
    /// let merged = ErrorList::merge(&mut left, &mut right);
    /// assert_eq!(merged, Ok(()));
    /// ```
    ///
    /// When given one `Ok` and one `Err`, the output will have the same list of
    /// errors as the `Err` variant:
    /// ```
    /// use tweep::{Error, ErrorList, ErrorType, Position};
    /// let mut left:Result<u8, ErrorList> = Ok(5);
    /// let mut right:Result<&str, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorType::EmptyName) ],
    /// });
    /// let merged = ErrorList::merge(&mut left, &mut right);
    /// assert_eq!(merged.err().unwrap().errors, vec![ Error::new(ErrorType::EmptyName) ]);
    /// ```
    ///
    /// When given two `Err` variants, the output will be have an `ErrorList`
    /// that contains the errors in `right` appended to the errors in `left`
    /// ```
    /// use tweep::{Error, ErrorList, ErrorType, Position};
    /// let mut left:Result<u8, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorType::EmptyName) ],
    /// });
    /// let mut right:Result<&str, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorType::LeadingWhitespace) ],
    /// });
    /// let merged = ErrorList::merge(&mut left, &mut right);
    /// assert_eq!(merged.err().unwrap().errors, vec![ Error::new(ErrorType::EmptyName), Error::new(ErrorType::LeadingWhitespace) ]);
    /// ```
    pub fn merge<T, U>(
        left: &mut Result<T, ErrorList>,
        right: &mut Result<U, ErrorList>,
    ) -> Result<(), ErrorList> {
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

    fn set_file(&mut self, file: String) {
        for error in &mut self.errors {
            error.set_file(file.clone());
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
            if res.is_err() {
                return res;
            }
        }
        res
    }
}

impl std::convert::From<Error> for ErrorList {
    fn from(e: Error) -> ErrorList {
        let mut error_list = ErrorList::default();
        error_list.push(e);
        error_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorType;

    #[test]
    fn basic() {
        let mut errs = ErrorList::default();
        assert!(errs.is_empty());
        errs.push(Error::new(ErrorType::EmptyName));
        assert!(!errs.is_empty());
        assert_eq!(errs.errors, vec![Error::new(ErrorType::EmptyName)]);
    }

    #[test]
    fn positional() {
        let mut errs = ErrorList::default();
        assert!(errs.is_empty());
        errs.push(Error::new(ErrorType::EmptyName));
        errs.push(Error::new(ErrorType::MissingSigil));
        assert!(!errs.is_empty());
        assert_eq!(
            errs.errors,
            vec![
                Error::new(ErrorType::EmptyName),
                Error::new(ErrorType::MissingSigil)
            ]
        );

        errs.set_row(23);
        assert!(errs.errors.iter().all(|e| e.get_row() == Some(23)));

        errs.set_column(5);
        assert!(errs.errors.iter().all(|e| e.get_column() == Some(5)));

        errs.offset_row(9);
        assert!(errs.errors.iter().all(|e| e.get_row() == Some(32)));

        errs.offset_column(18);
        assert!(errs.errors.iter().all(|e| e.get_column() == Some(23)));

        let file_name = "file.ext";
        errs.set_file(file_name.to_string());
        assert!(errs.errors.iter().all(|e| e.get_file() == Some(file_name)));
    }

    #[test]
    fn merge() {
        let mut ok_left = Ok(());
        let mut ok_right = Ok(());

        let error_list_left = ErrorList {
            errors: vec![Error::new(ErrorType::EmptyName)],
        };
        let error_list_right = ErrorList {
            errors: vec![Error::new(ErrorType::MissingSigil)],
        };

        let mut err_left: Result<(), _> = Err(error_list_left.clone());
        let mut err_right: Result<(), _> = Err(error_list_right.clone());

        assert!(ErrorList::merge(&mut ok_left, &mut ok_right).is_ok());

        let errs = ErrorList::merge(&mut err_left.clone(), &mut ok_right)
            .err()
            .unwrap();
        assert_eq!(&errs.errors, &error_list_left.errors);

        let errs = ErrorList::merge(&mut ok_left, &mut err_right.clone())
            .err()
            .unwrap();
        assert_eq!(&errs.errors, &error_list_right.errors);

        let errs = ErrorList::merge(&mut err_left, &mut err_right)
            .err()
            .unwrap();
        let mut expected = error_list_left.errors.clone();
        expected.append(&mut error_list_right.errors.clone());
        assert_eq!(errs.errors, expected);
    }
}
