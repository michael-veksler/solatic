use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::args::{SolverArgs};
use crate::solver::{Lit, Solver};
use crate::{dprintln_once, dprintln};

pub fn open(args: &SolverArgs, path: impl AsRef<Path>) -> Result<Solver> {
    let file = File::open(&path).with_context(|| format!("{}:", path.as_ref().display()))?;
    from_reader(args, BufReader::new(file))
}

struct CnfProblemLine {
    num_vars: usize,
    num_clauses: usize,
}

fn read_problem_line<I>(mut lines: I) -> Result<CnfProblemLine>
where
    I: Iterator<Item = (usize, Result<String, std::io::Error>)>,
{
    for (idx, line_maybe) in &mut lines {
        let line = line_maybe.with_context(|| format!("{}: ", idx + 1))?;
        if line.starts_with('c') {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('p') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 4 || parts[1] != "cnf" {
                return Err(anyhow!("{}: invalid problem line", idx + 1));
            }
            let num_vars = parts[2].parse::<usize>().context("has invalid integer value for number of variables")?;
            let num_clauses = parts[3].parse::<usize>().context("has invalid integer value for number of clauses")?;
            return Ok(CnfProblemLine { num_vars, num_clauses });
        }
    }
    Err(anyhow!("missing problem line"))
}

pub fn from_reader(args: &SolverArgs, reader: impl BufRead) -> Result<Solver> {
    let mut solver = Solver::new(args, 1);

    let mut read_iter = reader.lines().enumerate();
    let cnf_problem = read_problem_line(read_iter.by_ref())?;
    let mut actual_problem = CnfProblemLine { num_vars: cnf_problem.num_vars, num_clauses: 0 };
    let mut lits: Vec<Lit> = Vec::new();

    let mut add_clause = |lits: &[Lit], lineno: usize| -> Result<()> {
            solver.add_clause(&lits).ok_or_else(|| anyhow!("{}: clause trivially UNSAT", lineno))?;
            actual_problem.num_clauses += 1;
            Ok(())
    };
    let mut total_lines = 0usize;
    for (idx, line_maybe) in &mut read_iter {
        total_lines = idx + 1;
        let line = line_maybe.with_context(|| format!("{}: ", idx + 1))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("c") {
            dprintln_once!(args.verbose, "{}: WARNING: nonstandard comments after the beginning of the file", idx + 1);
            continue;
        }
        for token in line.split_whitespace() {
            let num = token.parse::<i32>().with_context(|| format!("{}: Expecting integer not {token:?}", idx + 1))?;
            actual_problem.num_vars = actual_problem.num_vars.max(num.unsigned_abs() as usize + 1);

            if num == 0 {
                add_clause(&lits, idx + 1)?;
                lits.clear();
            } else {
                lits.push(Lit::new(num.unsigned_abs() as usize - 1, num < 0));
            }
        }
    }
    if !lits.is_empty() {
        dprintln!(args.verbose, "{total_lines}: WARNING: Last clause lacks terminating 0");
        add_clause(&lits, total_lines)?;
        lits.clear();
    }
    dprintln!(args.verbose && cnf_problem.num_clauses != actual_problem.num_clauses,
              "WARNING: Expected {} clauses but found {}", cnf_problem.num_clauses, actual_problem.num_clauses);
    dprintln!(args.verbose && cnf_problem.num_vars != actual_problem.num_vars,
              "{total_lines}: WARNING: Expected {} variables, found {}", cnf_problem.num_vars, actual_problem.num_vars);
    Ok(solver)
}
