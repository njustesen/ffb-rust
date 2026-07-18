/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.BadHabitsHandler`.
/// Extends RandomSelectionPrayerHandler — selects D3 random opponent players and grants Loner (2+).
///
/// Effect: grants Loner (2+) skill to D3 selected opponent players via `apply_prayer_player_effect`.
/// Prayer enhancement tracking is fully implemented.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "BAD_HABITS";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_BAD_HABITS
}

/// Java: affectedPlayers(GameState) rolls D3.
pub fn affected_players(d3_roll: i32) -> i32 {
    d3_roll
}

pub fn init_effect(
    prayer_state: &mut PrayerState,
    game: &mut Game,
    rng: &mut GameRng,
    team_id: &str,
    d3_roll: i32,
    selector: &dyn PlayerSelector,
) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, affected_players(d3_roll), selector, &[SkillId::Loner])
}

pub fn remove_effect_internal(game: &mut Game, team_id: &str, selector: &dyn PlayerSelector) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME, selector);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_BAD_HABITS);
    }

    #[test]
    fn affected_players_returns_d3_roll() {
        assert_eq!(affected_players(2), 2);
        assert_eq!(affected_players(3), 3);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", 2, &stub));
    }

    #[test]
    fn remove_effect_is_callable() {
        let mut game = make_game();
        remove_effect_internal(&mut game, "home", &StubPlayerSelector);
    }

    /// Regression test: BAD_HABITS uses an opponent-team selector — the Loner enhancement
    /// is granted to the OPPOSING team's players (see BadHabitsHandler.java doc / affectedPlayers),
    /// so removal must clear the enhancement from the opponent's roster, not the praying team's.
    #[test]
    fn remove_effect_clears_enhancement_from_opponent_team() {
        use crate::inducements::bb2020::prayers::opponent_player_selector::OpponentPlayerSelector;
        use ffb_model::model::player::Player;
        use ffb_model::model::player_status::PlayerStatus;
        let mut game = make_game();
        game.team_away.players.push(Player {
            id: "a1".into(), name: "a1".into(), nr: 1,
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
        });
        game.field_model.add_prayer_enhancement("a1", PRAYER_NAME);
        let selector = OpponentPlayerSelector::new();
        // "home" is the praying team; enhancement lives on "a1" (the opponent's player).
        remove_effect_internal(&mut game, "home", &selector);
        assert!(!game.field_model.has_prayer_enhancement("a1", PRAYER_NAME));
    }

    #[test]
    fn prayer_name_constant_is_bad_habits() {
        assert_eq!(PRAYER_NAME, "BAD_HABITS");
    }

    #[test]
    fn affected_players_returns_one_for_roll_one() {
        assert_eq!(affected_players(1), 1);
    }
}
