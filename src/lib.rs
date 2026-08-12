pub mod dimacs_parser;
pub mod solver;

pub use solver::{to_lits, ClauseDb, Lit, SolveResult, Solver};
