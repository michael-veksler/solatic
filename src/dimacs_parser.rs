use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::args::SolverArgs;
use crate::solver::{Lit, Solver, MAX_VAR};
use crate::{dprintln, dprintln_once};

pub fn open(args: &SolverArgs, path: impl AsRef<Path>) -> Result<Solver> {
    let file = File::open(&path).with_context(|| format!("{}:", path.as_ref().display()))?;
    from_reader(args, BufReader::new(file))
}

/// The header of the DIMACS file
#[derive(Default)]
struct CnfProblemLine {
    num_vars: usize,
    num_clauses: usize,
}

#[derive(Default)]
struct DimacsBuilder {
    args: SolverArgs,
    problem_line: CnfProblemLine,
    actual_problem: CnfProblemLine,
    lineno: usize,
    lits: Vec<Lit>,
}

impl DimacsBuilder {
    fn new(args: &SolverArgs) -> Self {
        Self {
            args: *args,
            ..Self::default()
        }
    }

    /// Eat up the comment and empty lines up to the header, then return the header.
    fn read_problem_line<I>(&mut self, mut lines: I) -> Result<()>
    where
        I: Iterator<Item = Result<String, std::io::Error>>,
    {
        for maybe_line in &mut lines {
            self.lineno += 1;
            let line = maybe_line.with_context(|| format!("{}: ", self.lineno))?;
            if line.starts_with('c') {
                continue;
            }
            let line = line.trim();
            if line.starts_with('c') {
                dprintln_once!(
                    self.args.verbose,
                    "{}: Comment line should not start with a whitespace {line:?}",
                    self.lineno
                );
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() || parts[0] != "p" {
                return Err(anyhow!("{}: expected problem line", self.lineno));
            }
            if parts.len() != 4 || parts[1] != "cnf" {
                return Err(anyhow!(
                    "{}: invalid problem line, or unsupported problem type",
                    self.lineno
                ));
            }
            let num_vars = parts[2]
                .parse::<usize>()
                .context("has invalid integer value for number of variables")?;
            let num_clauses = parts[3]
                .parse::<usize>()
                .context("has invalid integer value for number of clauses")?;
            self.problem_line = CnfProblemLine { num_vars, num_clauses };
            self.actual_problem = CnfProblemLine {
                num_vars,
                num_clauses: 0,
            };
            return Ok(());
        }
        Err(anyhow!("missing problem line"))
    }

    pub fn add_clause(&mut self, solver: &mut Solver) -> Result<()> {
        solver
            .add_clause(&self.lits)
            .ok_or_else(|| anyhow!("{}: clause trivially UNSAT", self.lineno))?;
        self.lits.clear();
        self.actual_problem.num_clauses += 1;
        Ok(())
    }

    pub fn read(&mut self, reader: impl BufRead) -> Result<Solver> {
        let mut solver = Solver::new(&self.args, 1);

        let mut read_iter = reader.lines();
        self.read_problem_line(read_iter.by_ref())?;

        for line_maybe in &mut read_iter {
            self.lineno += 1;
            let line = line_maybe.with_context(|| format!("{}: ", self.lineno))?;
            self.parse_line(&mut solver, &line)?;
        }
        if !self.lits.is_empty() {
            dprintln!(
                self.args.verbose,
                "{}: WARNING: Last clause lacks terminating 0",
                self.lineno
            );
            self.add_clause(&mut solver)?;
            self.lits.clear();
        }
        dprintln!(
            self.args.verbose && self.problem_line.num_clauses != self.actual_problem.num_clauses,
            "WARNING: Expected {} clauses but found {}",
            self.problem_line.num_clauses,
            self.actual_problem.num_clauses
        );
        dprintln!(
            self.args.verbose && self.problem_line.num_vars != self.actual_problem.num_vars,
            "{}: WARNING: Expected {} variables, found {}",
            self.lineno,
            self.problem_line.num_vars,
            self.actual_problem.num_vars
        );
        Ok(solver)
    }

    fn parse_line(&mut self, solver: &mut Solver, line: &str) -> Result<()> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }
        if line.starts_with("c") {
            dprintln_once!(
                self.args.verbose,
                "{}: WARNING: nonstandard comments after the beginning of the file",
                self.lineno
            );
            return Ok(());
        }
        for token in line.split_whitespace() {
            let num = token
                .parse::<i32>()
                .with_context(|| format!("{}: Expecting integer not {token:?}", self.lineno))?;
            self.actual_problem.num_vars = self.actual_problem.num_vars.max(num.unsigned_abs() as usize);

            if num == 0 {
                self.add_clause(solver)?;
            } else if num.unsigned_abs() as usize > MAX_VAR {
                return Err(anyhow!("Out of bounds"));
            } else {
                self.lits.push(Lit::new(num.unsigned_abs() as usize - 1, num < 0));
            }
        }
        Ok(())
    }
}

pub fn from_reader(args: &SolverArgs, reader: impl BufRead) -> Result<Solver> {
    let mut builder: DimacsBuilder = DimacsBuilder::new(args);
    builder.read(reader)
}
