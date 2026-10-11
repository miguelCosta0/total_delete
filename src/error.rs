use std::{io, path::Path};

pub struct Error {
    pub error: io::Error,
    pub path: Option<String>,
}

impl Error {
    pub fn new(error: io::Error) -> Self {
        Self { error, path: None }
    }

    pub fn with_path(error: io::Error, path: &Path) -> Self {
        Self {
            error,
            path: Some(path.to_string_lossy().into_owned()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error { error, path: None }
    }
}
