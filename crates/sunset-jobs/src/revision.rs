use std::collections::HashMap;

pub struct RevisionTable {
    revisions: HashMap<u64, u64>,
}

impl RevisionTable {
    pub fn new() -> Self {
        Self {
            revisions: HashMap::new(),
        }
    }

    pub fn current(&self, key: u64) -> u64 {
        self.revisions.get(&key).copied().unwrap_or(0)
    }

    pub fn bump(&mut self, key: u64) -> u64 {
        let next = self.current(key) + 1;
        self.revisions.insert(key, next);
        next
    }

    pub fn is_stale(&self, key: u64, revision: u64) -> bool {
        revision < self.current(key)
    }
}

impl Default for RevisionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_key_is_revision_zero() {
        let table = RevisionTable::new();
        assert_eq!(table.current(0), 0);
        assert!(!table.is_stale(0, 0));
    }

    #[test]
    fn bump_advances_revision() {
        let mut table = RevisionTable::new();
        assert_eq!(table.bump(7), 1);
        assert_eq!(table.bump(7), 2);
        assert_eq!(table.current(7), 2);
    }

    #[test]
    fn stale_revision_is_rejected() {
        let mut table = RevisionTable::new();
        table.bump(7);
        table.bump(7);
        assert!(table.is_stale(7, 1));
        assert!(!table.is_stale(7, 2));
        assert!(!table.is_stale(7, 3));
    }
}
