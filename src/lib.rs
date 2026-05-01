mod file_type;

use file_type::FileType;

use std::fs::{self, File, canonicalize, remove_dir};
use std::io::{self, Write, stdout};
use std::path::{Component, Path, PathBuf};
use std::result;

type Result<T> = result::Result<T, (Option<String>, io::Error)>;

enum ProgramFlow {
    Continue,
    Abort,
}

const MEBIBYTE: usize = 1024 * 1024;
static MIB_NULL_DATA: [u8; MEBIBYTE] = [0; MEBIBYTE];
const CLEAR_LINE: &str = "\x1B[2K\r";

pub fn run(file_paths: &[String]) -> Result<()> {
    if file_paths.len() == 0 {
        return Err((None, io::Error::other("No arguments provided")));
    }

    let abs_paths = get_abs_paths(file_paths)?;

    for path in &abs_paths {
        let opt_path = Some(path.to_string_lossy().to_string());
        let file_type = FileType::new(path).or_else(|e| Err((opt_path, e)))?;
        println!("- {} ({})", path.display(), file_type);
    }

    match get_confirmation() {
        Ok(ProgramFlow::Continue) => (),
        Ok(ProgramFlow::Abort) => {
            println!("Aborting...");
            return Ok(());
        }
        Err(e) => return Err(e),
    }

    for path in &abs_paths {
        let opt_path = Some(path.to_string_lossy().to_string());
        total_delete(path).or_else(|e| Err((opt_path, e)))?;
    }

    Ok(())
}

fn get_abs_paths(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut abs_paths: Vec<PathBuf> = Vec::with_capacity(paths.len());
    for path in paths {
        let opt_path = Some(path.clone());
        let path = Path::new(path);
        let abs_path = if path.is_symlink() {
            get_symlink_abs_path(path)
        } else {
            canonicalize(path).or_else(|err| Err((opt_path, err)))?
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

fn get_confirmation() -> Result<ProgramFlow> {
    let mut atempts: u8 = 0;
    loop {
        if atempts >= 3 {
            return Ok(ProgramFlow::Abort);
        }

        print!("Are you sure you want to permanently delete these files? [y/n] ");
        io::stdout().flush().or_else(|e| Err((None, e)))?;

        let ans = {
            let mut temp = String::new();
            io::stdin()
                .read_line(&mut temp)
                .or_else(|e| Err((None, e)))?;
            temp.trim().to_ascii_lowercase()
        };

        match ans.as_str() {
            "no" | "n" => return Ok(ProgramFlow::Abort),
            "yes" | "ye" | "y" => return Ok(ProgramFlow::Continue),
            _ => println!("Invalid answer"),
        }

        atempts += 1;
    }
}

fn total_delete(path: &Path) -> io::Result<()> {
    let file_type = FileType::new(path)?;
    match file_type {
        FileType::Dir => delete_folder(path)?,
        FileType::File => delete_file(path)?,
        FileType::Symlink => delete_symlink(path)?,
    }
    Ok(())
}

fn delete_folder(folderpath: &Path) -> io::Result<()> {
    for entry in fs::read_dir(folderpath)? {
        let entry = entry?;
        let path = folderpath.join(entry.file_name());
        total_delete(&path)?;
    }

    print!("Deleting {}... ", folderpath.display());
    stdout().flush().unwrap();

    remove_dir(folderpath)?;

    println!("Done");
    Ok(())
}

fn delete_file(filepath: &Path) -> io::Result<()> {
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
