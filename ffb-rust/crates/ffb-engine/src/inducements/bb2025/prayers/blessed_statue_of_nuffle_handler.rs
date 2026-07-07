/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.BlessedStatueOfNuffleHandler`.
/// Extends mixed BlessedStatueOfNuffleHandler with BB2025 PlayerSelector (own team RESERVE).
/// Selects 1 random player on the praying team, marks prayer, and grants Pro.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2025::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::blessed_statue_of_nuffle_handler::{self, PRAYER_NAME};
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct BlessedStatueOfNuffleHandler;

impl BlessedStatueOfNuffleHandler {
    pub fn new() -> Self { Self }
}

impl Default for BlessedStatueOfNuffleHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for BlessedStatueOfNuffleHandler {
    fn handled_prayer_name(&self) -> &'static str { PRAYER_NAME }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE }
    fn get_name(&self) -> &'static str { "BlessedStatueOfNuffleHandler" }

    /// Java: initEffect — selects 1 RESERVE player on the praying team, grants Pro.
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        blessed_statue_of_nuffle_handler::init_effect(prayer_state, game, rng, team_id, &PlayerSelector::new())
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        blessed_statue_of_nuffle_handler::remove_effect_internal(game, team_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_reserve_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            player_status: ffb_model::model::player_status::PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn handles_prayer_blessed_statue_of_nuffle() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(h.handles_prayer("BLESSED_STATUE_OF_NUFFLE"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = BlessedStatueOfNuffleHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn init_effect_grants_pro_to_reserve_player() {
        let h = BlessedStatueOfNuffleHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(game.player("h1").unwrap().has_skill(SkillId::Pro));
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = BlessedStatueOfNuffleHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE);
    }
}
