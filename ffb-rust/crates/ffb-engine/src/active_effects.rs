/// 1:1 translation of `com.fumbbl.ffb.server.ActiveEffects`.
use std::collections::HashSet;
use ffb_model::enums::Weather;

#[derive(Debug, Default, Clone)]
pub struct ActiveEffects {
    old_weather: Option<Weather>,
    skip_restore_weather: bool,
    stalling: bool,
    team_ids_additional_assist: Vec<String>,
    shadowers: Vec<String>,
    leaders: HashSet<String>,
}

impl ActiveEffects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_old_weather(&self) -> Option<Weather> {
        self.old_weather
    }

    pub fn set_old_weather(&mut self, old_weather: Option<Weather>) {
        self.old_weather = old_weather;
    }

    pub fn is_skip_restore_weather(&self) -> bool {
        self.skip_restore_weather
    }

    pub fn set_skip_restore_weather(&mut self, skip_restore_weather: bool) {
        self.skip_restore_weather = skip_restore_weather;
    }

    pub fn is_stalling(&self) -> bool {
        self.stalling
    }

    pub fn set_stalling(&mut self, stalling: bool) {
        self.stalling = stalling;
    }

    pub fn set_team_ids_additional_assist(&mut self, team_ids: &[String]) {
        self.team_ids_additional_assist.extend_from_slice(team_ids);
    }

    pub fn get_team_ids_additional_assist(&self) -> &[String] {
        &self.team_ids_additional_assist
    }

    pub fn remove_additional_assist(&mut self, team_id: &str) {
        self.team_ids_additional_assist.retain(|id| id != team_id);
    }

    pub fn clear_shadowers(&mut self) {
        self.shadowers.clear();
    }

    pub fn add_shadower(&mut self, player_id: &str) {
        self.shadowers.push(player_id.to_string());
    }

    pub fn get_shadowers(&self) -> &[String] {
        &self.shadowers
    }

    pub fn add_leader(&mut self, leader: &str) {
        self.leaders.insert(leader.to_string());
    }

    pub fn get_leaders(&self) -> &HashSet<String> {
        &self.leaders
    }

    pub fn clear_leaders(&mut self) {
        self.leaders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_all_empty_false() {
        let ae = ActiveEffects::new();
        assert!(ae.get_old_weather().is_none());
        assert!(!ae.is_skip_restore_weather());
        assert!(!ae.is_stalling());
        assert!(ae.get_team_ids_additional_assist().is_empty());
        assert!(ae.get_shadowers().is_empty());
        assert!(ae.get_leaders().is_empty());
    }

    #[test]
    fn remove_additional_assist_only_removes_matching() {
        let mut ae = ActiveEffects::new();
        ae.set_team_ids_additional_assist(&["team1".to_string(), "team2".to_string()]);
        ae.remove_additional_assist("team1");
        assert_eq!(ae.get_team_ids_additional_assist(), &["team2".to_string()]);
    }

    #[test]
    fn shadowers_add_and_clear() {
        let mut ae = ActiveEffects::new();
        ae.add_shadower("p1");
        ae.add_shadower("p2");
        assert_eq!(ae.get_shadowers().len(), 2);
        ae.clear_shadowers();
        assert!(ae.get_shadowers().is_empty());
    }

    #[test]
    fn leaders_add_and_clear() {
        let mut ae = ActiveEffects::new();
        ae.add_leader("coach1");
        assert!(ae.get_leaders().contains("coach1"));
        ae.clear_leaders();
        assert!(ae.get_leaders().is_empty());
    }

    #[test]
    fn old_weather_set_and_get() {
        let mut ae = ActiveEffects::new();
        ae.set_old_weather(Some(Weather::Blizzard));
        assert_eq!(ae.get_old_weather(), Some(Weather::Blizzard));
    }

    #[test]
    fn stalling_and_skip_restore_weather_can_be_toggled() {
        let mut ae = ActiveEffects::new();
        ae.set_stalling(true);
        ae.set_skip_restore_weather(true);
        assert!(ae.is_stalling());
        assert!(ae.is_skip_restore_weather());
        ae.set_stalling(false);
        ae.set_skip_restore_weather(false);
        assert!(!ae.is_stalling());
        assert!(!ae.is_skip_restore_weather());
    }

    #[test]
    fn multiple_leaders_are_tracked() {
        let mut ae = ActiveEffects::new();
        ae.add_leader("coach1");
        ae.add_leader("coach2");
        assert!(ae.get_leaders().contains("coach1"));
        assert!(ae.get_leaders().contains("coach2"));
        assert_eq!(ae.get_leaders().len(), 2);
    }
}
