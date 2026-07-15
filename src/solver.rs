use bitflags::bitflags;
use std::io;
use std::ops::{Index, IndexMut};

pub type Lit = i32;
pub type ClauseId = u32;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClauseAccessor {
    begin: usize,
    post_end: usize,
}

impl ClauseAccessor {
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

    /// Ensure the falsified literal is stored in the second watched position.
    ///
    /// The clause representation assumes the first two literals are the watched positions.
    /// If the falsified literal is currently in the first watched position, it is swapped
    /// with the second watched position.
    ///
    /// # Panics
    ///
    /// Panics if `falsified_lit` is not one of the first two literals in the clause.
    pub fn ensure_falsified_at_pos1(&mut self, clause: ClauseAccessor, falsified_lit: Lit) -> Option<()> {
        let literals = self.literals_mut(clause);
        if literals[0] != -falsified_lit && literals[1] != -falsified_lit {
            return None;
        }
        if literals[0] == -falsified_lit {
            literals.swap(0, 1);
        }
        Some(())
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

#[derive(Default)]
pub struct Solver {
    clauses: ClauseDb,
    watchers: WatchersDb,
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

    #[must_use]
    pub fn find_satisfiable_literal(&self, clause: ClauseAccessor, first_lit_index: usize) -> Option<usize> {
        self.clauses
            .literals(clause)
            .iter()
            .enumerate()
            .skip(first_lit_index)
            .find_map(|(i, &lit)| {
                let state = self.literal_state(lit);
                if state.contains(Assignment::POSITIVE) {
                    Some(i)
                } else {
                    None
                }
            })
    }

    #[must_use]
    pub fn add_clause(&mut self, lits: &[Lit], decision_level: usize) -> Option<()> {
        // This function has to be rewritten once we have CDCL.
        // Watches should be added differently, when we add clauses on decision level > 0.
        assert!(
            decision_level == 0,
            "adding clauses at decision level > 0 is not supported yet"
        );
        let opt_max_var: Option<usize> = lits.iter().map(|&lit| var_of(lit).unwrap_or(0)).max();
        if let Some(max_var) = opt_max_var {
            self.ensure_vars(max_var);
        }
        match lits.len() {
            0 => return None,
            1 => self.set_literal(lits[0]),
            _ => {
                self.clauses.push(lits);
            }
        };

        Some(())
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

    fn propagate_clause(&mut self, clause_id: ClauseId, clause: ClauseAccessor, falsified_lit: Lit) -> Option<()> {
        assert!(
            clause.len() >= 2,
            "Short clauses should never be entered into the ClauseDB - they are stored as blocking literals only"
        );
        self.clauses.ensure_falsified_at_pos1(clause, falsified_lit);
        let blocking_literal = self.clauses.literals(clause)[0];
        let blocking_state = self.literal_state(blocking_literal);
        if blocking_state == Assignment::POSITIVE {
            return Some(());
        }
        if let Some(nonempty_pos) = self.find_satisfiable_literal(clause, 2) {
            self.clauses.literals_mut(clause).swap(1, nonempty_pos);
            let new_watched_lit = self.clauses.literals(clause)[1];
            self.watchers.add_watch(new_watched_lit, Watcher { clause: clause_id });
            Some(())
        } else if blocking_state == Assignment::NEGATIVE {
            None
        } else {
            self.set_literal(blocking_literal);
            Some(())
        }
    }
    fn initial_propagate_clause(&mut self, clause_id: ClauseId) -> Option<()> {
        let clause = self.clauses.get(clause_id);
        for pos in [1, 0] {
            let lit = self.clauses.literals(clause)[pos];
            if self.literal_state(lit) == Assignment::NEGATIVE {
                self.propagate_clause(clause_id, clause, lit)?;
            } else {
                self.watchers.add_watch(lit, Watcher { clause: clause_id });
            }
        }
        Some(())
    }

    fn set_literal(&mut self, lit: Lit) {
        let var = var_of(lit).expect("invalid literal");

        self.assigns[var] = Assignment::from(is_pos(lit));
        self.trail.push(lit);
    }
    #[must_use]
    fn bcp(&mut self, trail_read_start: usize) -> Option<()> {
        let mut trail_read = trail_read_start;
        while let Some(propagated_lit) = self.trail.get(trail_read) {
            self.propagate_literal(*propagated_lit)?;
            trail_read += 1;
        }
        Some(())
    }
    #[must_use]
    fn propagate_literal(&mut self, set_lit: Lit) -> Option<()> {
        let mut num_written_entries = 0usize;
        let mut read_entry;
        let falsified_lit = -set_lit;
        for i in 0..self.watchers[falsified_lit].len() {
            read_entry = i;
            let watcher = self.watchers[falsified_lit][read_entry];
            let clause = self.clauses.get(watcher.clause);

            if self.propagate_clause(watcher.clause, clause, set_lit).is_none() {
                // Don't remove the read_entry, which was failing
                self.watchers[falsified_lit].drain(num_written_entries..read_entry);
                // PERFORMANCE: Big O complexity-wise, it is better to move elements from the end to fill the gap than just drop()
                //              however, measurements of medium-small problems a simple O(N) drop was slightly faster.

                return None;
            }
            if clause.len() >= 2 {
                let literals = self.clauses.literals(clause);
                if literals[0] != falsified_lit && literals[1] != falsified_lit {
                    continue;
                }
            }
            self.watchers[falsified_lit][num_written_entries] = Watcher { clause: watcher.clause };
            num_written_entries += 1;
        }
        self.watchers[falsified_lit].drain(num_written_entries..);
        Some(())
    }

    #[must_use]
    fn initial_propagate(&mut self) -> Option<()> {
        (0..self.clauses.len()).try_for_each(|clause_id| self.initial_propagate_clause(clause_id as ClauseId))
    }

    #[must_use]
    fn make_decision(&mut self, decisions: &mut Vec<Lit>) -> Option<()> {
        let choice = -self.find_first_unassigned_var(1).map(|unassigned| unassigned as Lit)?;
        decisions.push(choice);
        self.set_literal(choice);
        Some(())
    }

    #[must_use]
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
    #[must_use]
    fn solve_loop(&mut self) -> Option<()> {
        let mut decisions: Vec<Lit> = Vec::new();
        let mut trail_read_pos = 0;
        loop {
            if self.bcp(trail_read_pos).is_some() {
                trail_read_pos = self.trail.len();
                if self.make_decision(&mut decisions).is_none() {
                    return Some(());
                }
                continue;
            }
            let conflict_lit = self.backtrack(&mut decisions)?;
            trail_read_pos = self.trail.len();
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

        db.add_watch(1, Watcher { clause: 0 });
        db.add_watch(-1, Watcher { clause: 1 });
        db.add_watch(1, Watcher { clause: 2 });

        assert_eq!(db[1], &[Watcher { clause: 0 }, Watcher { clause: 2 }]);
        assert_eq!(db[-1], &[Watcher { clause: 1 }]);

        db[1].push(Watcher { clause: 3 });
        assert_eq!(
            db[1],
            &[Watcher { clause: 0 }, Watcher { clause: 2 }, Watcher { clause: 3 }]
        );
    }
}
