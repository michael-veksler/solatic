use clap::Parser;
use solatic::args::SolverArgs;
use solatic::dimacs_parser::open;
use std::error::Error;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use rstest::rstest;
    use solatic::args::LiteralChoice;

    #[rstest]
    #[case(vec!["solatic", "file.cnf"], Ok(DimacsArgs { file: PathBuf::from("file.cnf"), solver_args: SolverArgs{ verbose: false, choose_literal: LiteralChoice::False }}))]
    #[case(vec!["solatic", "--verbose", "file.cnf"], Ok(DimacsArgs { file: PathBuf::from("file.cnf"), solver_args: SolverArgs{ verbose: true, choose_literal: LiteralChoice::False }}))]
    #[case(vec!["solatic", "--verbose", "--choose-literal", "true", "file.cnf"],
           Ok(DimacsArgs { file: PathBuf::from("file.cnf"), solver_args: SolverArgs{ verbose: true, choose_literal: LiteralChoice::True }}))]
    #[case(vec!["solatic", "--choose-literal", "false", "file.cnf"],
           Ok(DimacsArgs { file: PathBuf::from("file.cnf"), solver_args: SolverArgs{ verbose: false, choose_literal: LiteralChoice::False }}))]
    #[case(vec!["solatic"], Err(anyhow!("bla")))] // Missing file
    fn test_cmdline_parsing(#[case] input: Vec<&str>, #[case] expected: Result<DimacsArgs>) {
        let result = DimacsArgs::try_parse_from(input);

        match (result, expected) {
            (Ok(parsed), Ok(exp)) => assert_eq!(parsed, exp),
            (Err(err), Err(_)) => {
                assert!(err.use_stderr());
            }
            other => panic!("Test case mismatch: {:?}", other),
        }
    }
}
