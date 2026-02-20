mod error;

use std::fs::{self, File, canonicalize, read_dir};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::result;

type Result<T> = result::Result<T, error::FileError>;

const MiB: usize = 1024 * 1024;
static MIB_NULL_DATA: [u8; MiB] = [0; MiB];
const BACKSPACE: &str = "\x08";

pub fn run(file_paths: &[String]) -> Result {
    let abs_paths = get_abs_paths(file_paths)?;

    for path in &abs_paths {
        println!("- {}", path.to_str().unwrap())
    }

    println!("Are you sure you want to permanently delete these files? [y/n] ");

    let mut atempts: u8 = 0;
    loop {
        if atempts >= 3 {
            panic!("Aborting...")
        }

        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        ans.make_ascii_lowercase();

        if ans == "yes" || ans == "ye" || ans == "y" {
            break;
        } else if ans == "no" || ans == "n" {
            panic!("Aborting...")
        }

        atempts += 1;
    }

    for path in &abs_paths {
        let ftype = fs::metadata(path)?.file_type();
        if ftype.is_dir() {
            total_delete_dir(path)?;
        } else {
            total_delete_file(path)?;
        }
    }

    Ok(())
}

fn get_abs_paths(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut abs_paths: Vec<PathBuf> = Vec::with_capacity(paths.len());
    for path in paths {
        let abspath =
            canonicalize(path).or_else(|err| Err(error::FileError::new(path.as_ref(), err)))?;
        abs_paths.push(abspath);
    }
    Ok(abs_paths)
}

fn total_delete_dir(dirpath: &Path) -> Result {
    for dir_entry in read_dir(dirpath)? {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        if dir_entry.file_type()?.is_dir() {
            total_delete_dir(&path)?;
        } else {
            total_delete_file(&path)?;
        }
    }

    match (fs::remove_dir(dirpath)) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            io::ErrorKind::DirectoryNotEmpty => {
                println!("ERROR: could not remove {}", dirpath);
                Ok(())
            }
            _ => unknowerror,
        },
    }

    Ok(())
}

fn total_delete_file(filepath: &Path) -> Result {
    let mut file = File::options().read(true).write(true).open(&filepath)?;
    let file_size = file.metadata()?.len() as usize;
    let mut writen_len: usize = 0;

    let mut progress_fmt = String::from("00.00%");
    print!(
        "Deleting {}... {}",
        filepath.to_str().unwrap(),
        progress_fmt
    );

    while writen_len < file_size {
        let remain_len = file_size - writen_len;
        if remain_len < MiB {
            writen_len += file.write(&vec![0; remain_len])?;
        } else {
            writen_len += file.write(&MIB_NULL_DATA)?;
        }

        let progress = 100.0 * writen_len as f32 / file_size as f32;
        let len_old_progress = progress_fmt.len();
        progress_fmt = format!("{:0>5.2}%", progress);

        print!("{}{}", BACKSPACE.repeat(len_old_progress), progress_fmt);
    }

    println!(
        "{}{:width$}",
        BACKSPACE.repeat(progress_fmt.len()),
        "Done",
        width = progress_fmt.len()
    );

    file.flush()?;
    fs::remove_file(&filepath)?;

    Ok(())
}
