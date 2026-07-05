/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.FriendsWithTheRefHandler`.
use ffb_model::model::animation_type::AnimationType;
use crate::prayer_state::PrayerState;

pub fn init_effect(prayer_state: &mut PrayerState, team_id: &str) -> bool {
    prayer_state.add_friends_with_ref(team_id);
    true
}

pub fn remove_effect_internal(prayer_state: &mut PrayerState, team_id: &str) {
    prayer_state.remove_friends_with_ref(team_id);
}

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_FRIENDS_WITH_THE_REF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_effect_adds_friends_with_ref() {
        let mut state = PrayerState::new();
        assert!(init_effect(&mut state, "team1"));
        assert!(state.is_friends_with_ref("team1"));
    }

    #[test]
    fn remove_effect_removes_friends_with_ref() {
        let mut state = PrayerState::new();
        state.add_friends_with_ref("team1");
        remove_effect_internal(&mut state, "team1");
        assert!(!state.is_friends_with_ref("team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_FRIENDS_WITH_THE_REF);
    }
}
