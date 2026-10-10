use clap::Parser;
use solatic::args::SolverArgs;
use solatic::dimacs_parser::open;
use std::error::Error;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "SAT solver that reads DIMACS files")]
struct DimacsArgs {
    #[command(flatten)]
    solver_args: SolverArgs,

    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = DimacsArgs::parse();

    let mut solver = open(&args.solver_args, &args.file).expect("failed to parse DIMACS file");
    let mut stdout = io::stdout();
    solver.solve_and_write(&mut stdout)?;

    Ok(())
}
