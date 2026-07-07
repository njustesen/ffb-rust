/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.GreasyCleatsHandler`.
/// Extends mixed GreasyCleatsHandler with BB2025 OpponentPlayerSelector (opponent RESERVE).
/// Selects 1 random opponent player, marks prayer, and applies -1 MA via apply_prayer_player_effect.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2025::prayers::opponent_player_selector::OpponentPlayerSelector;
use crate::inducements::mixed::prayers::greasy_cleats_handler::{self, PRAYER_NAME};
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct GreasyCleatsHandler;

impl GreasyCleatsHandler {
    pub fn new() -> Self { Self }
}

impl Default for GreasyCleatsHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for GreasyCleatsHandler {
    fn handled_prayer_name(&self) -> &'static str { PRAYER_NAME }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_GREASY_CLEATS }
    fn get_name(&self) -> &'static str { "GreasyCleatsHandler" }

    /// Java: initEffect — selects 1 RESERVE opponent player, marks prayer, applies -1 MA.
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        greasy_cleats_handler::init_effect(prayer_state, game, rng, team_id, &OpponentPlayerSelector::new())
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        greasy_cleats_handler::remove_effect_internal(game, team_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn handles_prayer_greasy_cleats() {
        assert!(GreasyCleatsHandler.handles_prayer("GREASY_CLEATS"));
        assert!(!GreasyCleatsHandler.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(GreasyCleatsHandler.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(GreasyCleatsHandler.animation_type(), AnimationType::PRAYER_GREASY_CLEATS);
    }

    #[test]
    fn init_effect_selects_from_opponent_team() {
        use ffb_model::enums::{PS_RESERVE, PlayerState};
        use ffb_model::model::player::Player;
        use ffb_model::enums::PlayerType;
        let mut game = make_game();
        game.team_away.players.push(Player {
            id: "a1".into(), name: "a1".into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            is_big_guy: false,
            player_status: ffb_model::model::player_status::PlayerStatus::ACTIVE,
            ..Default::default()
});
        game.field_model.set_player_state("a1", PlayerState::new(PS_RESERVE));
        let mut state = PrayerState::new();
        // Home team gets greasy cleats — should affect away player
        GreasyCleatsHandler.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(game.field_model.has_prayer_enhancement("a1", "GREASY_CLEATS"));
    }
}
