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
}
