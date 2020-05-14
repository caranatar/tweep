use std::collections::HashMap;

#[derive(Debug)]
pub enum StoryMap {
    Map(CodeMap),
    File(FileMap),
}

#[derive(Debug, Default)]
pub struct CodeMap {
    pub map: HashMap<String, FileMap>,
}

#[derive(Clone, Debug)]
pub struct FileMap {
    pub id: usize,
    pub contents: String,
}

impl StoryMap {
    pub fn set_file(&mut self, file: String) {
        if let StoryMap::File(file_map) = self {
            let mut map = HashMap::new();
            map.insert(file, file_map.clone());
            let story_map = StoryMap::Map(CodeMap { map });
            *self = story_map;
        }
    }
}

impl Default for StoryMap {
    fn default() -> Self {
        StoryMap::Map(CodeMap::default())
    }
}
