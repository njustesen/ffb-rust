/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.PerfectPassingHandler`.
use ffb_model::model::animation_type::AnimationType;
use crate::prayer_state::PrayerState;

pub fn init_effect(prayer_state: &mut PrayerState, team_id: &str) -> bool {
    prayer_state.add_get_additional_completion_spp(team_id);
    true
}

pub fn remove_effect_internal(prayer_state: &mut PrayerState, team_id: &str) {
    prayer_state.remove_get_additional_completion_spp(team_id);
}

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_PERFECT_PASSING
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_effect_adds_additional_completion_spp() {
        let mut state = PrayerState::new();
        assert!(init_effect(&mut state, "team1"));
        assert!(state.get_additional_completion_spp_teams().contains("team1"));
    }

    #[test]
    fn remove_effect_removes_completion_spp_bonus() {
        let mut state = PrayerState::new();
        state.add_get_additional_completion_spp("team1");
        remove_effect_internal(&mut state, "team1");
        assert!(!state.get_additional_completion_spp_teams().contains("team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_PERFECT_PASSING);
    }

    #[test]
    fn init_effect_does_not_affect_other_team() {
        let mut state = PrayerState::new();
        init_effect(&mut state, "team1");
        assert!(!state.get_additional_completion_spp_teams().contains("team2"));
    }

    #[test]
    fn remove_effect_on_missing_team_is_safe() {
        let mut state = PrayerState::new();
        remove_effect_internal(&mut state, "team_not_present");
        assert!(!state.get_additional_completion_spp_teams().contains("team_not_present"));
    }

    #[test]
    fn init_effect_returns_true_always() {
        let mut state = PrayerState::new();
        assert!(init_effect(&mut state, "team2"));
    }

    #[test]
    fn double_add_and_remove_leaves_clean_state() {
        let mut state = PrayerState::new();
        init_effect(&mut state, "team1");
        init_effect(&mut state, "team1");
        remove_effect_internal(&mut state, "team1");
        assert!(!state.get_additional_completion_spp_teams().contains("team1"));
    }
}
