/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.BlessedStatueOfNuffleHandler`.
/// Java BB2025 extends RandomSelectionPrayerHandler DIRECTLY (not the mixed
/// BlessedStatueOfNuffleHandler) and handles Prayer.BLESSING_OF_NUFFLE — a different prayer id
/// than bb2020's BLESSED_STATUE_OF_NUFFLE. Selects 1 random player, marks prayer, grants Pro.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use crate::inducements::bb2025::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

// Bug (fixed, #11): this handler previously reused the mixed module's
// PRAYER_NAME = "BLESSED_STATUE_OF_NUFFLE" for dispatch and enhancement tagging, but the
// bb2025 prayer catalog (data/prayers/bb2025_prayers.json and Java bb2025 Prayer enum) uses
// BLESSING_OF_NUFFLE — the handler could never match the bb2025 prayer.
pub const PRAYER_NAME: &str = "BLESSING_OF_NUFFLE";

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

    /// Java: initEffect (RandomSelectionPrayerHandler) — selects 1 RESERVE player on the
    /// praying team, marks BLESSING_OF_NUFFLE, grants Pro.
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, 1, &PlayerSelector::new(), &[SkillId::Pro])
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        remove_effect_internal_random_selection(game, team_id, PRAYER_NAME, &PlayerSelector::new());
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
    // Bug (fixed, #11): previously asserted the bb2020 name BLESSED_STATUE_OF_NUFFLE; the
    // bb2025 prayer id is BLESSING_OF_NUFFLE (Java bb2025 Prayer enum / bb2025_prayers.json).
    fn handles_prayer_blessing_of_nuffle() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(h.handles_prayer("BLESSING_OF_NUFFLE"));
        assert!(!h.handles_prayer("BLESSED_STATUE_OF_NUFFLE"));
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
    #[test]
    fn does_not_handle_other_prayers() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(!h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer(""));
    }
}
