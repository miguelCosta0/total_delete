use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>()[1..].to_vec();

    if let Err((path, error)) = total_delete::run(&args) {
        match path {
            Some(path) => println!("ERROR at \"{}\": {}", path, error.kind()),
            None => println!("ERROR: {}", error),
        }
    };
}
