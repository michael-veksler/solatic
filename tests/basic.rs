use anyhow::anyhow;
use solatic::{to_lits, SolveResult, Solver};

#[test]
fn smoke_test() -> anyhow::Result<()> {
    let mut solver = Solver::new();
    solver
        .add_clause(&to_lits(&[1, -2, 3]))
        .ok_or_else(|| anyhow!("unexpected"))?;
    assert_eq!(solver.solve(), SolveResult::Sat);
    Ok(())
}

#[test]
fn empty_clause_is_unsat() {
    let mut solver = Solver::new();
    if solver.add_clause([].as_slice()).is_none() {
        return;
    }
    assert_eq!(solver.solve(), SolveResult::Unsat);
}

#[test]
fn nontrivial_sat() -> anyhow::Result<()> {
    let mut solver = Solver::new();
    let unexpected = || anyhow!("unexpected");

    // first we create mutual implication v1 -> !v2 -> v3 -> v1
    // Remember the rule (a->b) is equivalent to (!a || b)
    solver.add_clause(&to_lits(&[-1, -2])).ok_or_else(unexpected)?; // v1 -> !v2
    solver.add_clause(&to_lits(&[2, 3])).ok_or_else(unexpected)?; // !v2 -> v3
    solver.add_clause(&to_lits(&[-3, 1])).ok_or_else(unexpected)?; // v3 -> v1

    // now make it impossible to assign !v1, v2, !v3, leaving only v1, !v2, v3 as a solution
    solver.add_clause(&to_lits(&[1, -2, 3])).ok_or_else(unexpected)?;

    assert_eq!(solver.solve(), SolveResult::Sat);
    assert_eq!(solver.value_of(1), Some(true));
    assert_eq!(solver.value_of(2), Some(false));
    assert_eq!(solver.value_of(3), Some(true));
    Ok(())
}

#[test]
fn conflict_is_unsat() -> anyhow::Result<()> {
    let mut solver = Solver::new();
    let unexpected = || anyhow!("unexpected");

    // first we create mutual implication v1 -> !v2 -> v3 -> v1
    // Remember the rule (a->b) is equivalent to (!a || b)
    solver.add_clause(&to_lits(&[-1, -2])).ok_or_else(unexpected)?; // v1 -> !v2
    solver.add_clause(&to_lits(&[2, 3])).ok_or_else(unexpected)?; // !v2 -> v3
    solver.add_clause(&to_lits(&[-3, 1])).ok_or_else(unexpected)?; // v3 -> v1

    // now make it impossible to assign !v1, v2, !v3, leaving only v1, !v2, v3 as a solution
    solver.add_clause(&to_lits(&[1, -2, 3])).ok_or_else(unexpected)?;

    // and finally forbid the only viable solution
    solver.add_clause(&to_lits(&[-1, 2, -3])).ok_or_else(unexpected)?;

    assert_eq!(solver.solve(), SolveResult::Unsat);
    Ok(())
}

#[test]
fn repeated_literals() -> anyhow::Result<()> {
    let mut solver = Solver::new();
    let unexpected = || anyhow!("unexpected");
    solver.add_clause(&to_lits(&[-1, -1, -1])).ok_or_else(unexpected)?;
    solver.add_clause(&to_lits(&[2, -1])).ok_or_else(unexpected)?;
    solver.add_clause(&to_lits(&[1, -3, 3])).ok_or_else(unexpected)?; // tautology
    solver.add_clause(&to_lits(&[2, 3, 3])).ok_or_else(unexpected)?;
    solver.add_clause(&to_lits(&[1, 1, 3])).ok_or_else(unexpected)?;
    solver.add_clause(&to_lits(&[1, -3, -1])).ok_or_else(unexpected)?; // tautology
    assert_eq!(solver.solve(), SolveResult::Sat);
    assert_eq!(solver.value_of(1), Some(false));
    // V2 can be anything
    assert_eq!(solver.value_of(3), Some(true));
    Ok(())
}
