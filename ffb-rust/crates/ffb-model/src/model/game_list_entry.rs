use serde::{Deserialize, Serialize};
use crate::enums::GameStatus;

/// 1:1 translation of com.fumbbl.ffb.model.GameListEntry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameListEntry {
    pub game_id: u64,
    pub home_team: String,
    pub away_team: String,
    pub status: Option<GameStatus>,
}

impl GameListEntry {
    pub fn get_game_id(&self) -> u64 { self.game_id }
    pub fn get_home_team(&self) -> &str { &self.home_team }
    pub fn get_away_team(&self) -> &str { &self.away_team }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_id_is_zero() {
        assert_eq!(GameListEntry::default().get_game_id(), 0);
    }

    #[test]
    fn get_home_team_returns_name() {
        let e = GameListEntry { game_id: 1, home_team: "Home".to_string(), away_team: "Away".to_string(), status: None };
        assert_eq!(e.get_home_team(), "Home");
    }
}
