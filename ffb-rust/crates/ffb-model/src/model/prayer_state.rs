/// 1:1 translation of `com.fumbbl.ffb.server.PrayerState`.
/// Tracks per-team and per-player prayer effect state for a game.
/// Lives in ffb-model so that Game can own it and steps can access it
/// via `game.prayer_state` (mirroring Java's `getGameState().getPrayerState()`).
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PrayerState {
    friends_with_ref: HashSet<String>,
    get_additional_catches_spp: HashSet<String>,
    get_additional_completion_spp: HashSet<String>,
    get_additional_cas_spp: HashSet<String>,
    under_scrutiny: HashSet<String>,
    fouling_frenzy: HashSet<String>,
    fan_interaction: HashSet<String>,
    moles_under_the_pitch: HashSet<String>,
    should_not_stall: HashSet<String>,
    stallers: HashSet<String>,
}

impl PrayerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_friends_with_ref(&mut self, team_id: &str) {
        self.friends_with_ref.insert(team_id.to_string());
    }

    pub fn remove_friends_with_ref(&mut self, team_id: &str) {
        self.friends_with_ref.remove(team_id);
    }

    pub fn is_friends_with_ref(&self, team_id: &str) -> bool {
        self.friends_with_ref.contains(team_id)
    }

    pub fn add_get_additional_cas_spp(&mut self, team_id: &str) {
        self.get_additional_cas_spp.insert(team_id.to_string());
    }

    pub fn remove_get_additional_cas_spp(&mut self, team_id: &str) {
        self.get_additional_cas_spp.remove(team_id);
    }

    pub fn get_additional_cas_spp_teams(&self) -> &HashSet<String> {
        &self.get_additional_cas_spp
    }

    pub fn add_get_additional_completion_spp(&mut self, team_id: &str) {
        self.get_additional_completion_spp.insert(team_id.to_string());
    }

    pub fn remove_get_additional_completion_spp(&mut self, team_id: &str) {
        self.get_additional_completion_spp.remove(team_id);
    }

    pub fn get_additional_completion_spp_teams(&self) -> &HashSet<String> {
        &self.get_additional_completion_spp
    }

    pub fn add_get_additional_catches_spp(&mut self, team_id: &str) {
        self.get_additional_catches_spp.insert(team_id.to_string());
    }

    pub fn remove_get_additional_catches_spp(&mut self, team_id: &str) {
        self.get_additional_catches_spp.remove(team_id);
    }

    pub fn get_additional_catches_spp_teams(&self) -> &HashSet<String> {
        &self.get_additional_catches_spp
    }

    pub fn add_under_scrutiny(&mut self, team_id: &str) {
        self.under_scrutiny.insert(team_id.to_string());
    }

    pub fn remove_under_scrutiny(&mut self, team_id: &str) {
        self.under_scrutiny.remove(team_id);
    }

    pub fn is_under_scrutiny(&self, team_id: &str) -> bool {
        self.under_scrutiny.contains(team_id)
    }

    pub fn add_fan_interaction(&mut self, team_id: &str) {
        self.fan_interaction.insert(team_id.to_string());
    }

    pub fn remove_fan_interaction(&mut self, team_id: &str) {
        self.fan_interaction.remove(team_id);
    }

    pub fn has_fan_interaction(&self, team_id: &str) -> bool {
        self.fan_interaction.contains(team_id)
    }

    pub fn add_fouling_frenzy(&mut self, team_id: &str) {
        self.fouling_frenzy.insert(team_id.to_string());
    }

    pub fn remove_fouling_frenzy(&mut self, team_id: &str) {
        self.fouling_frenzy.remove(team_id);
    }

    pub fn has_fouling_frenzy(&self, team_id: &str) -> bool {
        self.fouling_frenzy.contains(team_id)
    }

    pub fn add_moles_under_the_pitch(&mut self, team_id: &str) {
        self.moles_under_the_pitch.insert(team_id.to_string());
    }

    pub fn remove_moles_under_the_pitch(&mut self, team_id: &str) {
        self.moles_under_the_pitch.remove(team_id);
    }

    pub fn get_moles_under_the_pitch(&self) -> &HashSet<String> {
        &self.moles_under_the_pitch
    }

    pub fn add_should_not_stall(&mut self, team_id: &str) {
        self.should_not_stall.insert(team_id.to_string());
    }

    pub fn remove_should_not_stall(&mut self, team_id: &str) {
        self.should_not_stall.remove(team_id);
    }

    pub fn should_not_stall(&self, team_id: &str) -> bool {
        self.should_not_stall.contains(team_id)
    }

    pub fn add_staller(&mut self, player_id: &str) {
        self.stallers.insert(player_id.to_string());
    }

    pub fn remove_staller(&mut self, player_id: &str) {
        self.stallers.remove(player_id);
    }

    pub fn clear_stallers(&mut self) {
        self.stallers.clear();
    }

    pub fn is_stalling(&self, player_id: &str) -> bool {
        self.stallers.contains(player_id)
    }

    pub fn get_staller_ids(&self) -> &HashSet<String> {
        &self.stallers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friends_with_ref_add_check_remove() {
        let mut state = PrayerState::new();
        state.add_friends_with_ref("team1");
        assert!(state.is_friends_with_ref("team1"));
        assert!(!state.is_friends_with_ref("team2"));
        state.remove_friends_with_ref("team1");
        assert!(!state.is_friends_with_ref("team1"));
    }

    #[test]
    fn stallers_add_check_clear() {
        let mut state = PrayerState::new();
        state.add_staller("player1");
        state.add_staller("player2");
        assert!(state.is_stalling("player1"));
        assert!(state.is_stalling("player2"));
        state.clear_stallers();
        assert!(!state.is_stalling("player1"));
        assert!(!state.is_stalling("player2"));
    }

    #[test]
    fn under_scrutiny_add_remove() {
        let mut state = PrayerState::new();
        state.add_under_scrutiny("teamA");
        assert!(state.is_under_scrutiny("teamA"));
        state.remove_under_scrutiny("teamA");
        assert!(!state.is_under_scrutiny("teamA"));
    }

    #[test]
    fn fouling_frenzy_independent_of_fan_interaction() {
        let mut state = PrayerState::new();
        state.add_fouling_frenzy("team1");
        state.add_fan_interaction("team2");
        assert!(state.has_fouling_frenzy("team1"));
        assert!(!state.has_fouling_frenzy("team2"));
        assert!(state.has_fan_interaction("team2"));
        assert!(!state.has_fan_interaction("team1"));
    }

    #[test]
    fn additional_spp_teams_returns_ref_to_set() {
        let mut state = PrayerState::new();
        state.add_get_additional_cas_spp("teamX");
        state.add_get_additional_completion_spp("teamY");
        assert!(state.get_additional_cas_spp_teams().contains("teamX"));
        assert!(state.get_additional_completion_spp_teams().contains("teamY"));
    }

    #[test]
    fn should_not_stall_add_remove() {
        let mut state = PrayerState::new();
        state.add_should_not_stall("team1");
        assert!(state.should_not_stall("team1"));
        state.remove_should_not_stall("team1");
        assert!(!state.should_not_stall("team1"));
    }
}
