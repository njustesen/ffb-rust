/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.IronManHandler`.
/// Extends SelectPlayerPrayerHandler — selects 1 random player who gains +1 AV.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::enhancement_remover::EnhancementRemover;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::prayer_player_effect::{apply_prayer_player_effect, remove_prayer_player_effect};
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "IRON_MAN";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_IRON_MAN
}

/// Java: initEffect — selects 1 RESERVE player on the praying team, marks prayer, grants +1 AV.
pub fn init_effect(prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str, selector: &dyn PlayerSelector) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, 1, selector, &[])
}

/// Java: applySelection — called from dialog response with the coach-chosen player.
/// In headless mode this is not called; the random selection in init_effect handles it.
pub fn apply_selection(_prayer_state: &mut PrayerState, game: &mut Game, player_id: &str) {
    game.field_model.add_prayer_enhancement(player_id, PRAYER_NAME);
    apply_prayer_player_effect(game, player_id, PRAYER_NAME);
}

pub fn remove_effect_internal(game: &mut Game, team_id: &str) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME);
}

pub fn remove_effect_for_player(game: &mut Game, player_id: &str) {
    remove_prayer_player_effect(game, player_id, PRAYER_NAME);
    game.field_model.remove_prayer_enhancement(player_id, PRAYER_NAME);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;
    use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020Selector;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_reserve_player(game: &mut Game, id: &str, armour: i32) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_IRON_MAN);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &stub));
    }

    #[test]
    fn init_effect_marks_prayer_on_selected_player() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1", 8);
        let selector = BB2020Selector::new();
        init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &selector);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }

    #[test]
    fn init_effect_applies_armour_boost() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1", 8);
        let selector = BB2020Selector::new();
        init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &selector);
        assert_eq!(game.player("h1").unwrap().armour_with_modifiers(), 9);
    }

    #[test]
    fn apply_selection_marks_and_boosts_player() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1", 8);
        apply_selection(&mut state, &mut game, "h1");
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
        assert_eq!(game.player("h1").unwrap().armour_with_modifiers(), 9);
    }

    #[test]
    fn remove_effect_clears_enhancement() {
        let mut game = make_game();
        add_reserve_player(&mut game, "h1", 8);
        game.field_model.add_prayer_enhancement("h1", PRAYER_NAME);
        remove_effect_internal(&mut game, "home");
        assert!(!game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }
}
