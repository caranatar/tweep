#[cfg(feature = "issue-context")]
pub trait Contextual {
    fn get_context_len(&self) -> &Option<usize>;

    fn mut_context_len(&mut self) -> &mut Option<usize>;

    fn with_context_len(mut self, context_len: usize) -> Self where Self: Sized {
        *self.mut_context_len() = Some(context_len);
        self
    }
}
