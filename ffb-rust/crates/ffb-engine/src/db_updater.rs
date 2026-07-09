use std::collections::VecDeque;

/// Async DB write queue — 1:1 translation of Java DbUpdater.
///
/// Processes DB transactions from a background queue. All actual DB work is Phase ZU.
pub struct DbUpdater {
    update_queue: VecDeque<DbTransaction>,
    stopped: bool,
}

/// Placeholder for a DB transaction batch (Phase ZU will wire actual SQLx transactions).
pub struct DbTransaction {
    pub statements: Vec<String>,
}

impl DbTransaction {
    pub fn new() -> Self {
        Self { statements: Vec::new() }
    }

    pub fn add(&mut self, statement: impl Into<String>) {
        self.statements.push(statement.into());
    }
}

impl Default for DbTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl DbUpdater {
    pub fn new() -> Self {
        Self { update_queue: VecDeque::new(), stopped: false }
    }

    pub fn add(&mut self, transaction: DbTransaction) -> bool {
        if self.stopped {
            return false;
        }
        self.update_queue.push_back(transaction);
        true
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn queue_size(&self) -> usize {
        self.update_queue.len()
    }

    pub fn run(&mut self) {
        // Phase ZU: dequeue and execute DB transactions
        todo!("Phase ZU: DB transaction execution")
    }
}

impl Default for DbUpdater {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_to_queue() {
        let mut updater = DbUpdater::new();
        let tx = DbTransaction::new();
        assert!(updater.add(tx));
        assert_eq!(updater.queue_size(), 1);
    }

    #[test]
    fn test_stopped_rejects_new_transactions() {
        let mut updater = DbUpdater::new();
        updater.stop();
        let tx = DbTransaction::new();
        assert!(!updater.add(tx));
        assert_eq!(updater.queue_size(), 0);
    }
}
