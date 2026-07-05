/// 1:1 translation of `com.fumbbl.ffb.server.ReplayState`.
use std::collections::HashSet;

pub struct ReplayState {
    name: String,
    command_nr: i32,
    speed: i32,
    running: bool,
    forward: bool,
    coaches_prevented_from_sketching: HashSet<String>,
}

impl ReplayState {
    pub fn new(name: &str) -> Self {
        ReplayState {
            name: name.to_string(),
            command_nr: 0,
            speed: 0,
            running: false,
            forward: false,
            coaches_prevented_from_sketching: HashSet::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_command_nr(&self) -> i32 {
        self.command_nr
    }

    pub fn get_speed(&self) -> i32 {
        self.speed
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_forward(&self) -> bool {
        self.forward
    }

    pub fn prevent_coach_from_sketching(&mut self, coach: &str) {
        self.coaches_prevented_from_sketching.insert(coach.to_string());
    }

    pub fn allow_coach_to_sketch(&mut self, coach: &str) {
        self.coaches_prevented_from_sketching.remove(coach);
    }

    pub fn is_coach_prevented_from_sketching(&self, coach: &str) -> bool {
        self.coaches_prevented_from_sketching.contains(coach)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_defaults() {
        let rs = ReplayState::new("my-replay");
        assert_eq!(rs.get_name(), "my-replay");
        assert_eq!(rs.get_command_nr(), 0);
        assert_eq!(rs.get_speed(), 0);
        assert!(!rs.is_running());
        assert!(!rs.is_forward());
    }

    #[test]
    fn prevent_and_allow_coach_sketching() {
        let mut rs = ReplayState::new("r");
        rs.prevent_coach_from_sketching("coachA");
        assert!(rs.is_coach_prevented_from_sketching("coachA"));
        assert!(!rs.is_coach_prevented_from_sketching("coachB"));
        rs.allow_coach_to_sketch("coachA");
        assert!(!rs.is_coach_prevented_from_sketching("coachA"));
    }

    #[test]
    fn multiple_coaches_tracked_independently() {
        let mut rs = ReplayState::new("r");
        rs.prevent_coach_from_sketching("c1");
        rs.prevent_coach_from_sketching("c2");
        rs.allow_coach_to_sketch("c1");
        assert!(!rs.is_coach_prevented_from_sketching("c1"));
        assert!(rs.is_coach_prevented_from_sketching("c2"));
    }
}
