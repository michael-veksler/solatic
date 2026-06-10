pub type Lit = i32;
pub type ClauseId = u32;
pub const TERMINATOR: Lit = 0;

#[derive(Debug)]
pub struct ClauseDb {
    pool: Vec<Lit>,
    offsets: Vec<usize>,
}

impl ClauseDb {
    pub fn new() -> Self {
        Self { pool: Vec::new(), offsets: Vec::new() }
    }

    pub fn add_clause(&mut self, lits: &[Lit]) {
        self.offsets.push(self.pool.len());
        self.pool.extend_from_slice(lits);
        self.pool.push(TERMINATOR);
    }

    pub fn clause(&self, id: ClauseId) -> &[Lit] {
        let start = self.offsets[id as usize];
        let end = self.pool[start..]
            .iter()
            .position(|&lit| lit == TERMINATOR)
            .expect("missing clause terminator")
            + start;
        &self.pool[start..end] // ignore the terminator at `end`
    }

    pub fn is_empty_clause(&self, id: ClauseId) -> bool {
        self.clause(id).len() == 0
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

#[derive(Debug)]
#[allow(dead_code)]
struct WatchersDb {
    // FIXME: Vec<Watch> is not cache friendly.
    //        Better have the size and cpacity in the same memory block with the first Watch.
    //        Maybe use unsafe code to manage pointers, or use thin-vec  or thin-dst / erasable
    pos_watchers: Vec<Vec<Watcher>>,
    neg_watchers: Vec<Vec<Watcher>>,
    empty_watch: [Watcher; 0],
}


#[allow(dead_code)]
impl WatchersDb {
    pub fn new() -> Self {
        Self {
            pos_watchers: Vec::new(),
            neg_watchers: Vec::new(),
            empty_watch: [],
        }
    }
    pub fn watches(&self, lit: Lit) -> &[Watcher] {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) { &self.pos_watchers} else { &self.neg_watchers };
        if var < watchers.len() {
            &watchers[var]
        } else {
            &self.empty_watch.as_slice()
        }
    }
    pub fn update_watches(&mut self, lit: Lit) -> &mut Vec<Watcher> {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) { &mut self.pos_watchers} else { &mut self.neg_watchers };
        if var >= watchers.len() {
            watchers.resize(var + 1, Vec::new());
        }
        &mut watchers[var]
    }
    pub fn add_watch(&mut self, lit: Lit, watch: Watcher) {
        let var = var_of(lit).expect("invalid literal");
        let watchers = if is_pos(lit) { &mut self.pos_watchers} else { &mut self.neg_watchers };
        if var >= watchers.len() {
            watchers.resize(var + 1, Vec::new());
        }
        watchers[var].push(watch);
    }
}

/// Return the variable id for a literal.
fn var_of(lit: Lit) -> Option<usize> { 
    if lit == 0 {
        None
    } else {
        Some(lit.abs() as usize)
    }
}

/// Return true when a literal is positive.
fn is_pos(lit: Lit) -> bool { 
    lit > 0 
}

pub struct Solver {
    db: ClauseDb,
}

impl Solver {
    pub fn new() -> Self {
        Self { db: ClauseDb::new() }
    }

    pub fn add_clause(&mut self, lits: &[Lit]) {
        self.db.add_clause(lits);
    }

    pub fn solve(&self) -> Result<SolveResult, String> {
        for id in 0..self.db.offsets.len() {
            if self.db.is_empty_clause(id as ClauseId) {
                return Ok(SolveResult::Unsat);
            }
        }
        Ok(SolveResult::Sat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_db() {
        let mut db = ClauseDb::new();
        let cl0 = [1, 2, 3, 4];
        let cl1 = [-1, -2, -3];
        let cl2 = [];
        let cl3 = [-5];
        db.add_clause(cl0.as_slice());
        db.add_clause(cl1.as_slice());
        db.add_clause(cl2.as_slice());
        db.add_clause(cl3.as_slice());
        assert_eq!(db.clause(0), &cl0);
        assert_eq!(db.clause(1), &cl1);
        assert_eq!(db.clause(2), &cl2);
        assert_eq!(db.clause(3), &cl3);

        let c4 = [5, 6];
        db.add_clause(c4.as_slice());
        assert_eq!(db.clause(4), &c4);
        assert!(db.is_empty_clause(2));
        assert!(! db.is_empty_clause(0));
    }

    #[test]
    fn test_watch_db() {
        let mut db = WatchersDb::new();
        assert!(db.watches(1).is_empty());
        assert!(db.watches(-1).is_empty());

        db.add_watch(1, Watcher { clause: 0, blocking_literal: 1 });
        db.add_watch(-1, Watcher { clause: 1, blocking_literal: -1 });
        db.add_watch(1, Watcher { clause: 2, blocking_literal: 3 });

        assert_eq!(db.watches(1), &[Watcher { clause: 0, blocking_literal: 1 }, 
                                    Watcher { clause: 2, blocking_literal: 3 }]);
        assert_eq!(db.watches(-1), &[Watcher { clause: 1, blocking_literal: -1 }]);

        db.update_watches(1).push(Watcher { clause: 3, blocking_literal: -8 });
        assert_eq!(db.watches(1), &[Watcher { clause: 0, blocking_literal: 1 }, 
                                    Watcher { clause: 2, blocking_literal: 3 }, 
                                    Watcher { clause: 3, blocking_literal: -8 }]);
    }
}