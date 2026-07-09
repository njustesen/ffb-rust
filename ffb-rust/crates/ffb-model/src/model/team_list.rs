use serde::{Deserialize, Serialize};
use super::team_list_entry::TeamListEntry;

/// 1:1 translation of com.fumbbl.ffb.model.TeamList.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamList {
    pub entries: Vec<TeamListEntry>,
}

impl TeamList {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, entry: TeamListEntry) { self.entries.push(entry); }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(TeamList::new().is_empty());
    }

    #[test]
    fn add_increases_len() {
        let mut tl = TeamList::new();
        tl.add(TeamListEntry::default());
        assert_eq!(tl.len(), 1);
    }
}
