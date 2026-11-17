use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>()[1..].to_vec();

    total_delete::run(&args);
}
