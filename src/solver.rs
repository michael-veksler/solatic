pub type Lit = i32;
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
        &self.pool[start..=end]
    }

    pub fn is_empty_clause(&self, id: usize) -> bool {
        self.clause(id).len() == 1
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolveResult {
    Sat,
    Unsat,
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
