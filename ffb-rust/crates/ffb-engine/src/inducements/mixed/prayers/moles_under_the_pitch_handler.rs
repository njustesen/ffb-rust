/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.MolesUnderThePitchHandler`.
use ffb_model::model::animation_type::AnimationType;
use crate::prayer_state::PrayerState;

pub fn init_effect(prayer_state: &mut PrayerState, team_id: &str) -> bool {
    prayer_state.add_moles_under_the_pitch(team_id);
    true
}

pub fn remove_effect_internal(prayer_state: &mut PrayerState, team_id: &str) {
    prayer_state.remove_moles_under_the_pitch(team_id);
}

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_MOLES_UNDER_THE_PITCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_effect_adds_moles_under_the_pitch() {
        let mut state = PrayerState::new();
        assert!(init_effect(&mut state, "teamA"));
        assert!(state.get_moles_under_the_pitch().contains("teamA"));
    }

    #[test]
    fn remove_effect_removes_moles() {
        let mut state = PrayerState::new();
        state.add_moles_under_the_pitch("teamA");
        remove_effect_internal(&mut state, "teamA");
        assert!(!state.get_moles_under_the_pitch().contains("teamA"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_MOLES_UNDER_THE_PITCH);
    }

    #[test]
    fn init_effect_does_not_affect_other_team() {
        let mut state = PrayerState::new();
        init_effect(&mut state, "teamA");
        assert!(!state.get_moles_under_the_pitch().contains("teamB"));
    }

    #[test]
    fn remove_effect_on_missing_team_is_safe() {
        let mut state = PrayerState::new();
        remove_effect_internal(&mut state, "team_not_present");
        assert!(!state.get_moles_under_the_pitch().contains("team_not_present"));
    }

    #[test]
    fn init_effect_returns_true_always() {
        let mut state = PrayerState::new();
        assert!(init_effect(&mut state, "teamB"));
    }

    #[test]
    fn double_add_and_remove_leaves_clean_state() {
        let mut state = PrayerState::new();
        init_effect(&mut state, "teamA");
        init_effect(&mut state, "teamA");
        remove_effect_internal(&mut state, "teamA");
        assert!(!state.get_moles_under_the_pitch().contains("teamA"));
    }
}
