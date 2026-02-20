use std::{io, path::Path};

pub enum FileError {
    PermissionDenied(String),
    NotFound(String),
    Other(io::ErrorKind),
}

impl FileError {
    pub fn new(filepath: &Path, error: io::Error) -> Self {
        let path_str = filepath.to_str().unwrap().to_string();
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound(path_str),
            io::ErrorKind::PermissionDenied => Self::PermissionDenied(path_str),
            other_err => Self::Other(other_err),
        }
    }
}
