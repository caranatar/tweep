use std::collections::HashMap;
use crate::FullContext;
use bimap::BiMap;
use std::ops::Range;

/// A code map for stories
///
/// The code map consists of a `BiMap` between file ids (usize) and file names
/// (String) along with a `HashMap` of file id to contexts
#[derive(Debug, Default)]
pub struct CodeMap {
    pub(crate) id_file_map: BiMap<usize, String>,
    pub(crate) contexts: HashMap<usize, FullContext>,
}

impl CodeMap {
    /// Gets the context for file id `id`
    pub fn get_context(&self, id: usize) -> Option<&FullContext> {
        self.contexts.get(&id)
    }

    /// Gets the file name for file id `id`
    pub fn lookup_name(&self, id: usize) -> Option<&str> {
        self.id_file_map.get_by_left(&id).map(|x| x.as_str())
    }

    /// Gets the file id for file name `name`
    pub fn lookup_id(&self, name: String) -> Option<usize> {
        self.id_file_map.get_by_right(&name).copied()
    }

    /// Gets the byte location of line starts for file id `id`
    pub fn line_starts(&self, id: usize) -> Option<&Vec<usize>> {
        self.get_context(id).map(|context| context.get_line_starts())
    }

    /// Gets the byte range of the line `line` for file id `id`
    pub fn line_range(&self, id: usize, line: usize) -> Option<Range<usize>> {
        self.get_context(id).and_then(|ctx| {
            let (start, end) = ctx.line_bytes(line).into_inner();
            Some(start..end+1)
        })
    }

    /// Adds a context to the code map
    pub(crate) fn add(&mut self, context: FullContext) {
        if let Some(file_name) = context.get_file_name() {
            let new_id = self.id_file_map.len();
            self.id_file_map.insert(new_id, file_name.clone());
            self.contexts.insert(new_id, context);
        }
    }
}

