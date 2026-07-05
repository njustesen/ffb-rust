/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.BlessedStatueOfNuffleHandler`.
/// Extends SelectPlayerPrayerHandler — coach selects a player who gains Pro.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use crate::inducements::mixed::prayers::enhancement_remover::EnhancementRemover;
use crate::prayer_state::PrayerState;

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE
}

pub fn init_effect(_prayer_state: &mut PrayerState, _game: &Game, _team_id: &str) -> bool {
    // DEFERRED(prayer-dialog): needs SelectPlayerPrayerHandler + dialog + fieldModel.addPrayerEnhancements
    false
}

pub fn remove_effect_internal(_prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
    EnhancementRemover::new().remove_enhancement(game, team_id, "BLESSED_STATUE_OF_NUFFLE");
}

pub fn apply_selection(_prayer_state: &mut PrayerState, _game: &mut Game, _player_id: &str) {
    // DEFERRED(prayer-dialog): needs fieldModel.addPrayerEnhancements for BLESSED_STATUE_OF_NUFFLE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE);
    }

    #[test]
    fn init_effect_returns_false_pending_dialog() {
        let mut state = PrayerState::new();
        let game = make_game();
        assert!(!init_effect(&mut state, &game, "team1"));
    }

    #[test]
    fn apply_selection_is_noop() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        apply_selection(&mut state, &mut game, "player1");
    }

    #[test]
    fn remove_effect_clears_enhancement() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut state = PrayerState::new();
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
});
        game.field_model.add_prayer_enhancement("p1", "BLESSED_STATUE_OF_NUFFLE");
        assert!(game.field_model.has_prayer_enhancement("p1", "BLESSED_STATUE_OF_NUFFLE"));
        remove_effect_internal(&mut state, &mut game, "home");
        assert!(!game.field_model.has_prayer_enhancement("p1", "BLESSED_STATUE_OF_NUFFLE"));
    }
}
