/// 1:1 translation of `com.fumbbl.ffb.server.Talk`.
use crate::session_mode::SessionMode;

pub struct Talk {
    pub game_id: i64,
    pub coach: String,
    pub session_mode: SessionMode,
    pub talk: String,
}

impl Talk {
    pub fn new(game_id: i64, coach: &str, session_mode: SessionMode, talk: &str) -> Self {
        Talk {
            game_id,
            coach: coach.to_string(),
            session_mode,
            talk: talk.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let t = Talk::new(42, "coach1", SessionMode::Home, "hello");
        assert_eq!(t.game_id, 42);
        assert_eq!(t.coach, "coach1");
        assert_eq!(t.session_mode, SessionMode::Home);
        assert_eq!(t.talk, "hello");
    }

    #[test]
    fn talk_field_stored_separately_from_coach() {
        let t = Talk::new(1, "player", SessionMode::Away, "gg");
        assert_eq!(t.talk, "gg");
        assert_ne!(t.coach, t.talk);
    }

    #[test]
    fn session_mode_spec_works() {
        let t = Talk::new(0, "spectator", SessionMode::Spec, "watching");
        assert_eq!(t.session_mode, SessionMode::Spec);
    }
}
