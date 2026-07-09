use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.net.GameCoach`.
/// Pairs a game id with a coach name; used in lobby/join tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCoach {
    /// Java: `fGame`
    pub game: String,
    /// Java: `fCoach`
    pub coach: String,
}

impl GameCoach {
    pub fn new(game: impl Into<String>, coach: impl Into<String>) -> Self {
        Self {
            game: game.into(),
            coach: coach.into(),
        }
    }
}

impl PartialEq for GameCoach {
    fn eq(&self, other: &Self) -> bool {
        self.game == other.game && self.coach == other.coach
    }
}

impl Eq for GameCoach {}

impl std::hash::Hash for GameCoach {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{}:{}", self.game, self.coach).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_fields() {
        let gc = GameCoach::new("game1", "coachA");
        assert_eq!(gc.game, "game1");
        assert_eq!(gc.coach, "coachA");
    }

    #[test]
    fn equality_same() {
        let a = GameCoach::new("g", "c");
        let b = GameCoach::new("g", "c");
        assert_eq!(a, b);
    }

    #[test]
    fn equality_different_coach() {
        let a = GameCoach::new("g", "c1");
        let b = GameCoach::new("g", "c2");
        assert_ne!(a, b);
    }

    #[test]
    fn serde_round_trip() {
        let gc = GameCoach::new("myGame", "myCoach");
        let json = serde_json::to_string(&gc).unwrap();
        let back: GameCoach = serde_json::from_str(&json).unwrap();
        assert_eq!(back, gc);
    }
}
