/// 1:1 translation of `com.fumbbl.ffb.CardTarget`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum CardTarget {
    #[default]
    TURN,
    OWN_PLAYER,
    OPPOSING_PLAYER,
    ANY_PLAYER,
}

impl CardTarget {
    /// Java: getId()
    pub fn get_id(self) -> i32 {
        match self {
            CardTarget::TURN => 1,
            CardTarget::OWN_PLAYER => 2,
            CardTarget::OPPOSING_PLAYER => 3,
            CardTarget::ANY_PLAYER => 4,
        }
    }

    /// Java: getName()
    pub fn get_name(self) -> &'static str {
        match self {
            CardTarget::TURN => "turn",
            CardTarget::OWN_PLAYER => "ownPlayer",
            CardTarget::OPPOSING_PLAYER => "opposingPlayer",
            CardTarget::ANY_PLAYER => "anyPlayer",
        }
    }

    /// Java: isPlayedOnPlayer()
    pub fn is_played_on_player(self) -> bool {
        matches!(self, CardTarget::OWN_PLAYER | CardTarget::OPPOSING_PLAYER | CardTarget::ANY_PLAYER)
    }

    /// Java: fromId(int)
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(CardTarget::TURN),
            2 => Some(CardTarget::OWN_PLAYER),
            3 => Some(CardTarget::OPPOSING_PLAYER),
            4 => Some(CardTarget::ANY_PLAYER),
            _ => None,
        }
    }

    /// Java: fromName(String)
    pub fn from_name(name: &str) -> Option<Self> {
        [CardTarget::TURN, CardTarget::OWN_PLAYER, CardTarget::OPPOSING_PLAYER, CardTarget::ANY_PLAYER]
            .into_iter()
            .find(|t| t.get_name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_returns_correct_id() {
        assert_eq!(CardTarget::TURN.get_id(), 1);
        assert_eq!(CardTarget::OWN_PLAYER.get_id(), 2);
        assert_eq!(CardTarget::OPPOSING_PLAYER.get_id(), 3);
        assert_eq!(CardTarget::ANY_PLAYER.get_id(), 4);
    }

    #[test]
    fn get_name_returns_correct_name() {
        assert_eq!(CardTarget::TURN.get_name(), "turn");
        assert_eq!(CardTarget::OWN_PLAYER.get_name(), "ownPlayer");
        assert_eq!(CardTarget::OPPOSING_PLAYER.get_name(), "opposingPlayer");
        assert_eq!(CardTarget::ANY_PLAYER.get_name(), "anyPlayer");
    }

    #[test]
    fn is_played_on_player_true_for_player_targets() {
        assert!(CardTarget::OWN_PLAYER.is_played_on_player());
        assert!(CardTarget::OPPOSING_PLAYER.is_played_on_player());
        assert!(CardTarget::ANY_PLAYER.is_played_on_player());
    }

    #[test]
    fn is_played_on_player_false_for_turn() {
        assert!(!CardTarget::TURN.is_played_on_player());
    }

    #[test]
    fn from_id_round_trips() {
        for t in [CardTarget::TURN, CardTarget::OWN_PLAYER, CardTarget::OPPOSING_PLAYER, CardTarget::ANY_PLAYER] {
            assert_eq!(CardTarget::from_id(t.get_id()), Some(t));
        }
        assert_eq!(CardTarget::from_id(99), None);
    }

    #[test]
    fn from_name_round_trips_case_insensitively() {
        assert_eq!(CardTarget::from_name("ownPlayer"), Some(CardTarget::OWN_PLAYER));
        assert_eq!(CardTarget::from_name("OWNPLAYER"), Some(CardTarget::OWN_PLAYER));
        assert_eq!(CardTarget::from_name("opposingPlayer"), Some(CardTarget::OPPOSING_PLAYER));
        assert_eq!(CardTarget::from_name("no-such-target"), None);
    }

    #[test]
    fn default_is_turn() {
        assert_eq!(CardTarget::default(), CardTarget::TURN);
    }
}
