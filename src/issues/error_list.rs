use crate::Error;

/// A wrapper type for a list of [`Error`]s
///
/// [`Error`]: struct.Error.html
#[derive(Debug, Default, Eq, PartialEq)]
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
    /// use tweep::{Error, ErrorList, ErrorKind, FullContext};
    /// let mut errors = ErrorList::default();
    /// let context = FullContext::from(None, "::".to_string());
    /// errors.push(Error::new(ErrorKind::EmptyName, Some(context.clone())));
    /// # assert_eq!(errors.errors, vec![ Error::new(ErrorKind::EmptyName, Some(context.clone())) ]);
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
    /// use tweep::{Error, ErrorList, ErrorKind, FullContext, Position};
    /// let mut left:Result<u8, ErrorList> = Ok(5);
    /// let right_context = FullContext::from(None, "::".to_string());
    /// let mut right:Result<&str, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorKind::EmptyName, Some(right_context.clone())) ],
    /// });
    /// let merged = ErrorList::merge(&mut left, &mut right);
    /// assert_eq!(merged.err().unwrap().errors, vec![ Error::new(ErrorKind::EmptyName, Some(right_context.clone())) ]);
    /// ```
    ///
    /// When given two `Err` variants, the output will be have an `ErrorList`
    /// that contains the errors in `right` appended to the errors in `left`
    /// ```
    /// use tweep::{Error, ErrorList, ErrorKind, FullContext, Position};
    /// let left_context = FullContext::from(None, "::".to_string());
    /// let mut left:Result<u8, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorKind::EmptyName, Some(left_context.clone())) ],
    /// });
    /// let right_context = FullContext::from(None, " :: Blah".to_string());
    /// let mut right:Result<&str, ErrorList> = Err(ErrorList {
    ///     errors: vec![ Error::new(ErrorKind::LeadingWhitespace, Some(right_context.clone())) ],
    /// });
    /// let merged = ErrorList::merge(&mut left, &mut right);
    /// assert_eq!(merged.err().unwrap().errors, vec![ Error::new(ErrorKind::EmptyName, Some(left_context.clone())), Error::new(ErrorKind::LeadingWhitespace, Some(right_context.clone())) ]);
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

impl  std::convert::From<Error> for ErrorList {
    fn from(e: Error) -> ErrorList {
        let mut error_list = ErrorList::default();
        error_list.push(e);
        error_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use crate::FullContext;

    #[test]
    fn basic() {
        let mut errs = ErrorList::default();
        assert!(errs.is_empty());
        let context = FullContext::from(None, "::".to_string());
        let expected = context.clone();
        errs.push(Error::new(ErrorKind::EmptyName, Some(context)));
        assert!(!errs.is_empty());
        assert_eq!(errs.errors, vec![Error::new(ErrorKind::EmptyName, Some(expected))]);
    }

    #[test]
    fn positional() {
        let mut errs = ErrorList::default();
        assert!(errs.is_empty());
        errs.push(Error::new(ErrorKind::EmptyName, Some(FullContext::from(None, "::".to_string()))));
        errs.push(Error::new(ErrorKind::MissingSigil, Some(FullContext::from(None, "Blah".to_string()))));
        assert!(!errs.is_empty());
        assert_eq!(
            errs.errors,
            vec![
                Error::new(ErrorKind::EmptyName, Some(FullContext::from(None, "::".to_string()))),
                Error::new(ErrorKind::MissingSigil, Some(FullContext::from(None, "Blah".to_string())))
            ]
        );
    }

    #[test]
    fn merge() {
        let mut ok_left = Ok(());
        let mut ok_right = Ok(());

        fn error_list_left() -> ErrorList {
            ErrorList {
                errors: vec![Error::new(ErrorKind::EmptyName, Some(FullContext::from(None, "::".to_string())))],
            }
        };
        fn error_list_right() -> ErrorList {
            ErrorList {
                errors: vec![Error::new(ErrorKind::MissingSigil, Some(FullContext::from(None, "Blah".to_string())))],
            }
        };

        assert!(ErrorList::merge(&mut ok_left, &mut ok_right).is_ok());

        let mut err_left: Result<(), _> = Err(error_list_left());
        let errs = ErrorList::merge(&mut err_left, &mut ok_right)
            .err()
            .unwrap();
        assert_eq!(&errs.errors, &error_list_left().errors);

        let mut err_right: Result<(), _> = Err(error_list_right());
        let errs = ErrorList::merge(&mut ok_left, &mut err_right)
            .err()
            .unwrap();
        assert_eq!(&errs.errors, &error_list_right().errors);

        let mut err_left: Result<(), _> = Err(error_list_left());
        let mut err_right: Result<(), _> = Err(error_list_right());
        let errs = ErrorList::merge(&mut err_left, &mut err_right)
            .err()
            .unwrap();
        let mut expected = error_list_left().errors;
        expected.append(&mut error_list_right().errors);
        assert_eq!(errs.errors, expected);
    }
}
