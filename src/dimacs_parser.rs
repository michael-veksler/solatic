use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::solver::{Lit, Solver, MAX_VAR};

pub fn open(path: impl AsRef<Path>) -> Result<Solver> {
    let file = File::open(&path).with_context(|| format!("{}:", path.as_ref().display()))?;
    from_reader(BufReader::new(file))
}

pub fn from_reader(reader: impl BufRead) -> Result<Solver> {
    let mut solver = Solver::new(1);

    for (idx, line_maybe) in reader.lines().enumerate() {
        let line = line_maybe.with_context(|| format!("{}: ", idx + 1))?;
        match parse_line(line)? {
            Some(clause) => {
                solver
                    .add_clause(&clause)
                    .ok_or(anyhow!("{}: clause trivially UNSAT", idx + 1))?;
            }
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
    let clause: Vec<Lit> = line
        .split_whitespace()
        .take_while(|&s| s != "0")
        .map(|s| {
            s.parse::<i32>().context("invalid literal ").and_then(|i| {
                if i.unsigned_abs() as usize > MAX_VAR {
                    Err(anyhow!("Out of bounds"))
                } else {
                    Ok(Lit::new(i.unsigned_abs() as usize - 1, i < 0))
                }
            })
        })
        .collect::<Result<_>>()?;

    Ok(Some(clause))
}
