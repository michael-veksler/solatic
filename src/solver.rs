use bitflags::bitflags;
use std::io;
use std::ops::{Index, IndexMut};

pub type Lit = i32;
pub type ClauseId = u32;

#[derive(Debug, Default)]
pub struct ClauseAccessor {
    begin: usize,
    post_end: usize,
}

impl ClauseAccessor {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.post_end - self.begin
    }
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.post_end == self.begin
    }
}

#[derive(Debug, Default)]
pub struct ClauseDb {
    pool: Vec<Lit>,
    offsets: Vec<usize>,
}

impl ClauseDb {
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    pub fn push(&mut self, lits: &[Lit]) -> ClauseId {
        let clause_id = self.offsets.len() as ClauseId;
        self.offsets.push(self.pool.len());
        let stored_size: Lit = lits.len().try_into().expect("clause too long");
        self.pool.push(stored_size);
        self.pool.extend_from_slice(lits);
        clause_id
    }

    pub fn get(&self, id: ClauseId) -> ClauseAccessor {
        let header_pos = self.offsets[id as usize];
        let size = self.pool[header_pos] as usize;
        let begin = header_pos + 1;
        ClauseAccessor {
            begin,
            post_end: begin + size,
        }
    }

    pub fn literals(&self, clause: ClauseAccessor) -> &[Lit] {
        &self.pool[clause.begin..clause.post_end]
    }
    #[allow(dead_code)]
    pub fn literals_mut(&mut self, clause: ClauseAccessor) -> &mut [Lit] {
        &mut self.pool[clause.begin..clause.post_end]
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Assignment : u8 {
        const NEGATIVE = 1 << 0;
        const POSITIVE = 1 << 1;
        const UNASSIGNED = Self::NEGATIVE.bits() | Self::POSITIVE.bits();
    }
}

impl Assignment {
    pub fn is_unassigned(self) -> bool {
        self == (Assignment::NEGATIVE | Assignment::POSITIVE)
    }
    pub fn negated(self) -> Assignment {
        let has_positive = self.contains(Assignment::POSITIVE);
        let has_negative = self.contains(Assignment::NEGATIVE);
        if has_negative == has_positive {
            self
        } else {
            self ^ Assignment::UNASSIGNED
        }
    }
}
impl From<bool> for Assignment {
    fn from(b: bool) -> Self {
        if b {
            Assignment::POSITIVE
        } else {
            Assignment::NEGATIVE
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolveResult {
    Sat,
    Unsat,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Watcher {
    clause: ClauseId,
    blocking_literal: Lit,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct WatchersDb {
    // FIXME: Vec<Watch> is not cache friendly.
    //        Better have the size and capacity in the same memory block with the first Watch.
    //        Maybe use unsafe code to manage pointers, or use thin-vec  or thin-dst / erasable
    pos_watchers: Vec<Vec<Watcher>>,
    neg_watchers: Vec<Vec<Watcher>>,
}

#[allow(dead_code)]
impl WatchersDb {
    pub fn add_watch(&mut self, lit: Lit, watch: Watcher) {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) {
            &mut self.pos_watchers
        } else {
            &mut self.neg_watchers
        };
        if var >= watchers.len() {
            watchers.resize(var + 1, Vec::new());
        }
        watchers[var].push(watch);
    }
}

impl Index<Lit> for WatchersDb {
    type Output = Vec<Watcher>;
    fn index(&self, lit: Lit) -> &Self::Output {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) {
            &self.pos_watchers
        } else {
            &self.neg_watchers
        };
        if let Some(watches) = watchers.get(var) {
            watches
        } else {
            static EMPTY_WATCH: Vec<Watcher> = Vec::new();
            &EMPTY_WATCH
        }
    }
}

impl IndexMut<Lit> for WatchersDb {
    fn index_mut(&mut self, lit: Lit) -> &mut Self::Output {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) {
            &mut self.pos_watchers
        } else {
            &mut self.neg_watchers
        };
        if var >= watchers.len() {
            watchers.resize(var + 1, Vec::new());
        }
        &mut watchers[var]
    }
}

/// Return the variable id for a literal.
fn var_of(lit: Lit) -> Option<usize> {
    if lit == 0 {
        None
    } else {
        Some(lit.unsigned_abs() as usize)
    }
}

/// Return true when a literal is positive.
fn is_pos(lit: Lit) -> bool {
    lit > 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropagationResult {
    Conflict,
    Unchanged,
    Propagated,
}

impl PropagationResult {
    pub fn to_option(self) -> Option<()> {
        if self == PropagationResult::Conflict {
            None
        } else {
            Some(())
        }
    }
}

#[derive(Default)]
pub struct Solver {
    clauses: ClauseDb,
    assigns: Vec<Assignment>,
    trail: Vec<Lit>,
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_vars(&mut self, var: usize) {
        if var >= self.assigns.len() {
            self.assigns
                .resize(var + 1, Assignment::POSITIVE | Assignment::NEGATIVE);
        }
    }

    pub fn add_clause(&mut self, lits: &[Lit]) {
        let opt_max_var: Option<usize> = lits.iter().map(|&lit| var_of(lit).unwrap_or(0)).max();
        if let Some(max_var) = opt_max_var {
            self.ensure_vars(max_var);
        }
        self.clauses.push(lits);
    }

    fn literal_state(&self, lit: Lit) -> Assignment {
        let var = var_of(lit).expect("invalid 0 literal");
        let assignment = self.assigns[var];
        if is_pos(lit) {
            assignment
        } else {
            assignment.negated()
        }
    }

    fn find_first_unassigned_var(&self, start: usize) -> Option<usize> {
        (start..self.assigns.len()).find(|&i| self.assigns[i].is_unassigned())
    }

    fn propagate_clause(&mut self, clause_id: ClauseId) -> PropagationResult {
        let mut free_literal: Option<Lit> = None;
        for lit in self.clauses.literals(self.clauses.get(clause_id)) {
            let state = self.literal_state(*lit);
            if state == Assignment::POSITIVE {
                return PropagationResult::Unchanged;
            }
            if state.is_unassigned() {
                if free_literal.is_some() {
                    return PropagationResult::Unchanged; // Nothing to do with 2 free literals
                }
                free_literal = Some(*lit);
            }
        }
        match free_literal {
            None => PropagationResult::Conflict,
            Some(free) => {
                self.set_literal(free);
                PropagationResult::Propagated
            }
        }
    }

    fn set_literal(&mut self, lit: Lit) {
        let var = var_of(lit).expect("invalid literal");

        self.assigns[var] = Assignment::from(is_pos(lit));
        self.trail.push(lit);
    }
    fn bcp(&mut self) -> Option<()> {
        let mut stable: bool = false;
        while !stable {
            stable = true;
            for clause_id in 0..self.clauses.len() {
                match self.propagate_clause(clause_id as ClauseId) {
                    PropagationResult::Conflict => return None,
                    PropagationResult::Propagated => {
                        stable = false;
                    }
                    PropagationResult::Unchanged => (),
                }
            }
        }

        (0..self.clauses.len()).try_for_each(|clause_id| self.propagate_clause(clause_id as ClauseId).to_option())
    }
    fn initial_propagate(&mut self) -> Option<()> {
        (0..self.clauses.len()).try_for_each(|clause_id| self.propagate_clause(clause_id as ClauseId).to_option())
    }

    fn make_decision(&mut self, decisions: &mut Vec<Lit>) -> Option<()> {
        let choice = self.find_first_unassigned_var(1).map(|unassigned| unassigned as Lit)?;
        decisions.push(choice);
        self.set_literal(choice);
        Some(())
    }

    fn backtrack(&mut self, decisions: &mut Vec<Lit>) -> Option<Lit> {
        let decision_lit = decisions.pop()?;
        while let Some(assigned_lit) = self.trail.pop() {
            let assigned_var = var_of(assigned_lit).expect("invalid literal");
            self.assigns[assigned_var] = Assignment::UNASSIGNED;
            if assigned_lit == decision_lit {
                break;
            }
        }
        Some(decision_lit)
    }
    fn solve_loop(&mut self) -> Option<()> {
        let mut decisions: Vec<Lit> = Vec::new();
        loop {
            let mut success = self.bcp();
            if success.is_some() {
                if self.make_decision(&mut decisions).is_none() {
                    return Some(());
                }
                success = self.bcp();
            }
            if success.is_some() {
                continue;
            }
            let conflict_lit = self.backtrack(&mut decisions)?;
            self.set_literal(-conflict_lit);
        }
    }

    pub fn values_len(&self) -> usize {
        self.assigns.len()
    }
    /// Query the current assignment of a variable.
    pub fn value_of(&self, var: usize) -> Option<bool> {
        if var >= self.assigns.len() {
            None
        } else {
            Some(self.assigns[var] == Assignment::POSITIVE)
        }
    }
    pub fn write_assignments(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        for var in 1..self.values_len() {
            write!(writer, "V{var}=")?;
            match self.value_of(var) {
                None => writeln!(writer, "unset")?,
                Some(val) => writeln!(writer, "{}", val as u32)?,
            }
        }
        Ok(())
    }
    pub fn solve_and_write(&mut self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self.solve() {
            SolveResult::Unsat => {
                writeln!(writer, "UNSAT")
            }
            SolveResult::Sat => {
                writeln!(writer, "SAT")?;
                self.write_assignments(writer)
            }
        }
    }

    pub fn solve(&mut self) -> SolveResult {
        if self.initial_propagate().and_then(|_| self.solve_loop()).is_some() {
            SolveResult::Sat
        } else {
            SolveResult::Unsat
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_db() {
        let mut db = ClauseDb::default();
        let cl0 = [1, 2, 3, 4];
        let cl1 = [-1, -2, -3];
        let cl2 = [];
        let cl3 = [-5];
        db.push(cl0.as_slice());
        db.push(cl1.as_slice());
        db.push(cl2.as_slice());
        db.push(cl3.as_slice());
        assert_eq!(db.literals(db.get(0)), &cl0);
        assert_eq!(db.literals(db.get(1)), &cl1);
        assert_eq!(db.literals(db.get(2)), &cl2);
        assert_eq!(db.literals(db.get(3)), &cl3);

        let c4 = [5, 6];
        db.push(c4.as_slice());
        assert_eq!(db.literals(db.get(4)), &c4);
        assert!(db.literals(db.get(2)).is_empty());
        assert!(!db.literals(db.get(0)).is_empty());

        db.literals_mut(db.get(4))[0] = 8;
        assert_eq!(db.literals(db.get(4)), &[8, 6]);
    }

    #[test]
    fn test_watch_db() {
        let mut db = WatchersDb::default();
        assert!(db[1].is_empty());
        assert!(db[-1].is_empty());

        db.add_watch(
            1,
            Watcher {
                clause: 0,
                blocking_literal: 1,
            },
        );
        db.add_watch(
            -1,
            Watcher {
                clause: 1,
                blocking_literal: -1,
            },
        );
        db.add_watch(
            1,
            Watcher {
                clause: 2,
                blocking_literal: 3,
            },
        );

        assert_eq!(
            db[1],
            &[
                Watcher {
                    clause: 0,
                    blocking_literal: 1
                },
                Watcher {
                    clause: 2,
                    blocking_literal: 3
                }
            ]
        );
        assert_eq!(
            db[-1],
            &[Watcher {
                clause: 1,
                blocking_literal: -1
            }]
        );

        db[1].push(Watcher {
            clause: 3,
            blocking_literal: -8,
        });
        assert_eq!(
            db[1],
            &[
                Watcher {
                    clause: 0,
                    blocking_literal: 1
                },
                Watcher {
                    clause: 2,
                    blocking_literal: 3
                },
                Watcher {
                    clause: 3,
                    blocking_literal: -8
                }
            ]
        );
    }
}
