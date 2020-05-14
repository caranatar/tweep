use crate::ErrorList;
use crate::StoryMap;
use crate::Positional;
use crate::Error;

#[derive(Debug)]
pub struct ContextErrorList {
    pub error_list: ErrorList,
    pub story_map: StoryMap,
}

impl Positional for ContextErrorList {
    fn set_row(&mut self, row: usize) {
        self.error_list.set_row(row);
    }

    fn set_column(&mut self, col: usize) {
        self.error_list.set_column(col);
    }

    fn set_file(&mut self, file: String) {
        self.error_list.set_file(file.clone());
        self.story_map.set_file(file);
    }

    fn offset_column(&mut self, offset: usize) {
        self.error_list.offset_column(offset);
    }

    fn offset_row(&mut self, offset: usize) {
        self.error_list.offset_row(offset);
    }
}

impl std::error::Error for ContextErrorList {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for ContextErrorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error_list)
    }
}

impl std::convert::From<Error> for ContextErrorList {
    fn from(e: Error) -> ContextErrorList {
        let error_list = e.into();
        ContextErrorList {
            error_list,
            story_map: StoryMap::default(),
        }
    }
}
