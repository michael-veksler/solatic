use solatic::{Solver, SolveResult};

#[test]
fn smoke_test() {
    let mut solver = Solver::new();
    solver.add_clause([1, -2, 3].as_slice());
    assert_eq!(solver.solve(), SolveResult::Sat);
}

#[test]
fn empty_clause_is_unsat() {
    let mut solver = Solver::new();
    solver.add_clause([].as_slice());
    assert_eq!(solver.solve(), SolveResult::Unsat);
}

#[test]
fn nontrivial_sat() {
    let mut solver = Solver::new();

    // first we create mutual implication v1 -> !v2 -> v3 -> v1
    // Remember the rule (a->b) is equivalent to (!a || b)
    solver.add_clause(&[-1, -2]);  // v1 -> !v2
    solver.add_clause(&[2, 3]);    // !v2 -> v3
    solver.add_clause(&[-3, 1]);   // v3 -> v1

    // now make it impossible to assign !v1, v2, !v3, leaving only v1, !v2, v3 as a solution
    solver.add_clause(&[1, -2, 3]);

    assert_eq!(solver.solve(), SolveResult::Sat);
    assert_eq!(solver.value_of(1), Some(true));
    assert_eq!(solver.value_of(2), Some(false));
    assert_eq!(solver.value_of(3), Some(true));
}

#[test]
fn conflict_is_unsat() {
    let mut solver = Solver::new();

    // first we create mutual implication v1 -> !v2 -> v3 -> v1
    // Remember the rule (a->b) is equivalent to (!a || b)
    solver.add_clause(&[-1, -2]);  // v1 -> !v2
    solver.add_clause(&[2, 3]);    // !v2 -> v3
    solver.add_clause(&[-3, 1]);   // v3 -> v1

    // now make it impossible to assign !v1, v2, !v3, leaving only v1, !v2, v3 as a solution
    solver.add_clause(&[1, -2, 3]);

    // and finally forbid the only viable solution
    solver.add_clause(&[-1, 2, -3]);

    assert_eq!(solver.solve(), SolveResult::Unsat);
}