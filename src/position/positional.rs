use crate::Position;

/// This trait represents something that has one or more `Position`s associated
/// with it, which can be manipulated through the interface provided.
///
/// There exists two major use cases for this trait:
/// # Basic usage
/// If the implementor of the trait has a single `Position` that can be directly
/// manipulated through the interface, it only needs to provide implementations
/// for the `get_position` and `mut_position` methods. All additional
/// functionality will be provided by the default trait implementations.
///
/// # Advanced usage
/// If the implementor of the trait has more complex positional data to track
/// (multiple `Position`s for instance), the situation becomes more complex. Any
/// applicable `set_*` and `offset_*` methods should be implemented directly and
/// the `get_position` and `mut_position` methods will default to unimplemented.
/// The `with_*` functionality will be automatically available as long as the
/// associated `set_*` functionality is provided, but any other functionality
/// will not be available.
///
/// # `Result`
/// An implementation of `Positional` is provided for any `Result<T, E>` where
/// `T` and `E` themselves both implement `Positional` which will forward any
/// `Positional` calls to the `Ok` or `Err` variant contained within the
/// `Result`.
pub trait Positional {
    /// Get an immutable reference to the associated `Position`. Unimplemented
    /// by default.
    fn get_position(&self) -> &Position {
        unimplemented!()
    }

    /// Get an mutable reference to the associated `Position`. Unimplemented
    /// by default.
    fn mut_position(&mut self) -> &mut Position {
        unimplemented!()
    }

    /// Gets the column of the associated `Position` if it has one
    ///
    /// Uses `get_position` by default
    fn get_column(&self) -> Option<usize> {
        self.get_position().get_column()
    }

    /// Sets the column of the associated `Position`
    ///
    /// Uses `mut_position` by default
    fn set_column(&mut self, col: usize) {
        self.mut_position().set_column(col)
    }

    /// Offsets the column of the associated `Position` by `offset`
    ///
    /// Uses `mut_position` by default
    fn offset_column(&mut self, offset: usize) {
        self.mut_position().offset_column(offset)
    }

    /// Gets the row of the associated `Position` if it has one
    ///
    /// Uses `get_position` by default
    fn get_row(&self) -> Option<usize> {
        self.get_position().get_row()
    }

    /// Sets the row of the associated `Position`
    ///
    /// Uses `mut_position` by default
    fn set_row(&mut self, row: usize) {
        self.mut_position().set_row(row)
    }

    /// Offsets the row of the associated `Position` by `offset`
    ///
    /// Uses `mut_position` by default
    fn offset_row(&mut self, offset: usize) {
        self.mut_position().offset_row(offset)
    }

    /// Gets the file name of the associated `Position` if it has one
    ///
    /// Uses `get_position` by default
    fn get_file(&self) -> Option<&str> {
        self.get_position().get_file()
    }

    /// Sets the file name of the associated `Position`
    ///
    /// Uses `mut_position` by default
    fn set_file(&mut self, file: String) {
        self.mut_position().set_file(file)
    }

    /// Moves `self`, calls `offset_row` with the given `offset` and returns a
    /// new instance with the offset value
    ///
    /// Uses `offset_row` by default
    fn with_offset_row(mut self, offset: usize) -> Self where Self: Sized {
        self.offset_row(offset);
        self
    }

    /// Moves `self`, calls `offset_column` with the given `offset` and returns
    /// a new instance with the offset value
    ///
    /// Uses `offset_column` by default
    fn with_offset_column(mut self, offset: usize) -> Self where Self: Sized {
        self.offset_column(offset);
        self
    }

    /// Moves `self`, calls `set_column` with the given `col` value and returns
    /// a new instance with the set value
    ///
    /// Uses `set_column` by default
    fn with_column(mut self, col: usize) -> Self where Self: Sized {
        self.set_column(col);
        self
    }

    /// Moves `self`, calls `set_row` with the given `row` value and returns a
    /// new instance with the set value
    ///
    /// Uses `set_row` by default
    fn with_row(mut self, row: usize) -> Self where Self: Sized {
        self.set_row(row);
        self
    }

    /// Moves `self`, calls `set_file` with the given `file` value and returns a
    /// new instance with the set value
    ///
    /// Uses `set_file` by default
    fn with_file(mut self, file: String) -> Self where Self: Sized {
        self.set_file(file);
        self
    }
}

impl<T,E> Positional for Result<T, E> where T: Positional, E: Positional {
    fn get_position(&self) -> &Position {
        match self {
            Ok(t) => t.get_position(),
            Err(e) => e.get_position(),
        }
    }

    fn mut_position(&mut self) -> &mut Position {
        match self {
            Ok(t) => t.mut_position(),
            Err(e) => e.mut_position(),
        }
    }

    fn get_column(&self) -> Option<usize> {
        match self {
            Ok(t) => t.get_column(),
            Err(e) => e.get_column(),
        }
    }

    fn set_column(&mut self, col: usize) {
        match self {
            Ok(t) => t.set_column(col),
            Err(e) => e.set_column(col),
        }
    }

    fn offset_column(&mut self, offset: usize) {
        match self {
            Ok(t) => t.offset_column(offset),
            Err(e) => e.offset_column(offset),
        }
    }

    fn get_row(&self) -> Option<usize> {
        match self {
            Ok(t) => t.get_row(),
            Err(e) => e.get_row(),
        }
    }
    
    fn set_row(&mut self, row: usize) {
        match self {
            Ok(t) => t.set_row(row),
            Err(e) => e.set_row(row),
        }
    }

    fn offset_row(&mut self, offset: usize) {
        match self {
            Ok(t) => t.offset_row(offset),
            Err(e) => e.offset_row(offset),
        }
    }

    fn get_file(&self) -> Option<&str> {
        match self {
            Ok(t) => t.get_file(),
            Err(e) => e.get_file(),
        }
    }

    fn set_file(&mut self, file: String) {
        match self {
            Ok(t) => t.set_file(file),
            Err(e) => e.set_file(file),
        }
    }
}
