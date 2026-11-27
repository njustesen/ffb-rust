/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.BlessedStatueOfNuffleHandler`.
/// Java: extends the mixed `BlessedStatueOfNuffleHandler` (a `SelectPlayerPrayerHandler`) with the
/// BB2025 `PlayerSelector`, and handles `Prayer.BLESSED_STATUE_OF_NUFFLE` of the bb2025 enum. The
/// coach CHOOSES the player in a dialog; Pro is granted by `applySelection`.
///
/// (§Q2.8: this file used to handle a non-existent "BLESSING_OF_NUFFLE" with a random pick, so roll
/// 8 of the bb2025 prayer table had no handler at all.)
use ffb_model::model::animation_type::AnimationType;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2025::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::select_player_prayer_handler as select_player_base;
use crate::inducements::mixed::prayers::{blessed_statue_of_nuffle_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "BLESSED_STATUE_OF_NUFFLE";

pub struct BlessedStatueOfNuffleHandler;

impl BlessedStatueOfNuffleHandler {
    pub fn new() -> Self { Self }
}

impl Default for BlessedStatueOfNuffleHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for BlessedStatueOfNuffleHandler {
    fn handled_prayer_name(&self) -> &'static str { PRAYER_NAME }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "BlessedStatueOfNuffleHandler" }

    /// Java `DialogPrayerHandler.initEffect`: an empty eligible list reports the prayer wasted and
    /// advances; otherwise it shows the player-choice dialog and waits (no shuffle on this path).
    fn init_effect(&self, _prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        let players = self.eligible_dialog_players(game, team_id);
        let wasted = players.is_empty();
        if wasted {
            let display = ffb_model::inducement::bb2025::prayer::Prayer::for_enum_name(PRAYER_NAME)
                .map(|p| p.get_name().to_string());
            game.report_list.add(ffb_model::report::mixed::report_prayer_wasted::ReportPrayerWasted::new(display, None));
        } else {
            select_player_base::create_dialog(game, team_id,
                ffb_model::model::player_choice_mode::PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE, players);
        }
        wasted
    }

    fn dialog_choice_mode(&self) -> Option<&'static str> { Some("BLESSED_STATUE_OF_NUFFLE") }

    fn eligible_dialog_players(&self, game: &Game, team_id: &str) -> Vec<String> {
        select_player_base::eligible_players_for_dialog(game, team_id, &[SkillId::Pro], &PlayerSelector::new())
    }

    fn apply_selection(&self, _prayer_state: &mut PrayerState, game: &mut Game, player_id: &str) {
        select_player_base::apply_selection_select_player(game, player_id, PRAYER_NAME);
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(game, team_id, &PlayerSelector::new());
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
    fn handles_the_bb2025_blessed_statue_of_nuffle() {
        let h = BlessedStatueOfNuffleHandler;
        assert!(h.handles_prayer("BLESSED_STATUE_OF_NUFFLE"));
        assert!(!h.handles_prayer("BLESSING_OF_NUFFLE"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn with_nobody_eligible_the_prayer_is_wasted() {
        let h = BlessedStatueOfNuffleHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    /// Java SelectPlayerPrayerHandler: initEffect only opens the dialog; Pro comes with the choice.
    #[test]
    fn init_effect_opens_a_dialog_and_apply_selection_grants_pro() {
        let h = BlessedStatueOfNuffleHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
        assert_eq!(h.dialog_choice_mode(), Some("BLESSED_STATUE_OF_NUFFLE"));
        assert!(!game.player("h1").unwrap().has_skill(SkillId::Pro));
        h.apply_selection(&mut state, &mut game, "h1");
        assert!(game.player("h1").unwrap().has_skill(SkillId::Pro));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = BlessedStatueOfNuffleHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE);
    }
}
