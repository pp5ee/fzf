use fzf::{parse_options, run};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    eprintln!("DEBUG: Starting parse_options");
    match parse_options(&args[1..]) {
        Ok(options) => {
            eprintln!("DEBUG: parse_options succeeded, starting run");
            match run(options) {
                Ok(code) => process::exit(code),
                Err(e) => {
                    eprintln!("fzf: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("fzf: {}", e);
            process::exit(1);
        }
    }
}
