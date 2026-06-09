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

    pub fn clause(&self, id: usize) -> &[Lit] {
        let start = self.offsets[id];
        let end = self.pool[start..]
            .iter()
            .position(|&lit| lit == TERMINATOR)
            .expect("missing clause terminator")
            + start;
        &self.pool[start..end] // ignore the terminator at `end`
    }

    pub fn is_empty_clause(&self, id: usize) -> bool {
        self.clause(id).len() == 0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolveResult {
    Sat,
    Unsat,
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Watch {
    clause: ClauseId,
}

#[derive(Debug)]
#[allow(dead_code)]
struct WatchDb {
    watches: Vec<Vec<Watch>>,
    empty_watch: [Watch; 0],
}


#[allow(dead_code)]
impl WatchDb {
    pub fn new() -> Self {
        Self {
            watches: Vec::new(),
            empty_watch: [],
        }
    }
    pub fn watches(&self, lit: Lit) -> &[Watch] {
        let idx = lit_index(lit).expect("invalid literal");
        if idx < self.watches.len() {
            &self.watches[idx]
        } else {
            &self.empty_watch.as_slice()
        }
    }
    pub fn update_watches(&mut self, lit: Lit) -> &mut Vec<Watch> {
        let idx = lit_index(lit).expect("invalid literal");
        if idx >= self.watches.len() {
            self.watches.resize(idx + 1, Vec::new());
        }
        &mut self.watches[idx]
    }
    pub fn add_watch(&mut self, lit: Lit, watch: Watch) {
        let idx = lit_index(lit).expect("invalid literal");
        if idx >= self.watches.len() {
            self.watches.resize(idx + 1, Vec::new());
        }
        self.watches[idx].push(watch);
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

/// Map a literal to its watch list index.
///
/// Literals are stored as `+v` or `-v`, and the watch arrays use a dense
/// index per signed literal.
fn lit_index(lit: Lit) -> Option<usize> {
    var_of(lit).map(|var| (var - 1) * 2 + if is_pos(lit) { 0 } else { 1 })
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
            if self.db.is_empty_clause(id) {
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
        let mut db = WatchDb::new();
        assert!(db.watches(1).is_empty());
        assert!(db.watches(-1).is_empty());

        db.add_watch(1, Watch { clause: 0 });
        db.add_watch(-1, Watch { clause: 1 });
        db.add_watch(1, Watch { clause: 2 });

        assert_eq!(db.watches(1), &[Watch { clause: 0 }, Watch { clause: 2 }]);
        assert_eq!(db.watches(-1), &[Watch { clause: 1 }]);

        db.update_watches(1).push(Watch { clause: 3 });
        assert_eq!(db.watches(1), &[Watch { clause: 0 }, Watch { clause: 2 }, Watch { clause: 3 }]);
    }
}