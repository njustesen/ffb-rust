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

    #[test]
    fn game_id_zero_allowed() {
        let t = Talk::new(0, "c", SessionMode::Admin, "msg");
        assert_eq!(t.game_id, 0);
    }

    #[test]
    fn game_id_negative_allowed() {
        let t = Talk::new(-1, "c", SessionMode::Dev, "msg");
        assert_eq!(t.game_id, -1);
    }

    #[test]
    fn admin_session_mode_stored() {
        let t = Talk::new(5, "admin", SessionMode::Admin, "admin-talk");
        assert_eq!(t.session_mode, SessionMode::Admin);
        assert_eq!(t.coach, "admin");
    }

    #[test]
    fn empty_talk_string_allowed() {
        let t = Talk::new(99, "silent", SessionMode::Home, "");
        assert_eq!(t.talk, "");
    }

    #[test]
    fn talk_and_coach_are_owned_strings() {
        // Verifies that Talk owns its strings (coach and talk are String, not &str).
        let coach = String::from("dynamic_coach");
        let msg = String::from("dynamic_msg");
        let t = Talk::new(7, &coach, SessionMode::Away, &msg);
        drop(coach);
        drop(msg);
        assert_eq!(t.coach, "dynamic_coach");
        assert_eq!(t.talk, "dynamic_msg");
    }
}
