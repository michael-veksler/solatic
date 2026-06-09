use solatic::{Solver, SolveResult};

#[test]
fn smoke_test() {
    let mut solver = Solver::new();
    solver.add_clause([1, -2, 3].as_slice());
    assert_eq!(solver.solve().unwrap(), SolveResult::Sat);
}

#[test]
fn empty_clause_is_unsat() {
    let mut solver = Solver::new();
    solver.add_clause([].as_slice());
    assert_eq!(solver.solve().unwrap(), SolveResult::Unsat);
}
