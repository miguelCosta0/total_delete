use std::fs::{self, File, read_to_string};
use std::io::{self, Read, Seek};
use std::path::Path;
use std::{array, env};
// use total_delete::total_delete_file;

// OBS THIS WONT FOLLOW SYMLINKSL

fn main() {
    let args: Vec<String> = env::args().collect();
    let Err(error) = total_delete::run(&args) else {
        return;
    };
    match error.kind() {
        io::ErrorKind::NotFound => {
            panic!("file not found");
        }
        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }
        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }

        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }

        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }

        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }

        io::ErrorKind::PermissionDenied => {
            panic!("permission denied");
        }
    }
}
