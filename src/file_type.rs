use std::fmt::Display;
use std::io;
use std::path::Path;

pub enum FileType {
    File,
    Dir,
    Symlink,
}

impl FileType {
    pub fn new(path: &Path) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;

        if metadata.is_file() {
            Ok(FileType::File)
        } else if metadata.is_dir() {
            Ok(FileType::Dir)
        } else {
            Ok(FileType::Symlink)
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type_str = match self {
            FileType::File => "file",
            FileType::Dir => "dir",
            FileType::Symlink => "symlink",
        };
        write!(f, "{}", file_type_str)
    }
}
