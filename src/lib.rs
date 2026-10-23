pub mod error;
mod file_type;

use error::Error;
use file_type::FileType;

use std::fs::{self, File, Permissions, canonicalize, remove_dir};
use std::io::{self, Write, stdout};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::{env, result};

type Result<T> = result::Result<T, Error>;

const MEBIBYTE: usize = 1024 * 1024;
static MIB_NULL_DATA: [u8; MEBIBYTE] = [0; MEBIBYTE];

pub fn run(file_paths: &[String]) -> Result<()> {
    if file_paths.len() == 0 {
        return Err(Error {
            error: io::Error::other("No arguments provided"),
            path: None,
        });
    }

    let abs_paths = get_abs_paths(file_paths)?;

    for path in &abs_paths {
        let file_type = FileType::new(path).or_else(|e| Err(Error::with_path(e, path)))?;
        println!("- {} ({})", path.display(), file_type);
    }

    if !continue_program() {
        println!("Aborting...");
        return Ok(());
    }

    for path in &abs_paths {
        total_delete(path)?;
    }

    Ok(())
}

fn get_abs_paths(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut abs_paths: Vec<PathBuf> = Vec::with_capacity(paths.len());
    for path in paths {
        let path = Path::new(path);
        let abs_path = if path.is_symlink() {
            get_symlink_abs_path(path)
        } else {
            canonicalize(path).or_else(|e| Err(Error::with_path(e, path)))?
        };
        abs_paths.push(abs_path);
    }
    Ok(abs_paths)
}

fn get_symlink_abs_path(path: &Path) -> PathBuf {
    if path.has_root() {
        return path.to_path_buf();
    }

    let mut abs_path = Path::new(".").canonicalize().unwrap();
    for comp in path.components() {
        match comp {
            Component::ParentDir => _ = abs_path.pop(),
            Component::Normal(c) => abs_path.push(c),
            _ => (),
        }
    }

    abs_path
}

fn continue_program() -> bool {
    let mut attempts: u8 = 0;
    while attempts < 3 {
        print!("Are you sure you want to permanently delete these files? [y/n] ");
        io::stdout().flush().unwrap();

        let mut ans = String::new();
        io::stdin().read_line(&mut ans).unwrap();
        let ans = ans.trim().to_ascii_lowercase();

        match ans.as_str() {
            "no" | "n" => return false,
            "yes" | "ye" | "y" => return true,
            _ => println!("Invalid answer"),
        }

        attempts += 1;
    }

    false
}

fn total_delete(path: &PathBuf) -> Result<()> {
    let file_type = FileType::new(path).or_else(|e| Err(Error::with_path(e, path)))?;

    let prohibited_paths = vec![
        env::home_dir().unwrap(),
        env::home_dir().unwrap().join(".config"),
    ];

    if prohibited_paths.contains(&path) {
        return Err(Error::with_path(
            io::Error::new(io::ErrorKind::Other, "Prohibited path"),
            path,
        ));
    }

    if file_type != FileType::Symlink {
        let full_permissions = Permissions::from_mode(0o777);
        fs::set_permissions(path, full_permissions).or_else(|e| Err(Error::with_path(e, path)))?;
    }

    match file_type {
        FileType::Dir => delete_folder(path)?,
        FileType::File => delete_file(path).or_else(|e| Err(Error::with_path(e, path)))?,
        FileType::Symlink => delete_symlink(path).or_else(|e| Err(Error::with_path(e, path)))?,
    }
    Ok(())
}

fn delete_folder(folderpath: &Path) -> Result<()> {
    let dir_entries = fs::read_dir(folderpath).or_else(|e| Err(Error::with_path(e, folderpath)))?;
    for entry in dir_entries {
        let entry = entry.or_else(|e| Err(Error::with_path(e, folderpath)))?;
        let path = folderpath.join(entry.file_name());
        total_delete(&path)?;
    }

    print!("Deleting {}... ", folderpath.display());
    stdout().flush().unwrap();

    remove_dir(folderpath).or_else(|e| Err(Error::with_path(e, folderpath)))?;

    println!("Done");
    Ok(())
}

fn delete_file(filepath: &Path) -> io::Result<()> {
    const CLEAR_LINE: &str = "\x1B[2K\r";
    let mut file = File::options().read(true).write(true).open(&filepath)?;
    let file_size = file.metadata()?.len() as usize;
    let mut writen_len: usize = 0;
    let title = format!("Deleting {}...", filepath.display());

    let mut out = stdout().lock();

    write!(out, "{} 00.00%", title).unwrap();
    out.flush().unwrap();

    while writen_len < file_size {
        let remain_len = file_size - writen_len;
        if remain_len < MEBIBYTE {
            writen_len += file.write(&vec![0; remain_len])?;
        } else {
            writen_len += file.write(&MIB_NULL_DATA)?;
        }

        let progress_str = {
            let progress = 100.0 * writen_len as f32 / file_size as f32;
            format!("{:05.2}%", progress)
        };

        write!(out, "{}{} {}", CLEAR_LINE, title, progress_str).unwrap();
        out.flush().unwrap();
    }

    write!(out, "{}{} Done\n", CLEAR_LINE, title).unwrap();

    file.flush()?;
    fs::remove_file(&filepath)?;

    Ok(())
}

fn delete_symlink(path: &Path) -> io::Result<()> {
    print!("Deleting {}... ", path.display());
    stdout().flush().unwrap();

    fs::remove_file(path)?;

    println!("Done");
    Ok(())
}
