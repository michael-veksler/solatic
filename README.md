# solatic

A small SAT solver written in Rust.

This repository is a focused engineering exercise in implementing a compact SAT solver in Rust. I already have prior experience with CDCL-style solving from work on CSP and constraint reasoning, so the emphasis here is not on learning SAT from scratch. Instead, the project is about building a solver in a way that is simple, readable, and representative of the core ideas used in modern SAT engines.

## Project goals

The goals are intentionally narrow:

1. Implement a fast SAT solver comparable to the original MiniSat, but not more than that.
2. Learn and practice Rust by writing the solver directly rather than relying on AI for the implementation itself.
3. Use the project as a place to experiment with solver engineering decisions and to explore related ideas in constraint reasoning.

I do use AI as a tutor for targeted questions about solver engineering and Rust best practices, but the code itself is meant to be written by hand.

## Who this project is for

This repository is aimed at a general audience of contributors and reviewers, including people who may be evaluating the project as an engineering sample. The README is written to explain the project clearly without pretending that it is a polished production solver.

## Current status

The codebase is still early and the structure may change as the implementation matures.

Implemented or in progress:

- DIMACS CNF parsing is ready.
- Clause storage is implemented.
- Assignment tracking exists, but reason tracking is not yet implemented.
- Basic propagation work is in progress.
- Watched literals are not yet an optimization; they will be added as the solver grows.
- Preprocessing is not planned yet. The focus is to reach a small, understandable CDCL-style core first.

## Quickstart

Build and run the solver with a DIMACS input file:

```bash
cargo run -- tests/fixtures/inputs/SAT-3-vars.cnf
```

Run the test suite:

```bash
cargo test
```

The CLI currently reports whether the input is SAT or UNSAT and, where available, the resulting assignments.

## A short overview of the solver

A SAT instance is typically expressed in conjunctive normal form (CNF): a set of clauses, where each clause is a disjunction of literals. The solver searches for an assignment that satisfies every clause.

In a CDCL-style solver, the main loop is roughly:

- propagate forced assignments,
- branch when necessary,
- detect conflicts,
- learn from conflicts,
- and backtrack.

This project is an attempt to implement that style of solver in a compact and understandable form. It is not intended to be a full production solver, but it does aim to reflect the central engineering ideas that matter in practice.

Two concepts that appear often in practical solvers are:

- unit propagation, where a clause with only one remaining possible literal forces an assignment,
- watched literals, a common technique for making propagation efficient.

For background, these references are useful:

- MiniSat: https://github.com/niklasso/minisat
- The Handbook of Satisfiability: https://www.elsevier.com/books/handbook-of-satisfiability/biere/978-0-12-381479-7
- Watched literals: https://en.wikipedia.org/wiki/Watched_literal

## Broader research direction

Beyond the immediate goal of building a small SAT solver in Rust, this repository may also be used to explore ideas that connect to earlier work in constraint reasoning. In particular, there is interest in experimenting with:

- non-clausal reasoning for some constraint families,
- conditional SAT-style modeling for configuration problems where a variable controls whether a subproblem exists,
- and broader connections between SAT-style solving and CSP-style reasoning.

These ideas are exploratory and will only be pursued once the core solver is in a more mature state.

## Contributing

This is primarily a personal learning and engineering project, but feedback and discussion are welcome. If you are interested in the solver design, the Rust implementation, or the broader direction of the project, pull requests and discussions are appreciated.
