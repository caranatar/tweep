use crate::Warning;
use crate::Positional;

/// Represents the output of an operation along with a [`Vec`] of any
/// [`Warning`]s generated by the operation.
///
/// If the contained type `T` implements the [`Positional`] trait, then
/// `Output<T>` will also implement the [`Positional`] trait, and will apply any
/// changes to both the contained output and the contained [`Warning`]s.
///
/// [`Vec`]: std::Vec
/// [`Warning`]: struct.Warning.html
/// [`Positional`]: trait.Positional.html
pub struct Output<T> {
    /// The output of the operation
    output: T,

    /// The associated [`Warning`]s
    warnings: Vec<Warning>,
}

impl<T> Output<T> {
    /// Creates a new `Output` with the given value and no [`Warning`]s
    ///
    /// # Arguments
    /// * `value` - The output value
    ///
    /// # Examples
    /// ```
    /// use tweep::Output;
    /// let out:Output<u32> = Output::new(23);
    /// assert_eq!(*out.get_output(), 23);
    /// assert!(!out.has_warnings());
    /// ```
    pub fn new(value: T) -> Self {
        Output { output: value, warnings: Vec::new() }
    }

    /// Builder method to add [`Warning`]s to an `Output` and return the object
    ///
    /// # Arguments
    /// * `warnings` - The list of [`Warning`]s to add to the object
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Warning, WarningType};
    /// let warnings = vec![ Warning::new(WarningType::MissingStoryTitle) ];
    /// let out:Output<u32> = Output::new(23).with_warnings(warnings.clone());
    /// assert!(out.has_warnings());
    /// assert_eq!(*out.get_warnings(), warnings);
    /// ```
    pub fn with_warnings(mut self, warnings: Vec<Warning>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Returns a reference to the output field
    ///
    /// # Examples
    /// ```
    /// use tweep::Output;
    /// let out = Output::new("hail eris");
    /// assert_eq!(*out.get_output(), "hail eris");
    /// ```
    pub fn get_output(&self) -> &T {
        &self.output
    }

    /// Returns a mutable reference to the output field
    ///
    /// # Examples
    /// ```
    /// use tweep::Output;
    /// let mut out = Output::new(23 as usize);
    /// *out.mut_output() = 5;
    /// assert_eq!(*out.get_output(), 5);
    /// ```
    pub fn mut_output(&mut self) -> &mut T {
        &mut self.output
    }

    /// Returns `true` if the object has associated [`Warning`]s
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Warning, WarningType};
    /// let out:Output<u8> = Output::new(5);
    /// assert!(!out.has_warnings());
    /// let out:Output<u8> = Output::new(5)
    ///     .with_warnings(vec![ Warning::new(WarningType::UnclosedLink) ]);
    /// assert!(out.has_warnings());
    /// ```
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns a reference to the associated [`Vec`] of [`Warning`]s
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Warning, WarningType};
    /// let out:Output<u8> = Output::new(5)
    ///     .with_warnings(vec![ Warning::new(WarningType::UnclosedLink) ]);
    /// assert_eq!(out.get_warnings(), &vec![ Warning::new(WarningType::UnclosedLink) ]);
    /// ```
    pub fn get_warnings(&self) -> &Vec<Warning> {
        &self.warnings
    }

    /// Consumes the `Output` and returns the `output` and `warnings` as a tuple
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Warning, WarningType};
    /// let warnings = vec![ Warning::new(WarningType::MissingStoryTitle) ];
    /// let out:Output<u32> = Output::new(23).with_warnings(warnings.clone());
    /// let (t, w) = out.take();
    /// assert_eq!(t, 23);
    /// assert_eq!(w, warnings);
    ///
    /// // Error!
    /// // let _ = out.get_output();
    /// ```
    pub fn take(self) -> (T, Vec<Warning>) {
        (self.output, self.warnings)
    }
}

/// This provides a handful of utility methods for an `Output` that contains a
/// [`Result`] as its contained output
///
/// [`Result`]: std::result::Result
impl<T,E> Output<Result<T,E>> {
    /// Returns `true` if the contained `Result` is `Ok`
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Error};
    /// let out:Output<Result<u32, Error>> = Output::new(Ok(23));
    /// assert!(out.is_ok());
    /// ```
    pub fn is_ok(&self) -> bool {
        self.output.is_ok()
    }

    /// Returns `true` if the contained `Result` is `Err`
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Error, ErrorType, FullContext};
    /// let context = FullContext::from(None, "::".to_string());
    /// let err = Error::new(ErrorType::EmptyName, context);
    /// let out:Output<Result<u32, Error>> = Output::new(Err(err));
    /// assert!(out.is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        self.output.is_err()
    }

    /// Converts this `Output<Result<T,E>>` into an `Output<Result<U,F>>`, where
    /// `T` can be converted into `U`.
    ///
    /// # Panics
    /// Panics if the contained `Result<T,E>` is not `Ok`
    ///
    /// # Examples
    /// ```
    /// use tweep::{Output, Error};
    /// let out:Output<Result<u8, Error>> = Output::new(Ok(23));
    ///
    /// // The destination Error type does not need to be convertible from the source
    /// let other:Output<Result<u32, String>> = out.into_ok();
    /// assert!(other.is_ok());
    /// assert_eq!(*other.get_output(), Ok(23));
    /// ```
    pub fn into_ok<U, F>(self) -> Output<Result<U,F>> where T: Into<U> {
        let (res, warnings) = self.take();
        let ok:U = res.ok().unwrap().into();
        Output::new(Ok(ok)).with_warnings(warnings)
    }

    /// Converts this `Output<Result<T,E>>` into an `Output<Result<U,F>>`, where
    /// `E` can be converted into `F`
    ///
    /// # Panics
    /// Panics if the contained `Result<T,E>` is not `Err`
    ///
    /// # Examples
    /// ```
    /// use tweep::Output;
    /// let out:Output<Result<u8, u8>> = Output::new(Err(5));
    ///
    /// // The destination Ok type does not need to be convertible from the source
    /// let other:Output<Result<String, u32>> = out.into_err();
    /// assert!(other.is_err());
    /// assert_eq!(*other.get_output(), Err(5));
    /// ```
    pub fn into_err<U, F>(self) -> Output<Result<U,F>> where E: Into<F> {
        let (res,warnings) = self.take();
        let err:F = res.err().unwrap().into();
        Output::new(Err(err)).with_warnings(warnings)
    }

    /// Converts this `Output<Result<T,E>>` into an `Output<Result<U,F>>`, where
    /// `T` can be converted into `U` and `E` can be converted into `F`
    ///
    /// # Examples
    /// ```
    /// use tweep::Output;
    /// let mut out:Output<Result<u8, u8>> = Output::new(Ok(23));
    /// let mut other:Output<Result<u32, u32>> = out.into_result();
    /// assert!(other.is_ok());
    /// assert_eq!(*other.get_output(), Ok(23));
    /// out = Output::new(Err(5));
    /// other = out.into_result();
    /// assert!(other.is_err());
    /// assert_eq!(*other.get_output(), Err(5));
    /// ```
    pub fn into_result<U,F>(self) -> Output<Result<U,F>> where T: Into<U>, E: Into<F> {
        if self.is_ok() {
            self.into_ok()
        } else {
            self.into_err()
        }
    }
}

impl<T> Positional for Output<T> where T: Positional {
    fn set_row(&mut self, row: usize) {
        self.output.set_row(row);
        for warning in &mut self.warnings {
            warning.set_row(row);
        }
    }

    fn set_column(&mut self, col: usize) {
        self.output.set_column(col);
        for warning in &mut self.warnings {
            warning.set_column(col);
        }
    }

    fn offset_column(&mut self, offset: usize) {
        self.output.offset_column(offset);
        for warning in &mut self.warnings {
            warning.offset_column(offset);
        }
    }

    fn offset_row(&mut self, offset: usize) {
        self.output.offset_row(offset);
        for warning in &mut self.warnings {
            warning.offset_row(offset);
        }
    }

    fn set_file(&mut self, file: String) {
        for warning in &mut self.warnings {
            warning.set_file(file.clone());
        }
        self.output.set_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let out:Output<u8> = Output::new(5);
        assert!(!out.has_warnings());
        assert_eq!(out.get_output(), &5);
        assert_eq!(out.get_warnings(), &Vec::new());
        let (x,w) = out.take();
        assert_eq!(x, 5);
        assert_eq!(w, Vec::new());
    }

    #[test]
    fn basic_with_warnings() {
        use crate::WarningType;
        use crate::FullContext;
        let context = FullContext::from(None, "".to_string());
        let warnings = vec![ Warning::new(WarningType::DuplicateStoryData, context.clone()),
                             Warning::new(WarningType::DuplicateStoryTitle, context.clone()) ];
        let expected = vec![ Warning::new(WarningType::DuplicateStoryData, context.clone()),
                             Warning::new(WarningType::DuplicateStoryTitle, context.clone()) ];
        let out:Output<u8> = Output::new(5).with_warnings(warnings);
        assert!(out.has_warnings());
        assert_eq!(out.get_output(), &5);
        assert_eq!(out.get_warnings(), &expected);
        let (x,w) = out.take();
        assert_eq!(x, 5);
        assert_eq!(w, expected);
    }

    #[test]
    fn output_result() {
        let ok_out:Output<Result<u8,String>> = Output::new(Ok(5));
        let err_out:Output<Result<String,u8>> = Output::new(Err(23));

        assert!(ok_out.is_ok());
        assert_eq!(ok_out.get_output(), &Ok(5));

        assert!(err_out.is_err());
        assert_eq!(err_out.get_output(), &Err(23));

        let x:Output<Result<u32,u32>> = ok_out.into_ok();
        assert!(x.is_ok());
        assert_eq!(x.get_output(), &Ok(5));

        let y:Output<Result<u64, u64>> = x.into_result();
        assert!(y.is_ok());
        assert_eq!(y.get_output(), &Ok(5));

        let x:Output<Result<u32,u32>> = err_out.into_err();
        assert!(x.is_err());
        assert_eq!(x.get_output(), &Err(23));

        let y:Output<Result<u64, u64>> = x.into_result();
        assert!(y.is_err());
        assert_eq!(y.get_output(), &Err(23));
    }

    #[test]
    #[should_panic]
    fn into_ok_panic() {
        let x:Output<Result<u8,u8>> = Output::new(Err(5));
        let _:Output<Result<u32,String>> = x.into_ok();
    }

    #[test]
    #[should_panic]
    fn into_err_panic() {
        let x:Output<Result<u8,u8>> = Output::new(Ok(5));
        let _:Output<Result<String,u32>> = x.into_err();
    }

    use crate::Position;
    struct PositionalU8 {
        pub number: u8,
        pub position: Position,
    }

    impl PositionalU8 {
        pub fn new(number: u8) -> Self {
            PositionalU8 { number, position: Position::default() }
        }
    }

    impl Positional for PositionalU8 {
        fn get_position(&self) -> &Position {
            &self.position
        }

        fn mut_position(&mut self) -> &mut Position {
            &mut self.position
        }
    }

    fn test_positions(warnings: &Vec<Warning>, position: &Position) {
        let warning_positions:Vec<&Position> = warnings
            .iter()
            .map(|w| w.get_position())
            .collect();
        for pos in warning_positions {
            assert_eq!(pos, position);
        }
    }
    
    #[test]
    fn positional() {
        use crate::WarningType;
        use crate::FullContext;
        let context = FullContext::from(None, "".to_string());
        let warnings = vec![ Warning::new(WarningType::DuplicateStoryData, context.subcontext(..)),
                             Warning::new(WarningType::DuplicateStoryTitle, context.subcontext(..)) ];
        let expected = vec![ Warning::new(WarningType::DuplicateStoryData, context.subcontext(..)),
                             Warning::new(WarningType::DuplicateStoryTitle, context.subcontext(..)) ];
        let mut out:Output<PositionalU8> = Output::new(PositionalU8::new(5)).with_warnings(warnings);
        assert!(out.has_warnings());
        assert_eq!(out.get_output().number, 5);
        assert_eq!(out.get_warnings(), &expected);

        let mut expected_position = Position::default();
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);

        expected_position.set_column(2);
        out.set_column(2);
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);

        expected_position.set_row(3);
        out.set_row(3);
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);

        let file_name = "blah.ext".to_string();
        expected_position.set_file(file_name.clone());
        out.set_file(file_name);
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);

        expected_position.offset_column(3);
        out.offset_column(3);
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);

        expected_position.offset_row(20);
        out.offset_row(20);
        assert_eq!(out.get_output().get_position(), &expected_position);
        test_positions(out.get_warnings(), &expected_position);
    }
}
