/// One-indexed, line and column numbers to be used within a [`Context`]
///
/// # Examples
/// ```
/// # use tweep::Position;
/// let c = Position::new(1, 3);
/// assert_eq!((c.line, c.column), (1, 3));
/// ```
///
/// [`Context`]: struct.Context.html
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    /// The one-indexed line number
    pub line: usize,

    /// The one-indexed column number
    pub column: usize,
}

impl Position {
    /// Create a new `Position` with the given one-indexed line and
    /// column numbers.
    ///
    /// # Examples
    /// ```
    /// # use tweep::Position;
    /// let c = Position::new(1, 3);
    /// assert_eq!((c.line, c.column), (1, 3));
    /// ```
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }

    /// Creates another `Position` using one-indexed line and column
    /// numbers to offset from this position
    ///
    /// # Examples
    /// ```
    /// # use tweep::Position;
    /// let c = Position::new(2, 3);
    /// // Since these are 1-indexed, this should be the same as c
    /// let s = c.subposition(1, 1);
    /// assert_eq!(c, s);
    /// ```
    ///
    /// ```
    /// # use tweep::Position;
    /// let c = Position::new(2, 3);
    /// // When the line number is one, the sub position column will be offset
    /// // from the source position column
    /// let s = c.subposition(1, 3);
    /// assert_eq!(s, Position::new(2, 5));
    /// ```
    ///
    /// ```
    /// # use tweep::Position;
    /// let c = Position::new(2, 3);
    /// // When the line number is greater than one, the column number will be
    /// // offset from the start of that line
    /// let s = c.subposition(2, 1);
    /// assert_eq!(s, Position::new(3, 1));
    /// ```
    pub fn subposition(&self, line: usize, column: usize) -> Self {
        let column = if line == 1 {
            self.column + column - 1
        } else {
            column
        };
        let line = self.line + line - 1;
        Self { line, column }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {} column: {}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn subpositions() {
        let p = Position { line: 2, column: 3 };
        let sub = p.subposition(1, 1);
        assert_eq!(sub, p);
        let sub = p.subposition(1, 5);
        assert_eq!(sub, Position { line: 2, column: 7 });
        let sub = p.subposition(4, 5);
        assert_eq!(sub, Position { line: 5, column: 5 });
    }
}
