use crate::ErrorList;
use crate::CodeMap;
use crate::Error;

/// An ErrorList with an attached CodeMap
#[derive(Debug)]
pub struct ContextErrorList {
    /// The underlying ErrorList
    pub error_list: ErrorList,

    /// The attached CodeMap
    pub code_map: CodeMap,
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
            code_map: CodeMap::default(),
        }
    }
}
