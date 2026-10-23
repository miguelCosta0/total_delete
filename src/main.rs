use std::env;

use total_delete::error::Error;

fn main() {
    let args = env::args().collect::<Vec<String>>()[1..].to_vec();

    if let Err(Error { error, path }) = total_delete::run(&args) {
        match path {
            Some(path) => println!("ERROR at \"{}\": {:?}", path, error),
            None => println!("ERROR: {}", error),
        }
    };
}
