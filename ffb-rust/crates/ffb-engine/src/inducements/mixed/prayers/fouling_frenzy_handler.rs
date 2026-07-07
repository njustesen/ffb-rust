/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.FoulingFrenzyHandler`.
/// Abstract mixed handler — provides initEffect/removeEffectInternal shared by bb2020/bb2025.
/// Edition-specific handlers implement `handled_prayer_name()`.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use crate::prayer_state::PrayerState;

/// Java: initEffect(GameState, Team) — adds fouling frenzy for the praying team.
pub fn init_effect(prayer_state: &mut PrayerState, team_id: &str) -> bool {
    prayer_state.add_fouling_frenzy(team_id);
    true
}

/// Java: removeEffectInternal(GameState, Team) — removes fouling frenzy for the team.
pub fn remove_effect_internal(prayer_state: &mut PrayerState, team_id: &str) {
    prayer_state.remove_fouling_frenzy(team_id);
}

/// Java: animationType()
pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_FOULING_FRENZY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_effect_adds_fouling_frenzy() {
        let mut state = PrayerState::new();
        let result = init_effect(&mut state, "team1");
        assert!(result);
        assert!(state.has_fouling_frenzy("team1"));
    }

    #[test]
    fn remove_effect_removes_fouling_frenzy() {
        let mut state = PrayerState::new();
        state.add_fouling_frenzy("team1");
        remove_effect_internal(&mut state, "team1");
        assert!(!state.has_fouling_frenzy("team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_FOULING_FRENZY);
    }

    #[test]
    fn init_effect_does_not_affect_other_team() {
        let mut state = PrayerState::new();
        init_effect(&mut state, "team1");
        assert!(!state.has_fouling_frenzy("team2"));
    }

    #[test]
    fn remove_effect_on_missing_team_is_safe() {
        let mut state = PrayerState::new();
        remove_effect_internal(&mut state, "team_not_present");
        assert!(!state.has_fouling_frenzy("team_not_present"));
    }
}
