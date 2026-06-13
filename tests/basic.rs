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

#[test]
fn unit_propagation_assigns() {
    let mut solver = Solver::new();
    // unit clause forces var1 = true
    solver.add_clause(&[1]);
    // clause (-1 v 2) becomes unit => var2 = true
    solver.add_clause(&[-1, 2]);
    assert_eq!(solver.solve().unwrap(), SolveResult::Sat);
    assert_eq!(solver.value_of(1), Some(true));
    assert_eq!(solver.value_of(2), Some(true));
}

#[test]
fn search_result() {
    let mut solver = Solver::new();
    solver.add_clause(&[-1, -2]);
    solver.add_clause(&[2, 3]);
    solver.add_clause(&[1, -2, 3]);
    solver.add_clause(&[1, -3]);
    assert_eq!(solver.solve().unwrap(), SolveResult::Sat);
    assert_eq!(solver.value_of(1), Some(true));
    assert_eq!(solver.value_of(2), Some(false));
    assert_eq!(solver.value_of(3), Some(true));
}