/// Indicates absolute/relative position
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionKind {
    /// Absolute position
    Absolute,

    /// Relative position
    Relative,
}

/// One-indexed, line and column numbers to be used within a [`Context`]
///
/// # Examples
/// ```
/// # use tweep::Position;
/// let c = Position::abs(1, 3);
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

    /// Kind of position
    pub kind: PositionKind,
}

impl Position {
    /// Create a new absolute `Position`
    ///
    /// # Examples
    /// ```
    /// # use tweep::Position;
    /// let c = Position::abs(2, 3);
    /// assert_eq!((c.line, c.column), (2, 3));
    /// ```
    pub fn abs(line: usize, column: usize) -> Self {
        Position { line, column, kind: PositionKind::Absolute }
    }

    /// Create a new relative `Position`
    ///
    /// # Examples
    /// ```
    /// # use tweep::Position;
    /// let c = Position::rel(2, 3);
    /// assert_eq!((c.line, c.column), (2, 3));
    /// ```
    pub fn rel(line: usize, column: usize) -> Self {
        Position { line, column, kind: PositionKind::Relative }
    }

    /// Creates another `Position` using one-indexed line and column
    /// numbers to offset from this position
    ///
    /// # Examples
    /// ```
    /// # use tweep::Position;
    /// let c = Position::abs(2, 3);
    /// // Since these are 1-indexed, this should be the same as c
    /// let s = c.subposition(1, 1);
    /// assert_eq!(c, s);
    /// ```
    ///
    /// ```
    /// # use tweep::Position;
    /// let c = Position::abs(2, 3);
    /// // When the line number is one, the sub position column will be offset
    /// // from the source position column
    /// let s = c.subposition(1, 3);
    /// assert_eq!(s, Position::abs(2, 5));
    /// ```
    ///
    /// ```
    /// # use tweep::Position;
    /// let c = Position::abs(2, 3);
    /// // When the line number is greater than one, the column number will be
    /// // offset from the start of that line
    /// let s = c.subposition(2, 1);
    /// assert_eq!(s, Position::abs(3, 1));
    /// ```
    pub fn subposition(&self, line: usize, column: usize) -> Self {
        let column = if line == 1 {
            self.column + column - 1
        } else {
            column
        };
        let line = self.line + line - 1;
        Self { line, column, kind: self.kind }
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
    use super::PositionKind;

    #[test]
    fn subpositions() {
        let p = Position { line: 2, column: 3, kind: PositionKind::Absolute };
        let sub = p.subposition(1, 1);
        assert_eq!(sub, p);
        let sub = p.subposition(1, 5);
        assert_eq!(sub, Position { line: 2, column: 7, kind: PositionKind::Absolute });
        let sub = p.subposition(4, 5);
        assert_eq!(sub, Position { line: 5, column: 5, kind: PositionKind::Absolute });
        let p = Position { kind: PositionKind::Relative, ..p };
        let sub = p.subposition(1, 1);
        assert_eq!(sub, p);
    }
}
