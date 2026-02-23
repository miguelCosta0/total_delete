use std::fs::{self, File, canonicalize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::result;

type Result<T> = result::Result<T, (Option<String>, io::Error)>;

enum ProgramFlow {
    Continue,
    Abort,
}

const MEBIBYTE: usize = 1024 * 1024;
static MIB_NULL_DATA: [u8; MEBIBYTE] = [0; MEBIBYTE];
const BACKSPACE: &str = "\x08";

pub fn run(file_paths: &[String]) -> Result<()> {
    let abs_paths = get_abs_paths(file_paths)?;

    for path in &abs_paths {
        println!("- {}", path.to_str().unwrap())
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
        let opt_path = Some(
            path.to_string_lossy()
                .to_string(),
        );
        let file_type = fs::metadata(path)
            .or_else(|e| Err((opt_path.clone(), e)))?
            .file_type();
        if file_type.is_dir() {
            return Err((opt_path, io::Error::other("Cannot remove directory")));
        } else {
            total_delete_file(path).or_else(|e| Err((opt_path, e)))?;
        }
    }

    Ok(())
}

fn get_abs_paths(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut abs_paths: Vec<PathBuf> = Vec::with_capacity(paths.len());
    for path in paths {
        let abspath = canonicalize(path).or_else(|err| Err((Some(path.to_string()), err)))?;
        abs_paths.push(abspath);
    }
    Ok(abs_paths)
}

fn get_confirmation() -> Result<ProgramFlow> {
    let mut atempts: u8 = 0;
    loop {
        if atempts >= 3 {
            return Ok(ProgramFlow::Abort);
        }

        print!("Are you sure you want to permanently delete these files? [y/n] ");
        io::stdout()
            .flush()
            .or_else(|e| Err((None, e)))?;

        let mut ans = String::new();
        io::stdin()
            .read_line(&mut ans)
            .or_else(|e| Err((None, e)))?;
        ans.pop();
        ans.make_ascii_lowercase();

        match ans.as_str() {
            "no" | "n" => return Ok(ProgramFlow::Abort),
            "yes" | "ye" | "y" => return Ok(ProgramFlow::Continue),
            _ => println!("Invalid answer"),
        }

        atempts += 1;
    }
}

fn total_delete_file(filepath: &Path) -> io::Result<()> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .open(&filepath)?;
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
        if remain_len < MEBIBYTE {
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
