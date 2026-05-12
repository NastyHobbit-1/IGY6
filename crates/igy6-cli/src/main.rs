use std::env;
use std::process;

fn main() {
    match igy6_cli::run_cli(env::args().skip(1)) {
        Ok(output) => {
            print!("{output}");
        }
        Err(error) => {
            eprintln!("ERROR: {error}");
            process::exit(2);
        }
    }
}
