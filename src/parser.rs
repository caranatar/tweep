/// This trait represents a parser which takes a reference to some input of type
/// `Input` and parses it to produce an output of type `Output`
pub trait Parser<'a> {
    /// The type produced by this parser
    type Output;

    /// The type accepted by this parser 
    type Input: ?Sized;

    /// Performs the parsing operation and returns the result
    ///
    /// # Arguments
    /// * `input` - the input to parse
    fn parse(input: &'a Self::Input) -> Self::Output;
}
