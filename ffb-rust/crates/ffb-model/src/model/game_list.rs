use serde::{Deserialize, Serialize};
use super::game_list_entry::GameListEntry;

/// 1:1 translation of com.fumbbl.ffb.model.GameList.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameList {
    pub entries: Vec<GameListEntry>,
}

impl GameList {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, entry: GameListEntry) { self.entries.push(entry); }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(GameList::new().is_empty());
    }

    #[test]
    fn add_increases_len() {
        let mut gl = GameList::new();
        gl.add(GameListEntry::default());
        assert_eq!(gl.len(), 1);
    }
}
