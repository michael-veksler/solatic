use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::solver::{Lit, Solver};

pub fn open(path: impl AsRef<Path>) -> Result<Solver> {
    let file = File::open(&path).with_context(|| format!("{}:", path.as_ref().display()))?;
    from_reader(BufReader::new(file))
}

pub fn from_reader(reader: impl BufRead) -> Result<Solver> {
    let mut solver = Solver::new();

    for (idx, line_maybe) in reader.lines().enumerate() {
        let line = line_maybe.with_context(|| format!("{}: ", idx + 1))?;
        match parse_line(line)? {
            Some(clause) => solver
                .add_clause(&clause, 0usize)
                .ok_or(anyhow!("{}: clause trivially UNSAT", idx + 1))?,
            None => continue,
        }
    }
    Ok(solver)
}

fn parse_line(line: String) -> Result<Option<Vec<Lit>>> {
    let line = line.trim();
    if line.starts_with('c') || line.starts_with('p') || line.is_empty() {
        return Ok(None); // skip comments and problem line
                         // we skip problem line because we don't need to know the number of variables or clauses in advance
    }
    let mut clause: Vec<i32> = line
        .split_whitespace()
        .map(|s| s.parse::<Lit>().context("invalid literal "))
        .collect::<Result<_>>()?;

    match clause.pop() {
        None => panic!("Got an empty clause"),
        Some(last) => {
            if last != 0 {
                panic!("Last literal in DIMACS should be 0, not {}", last);
            }
        }
    }
    Ok(Some(clause))
}
