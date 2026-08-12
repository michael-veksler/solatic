use bitflags::bitflags;
use std::ops::{Index, IndexMut};
use std::{fmt, io};
#[derive(Clone, Copy, PartialEq, Eq)]

// A variableID with the MSB indicates the sign of the literal (0=positive, 1=negative).
pub struct Lit(u32);

impl Lit {
    const SIGN_POS: usize = size_of::<Lit>() * 8 - 1;
    const SIGN_MASK: u32 = 1 << Lit::SIGN_POS;
    const VAR_MASK: u32 = !Lit::SIGN_MASK;
    const VAR_MAX: u32 = Lit::VAR_MASK;
    pub const fn new(var: usize, is_neg: bool) -> Self {
        const {
            assert!((MAX_VAR >> 31) == 0, "The assert below assumes we allow 31 bits");
        }
        assert!(var <= MAX_VAR, "Variable value must fit 31 bits");
        if is_neg {
            Lit((var as u32) | Lit::SIGN_MASK)
        } else {
            Lit(var as u32)
        }
    }
    fn is_pos(self) -> bool {
        (self.0 & Lit::SIGN_MASK) == 0
    }
    fn var(self) -> usize {
        (self.0 & Lit::VAR_MASK) as usize
    }
}

pub const MAX_VAR: usize = Lit::VAR_MAX as usize;

impl std::ops::Neg for Lit {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Lit(self.0 ^ Lit::SIGN_MASK)
    }
}

impl fmt::Debug for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.is_pos() { "" } else { "-" };
        write!(f, "Lit({sign}{})", self.var())
    }
}

pub type ClauseId = u32;
const NULL_CLAUSE: ClauseId = ClauseId::MAX;

type Reason = ClauseId;

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

    /// Push unique literals into the DB.
    ///
    /// Return: the id of the inserted clause
    ///         NULL_CLAUSE if the clause was a tautology and thus ignored.
    ///         Any other clause, including the empty clause, is returned as a valid clause id.
    pub fn push(&mut self, lits: &[Lit], variables: &mut VariableDb) -> ClauseId {
        let clause_id = self.offsets.len() as ClauseId;
        let clause_start = self.pool.len();
        self.offsets.push(clause_start);

        // This is not logically a Lit, that's why we don't use its new() method.
        let stored_size = Lit(lits.len().try_into().expect("clause too long"));
        self.pool.push(stored_size);
        let literal_start = self.pool.len();
        let mut is_tautology = false;
        for &lit in lits {
            let var = lit.var();
            let next_seen: Assignment = lit.into();
            if variables.get_seen(var) == next_seen {
                continue; // a simple duplicate - to ignore
            }
            if variables.get_seen(var) != Assignment::empty() {
                is_tautology = true;
                break;
            }
            variables.set_seen(var, next_seen);
            self.pool.push(lit);
        }
        for i in literal_start..self.pool.len() {
            variables.reset_seen(self.pool[i].var());
        }
        if is_tautology {
            self.offsets.pop();
            self.pool.truncate(clause_start);
            NULL_CLAUSE
        } else {
            // This is not logically a Lit, that's why we don't use its new() method.
            self.pool[clause_start] = Lit((self.pool.len() - literal_start)
                .try_into()
                .expect("clause too long to store in ClauseDb"));
            clause_id
        }
    }

    fn drop_last_clause(&mut self) {
        if let Some(last_offset) = self.offsets.pop() {
            self.pool.truncate(last_offset);
        }
    }

    pub fn get(&self, id: ClauseId) -> ClauseAccessor {
        let header_pos = self.offsets[id as usize];

        // This is not logically a Lit, that's why we access the underlying data directly
        let size = self.pool[header_pos].0 as usize;
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
    fn from(b: bool) -> Assignment {
        if b {
            Assignment::POSITIVE
        } else {
            Assignment::NEGATIVE
        }
    }
}
impl From<Lit> for Assignment {
    fn from(lit: Lit) -> Assignment {
        if lit.is_pos() {
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
pub struct Watcher {
    clause: ClauseId,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct WatchersDb {
    // FIXME: Vec<Watch> is not cache friendly.
    //        Better have the size and capacity in the same memory block with the first Watch.
    //        Maybe use unsafe code to manage pointers, or use thin-vec  or thin-dst / erasable
    pos_watchers: Vec<Vec<Watcher>>,
    neg_watchers: Vec<Vec<Watcher>>,
}

#[allow(dead_code)]
impl WatchersDb {
    pub fn add_watch(&mut self, lit: Lit, watch: Watcher) {
        let watchers = if lit.is_pos() {
            &mut self.pos_watchers
        } else {
            &mut self.neg_watchers
        };
        if lit.var() >= watchers.len() {
            watchers.resize(lit.var() + 1, Vec::new());
        }
        watchers[lit.var()].push(watch);
    }
}

impl Index<Lit> for WatchersDb {
    type Output = Vec<Watcher>;
    fn index(&self, lit: Lit) -> &Self::Output {
        let watchers = if lit.is_pos() {
            &self.pos_watchers
        } else {
            &self.neg_watchers
        };
        if let Some(watches) = watchers.get(lit.var()) {
            watches
        } else {
            static EMPTY_WATCH: Vec<Watcher> = Vec::new();
            &EMPTY_WATCH
        }
    }
}

impl IndexMut<Lit> for WatchersDb {
    fn index_mut(&mut self, lit: Lit) -> &mut Self::Output {
        let watchers = if lit.is_pos() {
            &mut self.pos_watchers
        } else {
            &mut self.neg_watchers
        };
        if lit.var() >= watchers.len() {
            watchers.resize(lit.var() + 1, Vec::new());
        }
        &mut watchers[lit.var()]
    }
}

#[derive(Default)]
pub struct ConflictInfo {
    frontier: Vec<Lit>,
    decision_level: u32,
    num_lit_in_level: usize,
    latest_non_uip: usize,
    latest_non_uip_level: u32,
}

impl ConflictInfo {
    pub fn new_conflict(&mut self, variables: &mut VariableDb, decision_level: u32, conflict_literals: &[Lit]) {
        self.frontier.clear();
        self.decision_level = decision_level;
        self.latest_non_uip_level = 0;
        self.latest_non_uip = 0;
        self.num_lit_in_level = 0;
        for &lit in conflict_literals {
            variables.set_seen(lit.var(), Assignment::from(lit));
            let lit_level = variables.history[lit.var()].level;
            debug_assert!(
                lit_level <= decision_level,
                "conflict literal {lit:?} has level {lit_level} but current level is {decision_level}"
            );
            if lit_level == self.decision_level {
                self.num_lit_in_level += 1;
            } else {
                if self.latest_non_uip_level < lit_level {
                    self.latest_non_uip_level = lit_level;
                    self.latest_non_uip = self.frontier.len();
                }
                self.frontier.push(lit);
            }
        }
    }
    pub fn resolve(&mut self, variables: &mut VariableDb, literals: &[Lit], pivot: Lit) {
        for &clause_lit in literals {
            if clause_lit == pivot {
                self.num_lit_in_level -= 1;
                variables.reset_seen(clause_lit.var());
                continue;
            }
            if !variables.get_seen(clause_lit.var()).is_empty() {
                continue;
            }
            variables.set_seen(clause_lit.var(), Assignment::from(clause_lit));
            let clause_lit_level = variables.history[clause_lit.var()].level;
            if clause_lit_level == self.decision_level {
                self.num_lit_in_level += 1;
                continue;
            }
            if clause_lit_level > self.latest_non_uip_level {
                self.latest_non_uip = self.frontier.len();
                self.latest_non_uip_level = clause_lit_level;
            }
            variables.set_seen(clause_lit.var(), Assignment::from(clause_lit));
            self.frontier.push(clause_lit);
        }
    }
    pub fn find_trail_index_before(&self, variables: &mut VariableDb, trail: &[Lit], trail_index: usize) -> usize {
        // find 1-UIP
        for trail_index in (0..=trail_index).rev() {
            let trail_var = trail[trail_index].var();
            debug_assert!(variables.history[trail_var].level == self.decision_level);
            if !variables.get_seen(trail_var).is_empty() {
                return trail_index;
            }
        }
        0
    }
    pub fn finalize_clause(&mut self, uip_lit: Lit) {
        let last_frontier_index = self.frontier.len();
        self.frontier.push(-uip_lit);
        if last_frontier_index == 0 {
            return;
        }
        self.frontier.swap(0, last_frontier_index);
        if self.latest_non_uip == 0 {
            self.latest_non_uip = last_frontier_index;
        }
        self.latest_non_uip = 1;
        self.frontier.swap(self.latest_non_uip, 1);
    }
    pub fn add_watches(&self, watchers: &mut WatchersDb, clause_id: ClauseId) {
        watchers.add_watch(self.frontier[0], Watcher { clause: clause_id });
        if self.frontier.len() > 1 {
            watchers.add_watch(self.frontier[1], Watcher { clause: clause_id });
        }
    }
}

#[derive(Clone, Copy)]
struct AssignmentHistory {
    reason: Reason,
    level: u32,
}

#[derive(Default)]
pub struct VariableDb {
    values: Vec<Assignment>,
    history: Vec<AssignmentHistory>,
    // Invariant: When not in clause construction, seen_in_clause[i] == Assignment::empty()
    seen_in_clause: Vec<Assignment>, // empty() = not seen
}

impl VariableDb {
    fn ensure_vars(&mut self, var: usize) {
        if var >= self.len() {
            self.values.resize(var + 1, Assignment::POSITIVE | Assignment::NEGATIVE);
            self.history.resize(var + 1, AssignmentHistory { reason: 0, level: 0 });
            self.seen_in_clause.resize(var + 1, Assignment::empty());
        }
    }

    fn get_value(&self, i: usize) -> Assignment {
        self.values[i]
    }
    fn set_value(&mut self, i: usize, value: Assignment) {
        self.values[i] = value;
    }
    fn len(&self) -> usize {
        debug_assert!(self.values.len() == self.history.len());
        self.values.len()
    }
    fn get_seen(&self, var: usize) -> Assignment {
        self.seen_in_clause[var]
    }
    fn set_seen(&mut self, var: usize, seen: Assignment) {
        self.seen_in_clause[var] = seen;
    }
    fn reset_seen(&mut self, var: usize) {
        self.seen_in_clause[var] = Assignment::empty();
    }
    fn reset_seen_literals(&mut self, literals: &[Lit]) {
        for &lit in literals {
            self.reset_seen(lit.var());
        }
    }
}
#[derive(Default)]
pub struct Solver {
    clauses: ClauseDb,
    watchers: WatchersDb,
    variables: VariableDb,
    trail_lim: Vec<usize>, // The trail-indices where decisions were made
    trail: Vec<Lit>,
    conflict_cache: ConflictInfo,
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
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

    /// Add a clause, and on success return its ClauseId
    ///
    /// None means there was a conflict, i.e., an empty clause inserted
    /// Some(NULL_CLAUSE) means that the clause was accounter for without recording a regular clause.
    /// Possible cases for Some(NULL_CLAUSE):For example, if it was propagated immediately to variables' assignment state.
    ///  - The clause was propagated immediately to variables
    ///  - The clause was a tautology, and thus ignored
    #[must_use]
    pub fn add_clause(&mut self, lits: &[Lit]) -> Option<ClauseId> {
        if lits.is_empty() {
            return None;
        }
        let opt_max_var: Option<usize> = lits.iter().map(|&lit| lit.var()).max();
        if let Some(max_var) = opt_max_var {
            self.variables.ensure_vars(max_var);
        }
        let clause_id = self.clauses.push(lits, &mut self.variables);
        if clause_id == NULL_CLAUSE {
            return Some(NULL_CLAUSE);
        }
        let num_lits = self.clauses.get(clause_id).len();
        if num_lits == 1 {
            let lit = self.clauses.literals(self.clauses.get(clause_id))[0];
            self.clauses.drop_last_clause();
            self.set_literal(lit, NULL_CLAUSE);
            Some(NULL_CLAUSE)
        } else {
            Some(clause_id)
        }
    }

    fn literal_state(&self, lit: Lit) -> Assignment {
        let assignment = self.variables.get_value(lit.var());
        if lit.is_pos() {
            assignment
        } else {
            assignment.negated()
        }
    }

    fn find_first_unassigned_var(&self, start: usize) -> Option<usize> {
        (start..self.variables.len()).find(|&i| self.variables.get_value(i).is_unassigned())
    }

    /// Propagate the clause.
    ///
    /// The clause is propagated only if falsified_lit is one of the first 2 literals of the clause.
    /// Otherwise it is simply ignored.
    /// TODO Future optimization: Don't even call this function in this case.
    ///
    /// Return: Some(()) on success, and None on conflict.
    #[must_use]
    fn propagate_clause(&mut self, clause_id: ClauseId, clause: ClauseAccessor, falsified_lit: Lit) -> Option<()> {
        assert!(
            clause.len() >= 2,
            "Short clauses should never be entered into the ClauseDB - they are stored as blocking literals only"
        );
        if self.clauses.ensure_falsified_at_pos1(clause, falsified_lit).is_none() {
            return Some(()); // Triggered by a stale watch, so ignore it.
        }
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
            self.set_literal(blocking_literal, clause_id);
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

    fn set_literal(&mut self, lit: Lit, reason: Reason) {
        self.variables.set_value(lit.var(), Assignment::from(lit.is_pos()));
        self.variables.history[lit.var()] = AssignmentHistory {
            reason,
            level: self.trail_lim.len() as u32,
        };
        self.trail.push(lit);
    }
    /// Propagate all newly modified literals
    ///
    /// If the propagation modifies literals, then also consider them, until all modified literals are propagated.
    ///
    /// Return: None on success, Some(reason) if a conflict was caused by the reason.
    ///         A reason is a clause or another literal.
    #[must_use]
    fn bcp(&mut self, trail_read_start: usize) -> Option<Reason> {
        let mut trail_read = trail_read_start;
        while let Some(set_lit_ref) = self.trail.get(trail_read) {
            let set_lit = *set_lit_ref;
            if let Some(conflict_reason) = self.propagate_literal(set_lit) {
                return Some(conflict_reason);
            }
            trail_read += 1;
        }
        None
    }

    /// Propagate all clauses that watch the effect of setting a literal.
    ///
    /// The passed parameter set_lit indicates which literal was set,
    /// but the watches are for its negation - the falsified literal.
    ///
    /// Return: The reason for the conflict, or None.
    ///         Usually this is the id of the conflicting clause or None if no conflict.
    #[must_use]
    fn propagate_literal(&mut self, set_lit: Lit) -> Option<Reason> {
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

                return Some(watcher.clause);
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
        None
    }

    #[must_use]
    fn initial_propagate(&mut self) -> Option<()> {
        (0..self.clauses.len()).try_for_each(|clause_id| self.initial_propagate_clause(clause_id as ClauseId))
    }

    #[must_use]
    fn make_decision(&mut self) -> Option<()> {
        let choice = -self
            .find_first_unassigned_var(1)
            .map(|unassigned| Lit::new(unassigned, false))?;
        self.trail_lim.push(self.trail.len());
        self.set_literal(choice, NULL_CLAUSE);
        Some(())
    }

    fn backjump(&mut self, level: usize) {
        let target_trail = self.trail_lim[level];
        self.trail_lim.truncate(level);
        while target_trail < self.trail.len() {
            let assigned_lit = self.trail.pop().unwrap();
            self.variables.set_value(assigned_lit.var(), Assignment::UNASSIGNED);
        }
    }

    fn resolve_until_1uip(&mut self, conflict_info: &mut ConflictInfo) -> Lit {
        let mut trail_index = self.trail.len();
        // generate conflict clause
        while conflict_info.num_lit_in_level > 1 {
            debug_assert!(
                trail_index > 0,
                "If all invariants were kept, we should stop before exhausting the trail"
            );
            trail_index -= 1;
            let trail_lit = self.trail[trail_index];
            let trail_var = trail_lit.var();
            if self.variables.get_seen(trail_var).is_empty() {
                continue;
            }
            debug_assert!(self.variables.history[trail_var].level == conflict_info.decision_level);
            let reason = self.variables.history[trail_var].reason;
            assert!(reason != NULL_CLAUSE);
            let clause = self.clauses.get(reason);
            conflict_info.resolve(&mut self.variables, self.clauses.literals(clause), trail_lit);
        }
        debug_assert!(trail_index != 0);
        let trail_uip = conflict_info.find_trail_index_before(&mut self.variables, &self.trail, trail_index - 1);
        debug_assert!(
            trail_uip >= *self.trail_lim.last().unwrap(),
            "1-UIP must be found at the conflicting level"
        );
        self.trail[trail_uip]
    }

    #[must_use]
    fn make_conflict_clause(&mut self, conflicting_clause_id: ClauseId) -> ConflictInfo {
        let mut conflict_info = std::mem::take(&mut self.conflict_cache);

        let conflicting_clause = self.clauses.get(conflicting_clause_id);
        conflict_info.new_conflict(
            &mut self.variables,
            self.trail_lim.len() as u32,
            self.clauses.literals(conflicting_clause),
        );
        debug_assert!(
            !self.trail.is_empty(),
            "If all invariants were kept, the trail should not have been empty"
        );

        let lit_1uip = self.resolve_until_1uip(&mut conflict_info);
        conflict_info.finalize_clause(lit_1uip);
        self.variables.reset_seen_literals(&conflict_info.frontier);
        conflict_info
    }

    #[must_use]
    fn handle_conflict(&mut self, conflicting_clause: ClauseId) -> Option<()> {
        if self.trail_lim.is_empty() {
            return None;
        }
        let conflict_info = self.make_conflict_clause(conflicting_clause);
        if conflict_info.frontier.len() == 1 {
            let uip_lit = conflict_info.frontier[0];
            self.backjump(0);
            self.set_literal(uip_lit, NULL_CLAUSE);
            return Some(());
        }
        let conflict_clause_id = self.add_clause(&conflict_info.frontier)?;
        conflict_info.add_watches(&mut self.watchers, conflict_clause_id);

        let latest_non_uip_lit = conflict_info.frontier[conflict_info.latest_non_uip];
        self.backjump(conflict_info.latest_non_uip_level as usize);
        self.conflict_cache = conflict_info;
        if conflict_clause_id == NULL_CLAUSE {
            return Some(());
        }
        let conflict_clause = self.clauses.get(conflict_clause_id);
        self.propagate_clause(conflict_clause_id, conflict_clause, -latest_non_uip_lit)
    }
    #[must_use]
    fn solve_loop(&mut self) -> Option<()> {
        self.trail_lim.clear();
        let mut trail_read_pos = 0;
        loop {
            if let Some(conflict_reason) = self.bcp(trail_read_pos) {
                self.handle_conflict(conflict_reason)?;
                trail_read_pos = self.trail.len() - 1;
                continue;
            }
            trail_read_pos = self.trail.len();
            if self.make_decision().is_none() {
                return Some(());
            }
        }
    }

    pub fn values_len(&self) -> usize {
        self.variables.len()
    }
    /// Query the current assignment of a variable.
    pub fn value_of(&self, var: usize) -> Option<bool> {
        if var >= self.variables.len() {
            None
        } else {
            Some(self.variables.get_value(var) == Assignment::POSITIVE)
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

#[allow(dead_code)]
pub fn to_lits(signed_values: &[i32]) -> Vec<Lit> {
    signed_values
        .iter()
        .map(|i| Lit::new(i.unsigned_abs() as usize, i.is_negative()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_db() {
        let mut db = ClauseDb::default();
        let cl0 = to_lits(&[1, 2, 3, 4]);
        let cl0_tautology = to_lits(&[1, 2, 3, -2, 5, 7]);
        let cl2 = to_lits(&[-1, -2, -3]);
        let cl3 = to_lits(&[1, 2, 1, 2, 3, -4, 5, -4]);
        let cl3_compact = to_lits(&[1, 2, 3, -4, 5]);
        let cl4 = to_lits(&[]);
        let cl5 = to_lits(&[-5]);
        let mut variables = VariableDb::default();
        variables.ensure_vars(9);
        let clause_id0 = db.push(cl0.as_slice(), &mut variables);
        assert_eq!(clause_id0, 0);
        let clause_id0_tautology = db.push(cl0_tautology.as_slice(), &mut variables);
        assert_eq!(clause_id0_tautology, NULL_CLAUSE);
        let clause_id2 = db.push(cl2.as_slice(), &mut variables);
        assert_eq!(clause_id2, 1);
        let clause_id3 = db.push(cl3.as_slice(), &mut variables);
        assert_eq!(clause_id3, 2);
        let clause_id4 = db.push(cl4.as_slice(), &mut variables);
        assert_eq!(clause_id4, 3);
        let clause_id5 = db.push(cl5.as_slice(), &mut variables);
        assert_eq!(clause_id5, 4);
        assert_eq!(db.literals(db.get(clause_id0)), &cl0);
        assert_eq!(db.literals(db.get(clause_id2)), &cl2);
        assert_eq!(db.literals(db.get(clause_id3)), &cl3_compact);
        assert_eq!(db.literals(db.get(clause_id4)), &cl4);
        assert_eq!(db.literals(db.get(clause_id5)), &cl5);

        let c6 = to_lits(&[5, 6]);
        let clause_id6 = db.push(c6.as_slice(), &mut variables);
        assert_eq!(db.literals(db.get(clause_id6)), &c6);
        assert!(db.literals(db.get(clause_id4)).is_empty());
        assert!(!db.literals(db.get(clause_id0)).is_empty());

        db.literals_mut(db.get(clause_id6))[0] = Lit::new(8, false);
        assert_eq!(db.literals(db.get(clause_id6)), &to_lits(&[8, 6]));
    }

    #[test]
    fn test_watch_db() {
        let mut db = WatchersDb::default();
        assert!(db[Lit::new(1, false)].is_empty());
        assert!(db[-Lit::new(1, false)].is_empty());

        db.add_watch(Lit::new(1, false), Watcher { clause: 0 });
        db.add_watch(-Lit::new(1, false), Watcher { clause: 1 });
        db.add_watch(Lit::new(1, false), Watcher { clause: 2 });

        assert_eq!(db[Lit::new(1, false)], &[Watcher { clause: 0 }, Watcher { clause: 2 }]);
        assert_eq!(db[-Lit::new(1, false)], &[Watcher { clause: 1 }]);

        db[Lit::new(1, false)].push(Watcher { clause: 3 });
        assert_eq!(
            db[Lit::new(1, false)],
            &[Watcher { clause: 0 }, Watcher { clause: 2 }, Watcher { clause: 3 }]
        );
    }
}
