mod dimacs_parser;
mod solver;

use crate::dimacs_parser::open;
use std::env;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let mut solver = open(&args[1]).expect("failed to parse DIMACS file");
    let mut stdout = io::stdout();
    solver.solve_and_write(&mut stdout)?;

    Ok(())
}
