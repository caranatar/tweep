/// A position within the content of a Twee story.
///
/// By default, a `Position` is a global `StoryLevel`. Additional context can be
/// added from there to create a position at File, row, and column level.
///
/// # Notes
/// When setting an individual field of a `Position`, if the current enum
/// variant does not support that field, the value will automatically be
/// promoted to the most general variant that does support that field and
/// default values will be used for any remaining, unset fields. For instance,
/// if `set_column` is called on a `StoryLevel` variant, it will be promoted to
/// a `Column` variant with the given column value. However, if `set_file` is
/// called on a `Column` variant, it will be promoted to a `File` variant, which
/// will contain the newly set filename, the existing column value, and a
/// default row value of 0.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Position {
    /// A column location, contains the column number
    Column(usize),

    /// A row and column location, contains the row number then column number
    RowColumn(usize, usize),

    /// A file location, contains the file name, row, and column
    File(String, usize, usize),

    /// Default, global level
    StoryLevel,
}

impl Position {
    /// Gets the column of this `Position` if one exists.
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos.get_column(), None);
    /// pos.set_column(5);
    /// assert_eq!(pos.get_column(), Some(5));
    /// ```
    pub fn get_column(&self) -> Option<usize> {
        match self {
            Position::Column(col) => Some(*col),
            Position::RowColumn(_, col) => Some(*col),
            Position::File(_, _, col) => Some(*col),
            Position::StoryLevel => None,
        }
    }

    /// Sets the column of this `Position`. If the current value does not
    /// accomodate a column, the enum will be promoted to a value that does.
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos, Position::StoryLevel);
    /// pos.set_column(23);
    /// assert_eq!(pos, Position::Column(23));
    /// ```
    pub fn set_column(&mut self, col: usize) {
        *self = match self {
            Position::RowColumn(row, _) => Position::RowColumn(*row, col),
            Position::File(file, row, _) => Position::File(file.clone(), *row, col),
            _ => Position::Column(col),
        };
    }

    /// Gets the file name referenced by this `Position`, if there is one.
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos.get_file(), None);
    /// let file_name = "file.ext";
    /// pos.set_file(file_name.to_string());
    /// assert_eq!(pos.get_file(), Some(file_name));
    /// ```
    pub fn get_file(&self) -> Option<&str> {
        match &self {
            Position::File(file, _, _) => Some(file),
            _ => None,
        }
    }

    /// Sets the file name of this `Position`. If the current value does not
    /// accomodate a file name, the enum will be promoted to a value that does.
    /// In such a case, the row and column will be set to a default of 0 if they
    /// are not already set
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos, Position::StoryLevel);
    /// let file_name = "file.ext";
    /// pos.set_file(file_name.to_string());
    /// assert_eq!(pos, Position::File(file_name.to_string(), 0, 0));
    /// ```
    pub fn set_file(&mut self, file: String) {
        let row = self.get_row().unwrap_or(0);
        let col = self.get_column().unwrap_or(0);
        *self = Position::File(file, row, col);
    }

    /// Offsets the column of this `Position` by `offset`. If it's not currently
    /// storing a column, this has the same effect as `set_column(offset)`
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos.get_column(), None);
    /// pos.offset_column(5);
    /// assert_eq!(pos.get_column(), Some(5));
    /// pos.offset_column(5);
    /// assert_eq!(pos.get_column(), Some(10));
    /// ```
    pub fn offset_column(&mut self, offset: usize) {
        let col = self.get_column().unwrap_or(0);
        self.set_column(col + offset);
    }

    /// Gets the row of this `Position` if one exists.
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos.get_row(), None);
    /// pos.set_row(5);
    /// assert_eq!(pos.get_row(), Some(5));
    /// ```
    pub fn get_row(&self) -> Option<usize> {
        match self {
            Position::File(_, row, _) => Some(*row),
            Position::RowColumn(row, _) => Some(*row),
            _ => None,
        }
    }

    /// Sets the row of this `Position`. If the current value does not
    /// accomodate a row, the enum will be promoted to a value that does.
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos, Position::StoryLevel);
    /// pos.set_row(23);
    /// assert_eq!(pos, Position::RowColumn(23, 0));
    /// ```
    pub fn set_row(&mut self, row: usize) {
        let col = self.get_column().unwrap_or(0);
        *self = match self {
            Position::File(file, _, _) => Position::File(file.clone(), row, col),
            _ => Position::RowColumn(row, col),
        };
    }

    /// Offsets the row of this `Position` by `offset`. If it's not currently
    /// storing a row, this has the same effect as `set_row(offset)`
    ///
    /// # Examples
    /// ```
    /// use tweep::Position;
    /// let mut pos = Position::default();
    /// assert_eq!(pos.get_row(), None);
    /// pos.offset_row(5);
    /// assert_eq!(pos.get_row(), Some(5));
    /// pos.offset_row(5);
    /// assert_eq!(pos.get_row(), Some(10));
    /// ```
    pub fn offset_row(&mut self, offset: usize) {
        let row = self.get_row().unwrap_or(0);
        self.set_row(offset + row);
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Position::Column(col) => format!("column {}", col),
                Position::RowColumn(row, col) => format!("row {}, column {}", row, col),
                Position::File(file, row, col) => format!("{}: row {}, column {}", file, row, col),
                Position::StoryLevel => "story level".to_string(),
            }
        )
    }
}

impl std::default::Default for Position {
    fn default() -> Self {
        Self::StoryLevel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default() -> Position {
        let pos = Position::default();
        assert_eq!(pos, Position::StoryLevel);
        assert_eq!(pos.get_column(), None);
        assert_eq!(pos.get_row(), None);
        assert_eq!(pos.get_file(), None);

        pos
    }

    #[test]
    fn incremental_build() {
        let mut pos = tests::get_default();

        pos.set_column(23);
        assert_eq!(pos, Position::Column(23));
        assert_eq!(pos.get_column(), Some(23));
        assert_eq!(pos.get_row(), None);
        assert_eq!(pos.get_file(), None);

        pos.set_row(5);
        assert_eq!(pos, Position::RowColumn(5, 23));
        assert_eq!(pos.get_column(), Some(23));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), None);

        let file_name = "file.ext".to_string();
        pos.set_file(file_name.clone());
        assert_eq!(pos, Position::File(file_name.clone(), 5, 23));
        assert_eq!(pos.get_column(), Some(23));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), Some(&file_name[..]));
    }

    #[test]
    fn file() {
        let mut pos = tests::get_default();

        let file_name = "file.ext".to_string();
        pos.set_file(file_name.clone());
        assert_eq!(pos, Position::File(file_name.clone(), 0, 0));
        assert_eq!(pos.get_column(), Some(0));
        assert_eq!(pos.get_row(), Some(0));
        assert_eq!(pos.get_file(), Some(&file_name[..]));
    }

    #[test]
    fn row() {
        let mut pos = tests::get_default();

        pos.set_row(5);
        assert_eq!(pos, Position::RowColumn(5, 0));
        assert_eq!(pos.get_column(), Some(0));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), None);
    }

    #[test]
    fn offset_column() {
        let mut pos = tests::get_default();

        pos.offset_column(23);
        assert_eq!(pos, Position::Column(23));
        assert_eq!(pos.get_column(), Some(23));
        assert_eq!(pos.get_row(), None);
        assert_eq!(pos.get_file(), None);

        pos.offset_column(23);
        assert_eq!(pos, Position::Column(46));
        assert_eq!(pos.get_column(), Some(46));
        assert_eq!(pos.get_row(), None);
        assert_eq!(pos.get_file(), None);

        pos.set_row(5);
        assert_eq!(pos, Position::RowColumn(5, 46));
        assert_eq!(pos.get_column(), Some(46));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), None);

        pos.offset_column(23);
        assert_eq!(pos, Position::RowColumn(5, 69));
        assert_eq!(pos.get_column(), Some(69));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), None);

        let file_name = "file.ext".to_string();
        pos.set_file(file_name.clone());
        assert_eq!(pos, Position::File(file_name.clone(), 5, 69));
        assert_eq!(pos.get_column(), Some(69));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), Some(&file_name[..]));

        pos.offset_column(23);
        assert_eq!(pos, Position::File(file_name.clone(), 5, 92));
        assert_eq!(pos.get_column(), Some(92));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), Some(&file_name[..]));
    }

    #[test]
    fn offset_row() {
        let mut pos = tests::get_default();

        pos.offset_row(5);
        assert_eq!(pos, Position::RowColumn(5, 0));
        assert_eq!(pos.get_column(), Some(0));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), None);

        let file_name = "file.ext".to_string();
        pos.set_file(file_name.clone());
        assert_eq!(pos, Position::File(file_name.clone(), 5, 0));
        assert_eq!(pos.get_column(), Some(0));
        assert_eq!(pos.get_row(), Some(5));
        assert_eq!(pos.get_file(), Some(&file_name[..]));

        pos.offset_row(5);
        assert_eq!(pos, Position::File(file_name.clone(), 10, 0));
        assert_eq!(pos.get_column(), Some(0));
        assert_eq!(pos.get_row(), Some(10));
        assert_eq!(pos.get_file(), Some(&file_name[..]));
    }
}
