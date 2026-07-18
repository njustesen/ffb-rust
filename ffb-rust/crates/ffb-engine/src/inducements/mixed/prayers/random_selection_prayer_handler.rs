/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.RandomSelectionPrayerHandler`.
/// Abstract handler that randomly selects N players and applies a prayer enhancement to each.
///
/// Java:
///   initEffect: selects players via selector().selectPlayers, calls fieldModel.addPrayerEnhancements per player.
///   removeEffectInternal: calls enhancementRemover.removeEnhancement.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::inducements::mixed::prayers::enhancement_remover::EnhancementRemover;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::prayer_player_effect::{apply_prayer_player_effect, remove_prayer_player_effect};
use crate::prayer_state::PrayerState;

/// Java: RandomSelectionPrayerHandler.initEffect(GameState, Team)
/// Selects `nr_of_players` players via the given selector and marks the prayer as active on each.
///
/// Returns true (prayer applied immediately, no dialog needed).
pub fn init_effect_random_selection(
    _prayer_state: &mut PrayerState,
    game: &mut Game,
    rng: &mut GameRng,
    team_id: &str,
    prayer_name: &str,
    nr_of_players: i32,
    selector: &dyn PlayerSelector,
    added_skills: &[SkillId],
) -> bool {
    let selected = selector.select_players(game, team_id, nr_of_players, rng, added_skills);
    for player_id in &selected {
        game.field_model.add_prayer_enhancement(player_id, prayer_name);
        apply_prayer_player_effect(game, player_id, prayer_name);
    }
    // Java: if players.isEmpty() → ReportPrayerWasted; not yet ported (report infra)
    true
}

/// Java: RandomSelectionPrayerHandler.removeEffectInternal(GameState, Team)
/// Removes the prayer enhancement tracking from all affected players.
/// Java: `enhancementRemover.removeEnhancement(gameState, team, selector(), handledPrayer())` —
/// the `selector` determines which team (own or opponent) was actually affected.
pub fn remove_effect_internal_random_selection(
    game: &mut Game,
    team_id: &str,
    prayer_name: &str,
    selector: &dyn PlayerSelector,
) {
    let remover = EnhancementRemover::new();
    remover.remove_enhancement(game, team_id, prayer_name, selector);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING, PS_RESERVE};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;
    use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020Selector;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player_reserve(game: &mut Game, team_id: &str, id: &str) {
        use ffb_model::model::player::Player;
        use ffb_model::model::player_status::PlayerStatus;
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            is_big_guy: false,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect_random_selection(&mut state, &mut game, &mut GameRng::new(0), "home", "STILETTO", 1, &stub, &[]));
    }

    #[test]
    fn init_effect_marks_prayer_on_selected_players() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::StartGame;
        add_player_reserve(&mut game, "home", "h1");
        let selector = BB2020Selector::new();
        init_effect_random_selection(&mut state, &mut game, &mut GameRng::new(0), "home", "STILETTO", 1, &selector, &[]);
        assert!(game.field_model.has_prayer_enhancement("h1", "STILETTO"));
    }

    #[test]
    fn remove_effect_clears_prayer_tracking() {
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::StartGame;
        add_player_reserve(&mut game, "home", "h1");
        game.field_model.add_prayer_enhancement("h1", "STILETTO");
        let selector = BB2020Selector::new();
        remove_effect_internal_random_selection(&mut game, "home", "STILETTO", &selector);
        assert!(!game.field_model.has_prayer_enhancement("h1", "STILETTO"));
    }

    #[test]
    fn init_effect_handles_zero_eligible_players() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        // No players added → empty selection
        let stub = StubPlayerSelector;
        let result = init_effect_random_selection(&mut state, &mut game, &mut GameRng::new(0), "home", "BAD_HABITS", 3, &stub, &[]);
        assert!(result);
    }

    #[test]
    fn remove_effect_on_player_without_enhancement_is_safe() {
        let mut game = make_game();
        // Player h2 has no enhancement — remove should not panic.
        add_player_reserve(&mut game, "home", "h2");
        remove_effect_internal_random_selection(&mut game, "home", "GREASY_CLEATS", &StubPlayerSelector);
        assert!(!game.field_model.has_prayer_enhancement("h2", "GREASY_CLEATS"));
    }

    #[test]
    fn init_effect_prayer_name_is_used_as_enhancement_key() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::StartGame;
        add_player_reserve(&mut game, "home", "h3");
        let selector = BB2020Selector::new();
        init_effect_random_selection(&mut state, &mut game, &mut GameRng::new(0), "home", "KNUCKLE_DUSTERS", 1, &selector, &[]);
        assert!(game.field_model.has_prayer_enhancement("h3", "KNUCKLE_DUSTERS"));
    }
}
