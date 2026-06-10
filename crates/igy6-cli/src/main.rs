use std::env;
use std::process;

fn main() {
    match igy6_cli::execute_cli(env::args().skip(1)) {
        Ok(outcome) => {
            print!("{}", outcome.stdout);
            eprint!("{}", outcome.stderr);
            if outcome.exit_code != 0 {
                process::exit(outcome.exit_code);
            }
        }
        Err(error) => {
            eprintln!("ERROR: {error}");
            process::exit(2);
        }
    }
}
