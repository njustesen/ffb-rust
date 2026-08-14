/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.KnuckleDustersHandler`.
/// Extends mixed KnuckleDustersHandler with BB2020 PlayerSelector (own team RESERVE).
/// Selects 1 random player on the praying team, marks prayer, and grants Mighty Blow (+1).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020PlayerSelector;
use crate::inducements::mixed::prayers::select_player_prayer_handler as select_player_base;
use crate::inducements::mixed::prayers::{knuckle_dusters_handler as base, prayer_handler::PrayerHandler};
use crate::prayer_state::PrayerState;

pub struct KnuckleDustersHandler;

impl KnuckleDustersHandler {
    pub fn new() -> Self { Self }
}

impl Default for KnuckleDustersHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for KnuckleDustersHandler {
    fn handled_prayer_name(&self) -> &'static str { "KNUCKLE_DUSTERS" }
    fn animation_type(&self) -> AnimationType { base::animation_type() }
    fn get_name(&self) -> &'static str { "KnuckleDustersHandler" }
    /// Java `DialogPrayerHandler.initEffect`: build the eligible list, report the prayer wasted
    /// and advance if it is empty, otherwise SHOW A DIALOG and wait (`handled()` == false for
    /// `SelectPlayerPrayerHandler`). The coach picks — there is NO shuffle on this path.
    fn init_effect(&self, _prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        // Java: reports.add(new ReportPrayerWasted(...)) on the empty branch (report infra deferred).
        self.eligible_dialog_players(game, team_id).is_empty()
    }

    fn dialog_choice_mode(&self) -> Option<&'static str> { Some("KNUCKLE_DUSTERS") }

    fn eligible_dialog_players(&self, game: &Game, team_id: &str) -> Vec<String> {
        select_player_base::eligible_players_for_dialog(
            game, team_id, &[SkillId::MightyBlow], &BB2020PlayerSelector::new())
    }

    fn apply_selection(&self, _prayer_state: &mut PrayerState, game: &mut Game, player_id: &str) {
        select_player_base::apply_selection_select_player(game, player_id, "KNUCKLE_DUSTERS");
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        base::remove_effect_internal(game, team_id, &BB2020PlayerSelector::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn handles_prayer_knuckle_dusters() {
        let h = KnuckleDustersHandler;
        assert!(h.handles_prayer("KNUCKLE_DUSTERS"));
        assert!(!h.handles_prayer("STILETTO"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = KnuckleDustersHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_KNUCKLE_DUSTERS);
    }

    #[test]
    fn init_effect_returns_true() {
        let h = KnuckleDustersHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    /// Java's `SelectPlayerPrayerHandler` grants nothing in `initEffect` — it opens a dialog and
    /// waits (`handled()` == false); the skill lands in `applySelection` on the COACH's pick.
    /// Granting it during `initEffect` (the old random-selection route) both chose a different
    /// player than Java and drew from the Collections stream Java never touches here.
    #[test]
    fn init_effect_opens_a_dialog_and_grants_nothing_yet() {
        let h = KnuckleDustersHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        assert!(!h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"),
            "an eligible player exists → dialog pending, step must wait");
        assert_eq!(h.dialog_choice_mode(), Some("KNUCKLE_DUSTERS"));
        assert_eq!(h.eligible_dialog_players(&game, "home"), vec!["h1".to_string()]);
        assert!(!game.player("h1").unwrap().has_skill(SkillId::MightyBlow));
    }

    #[test]
    fn apply_selection_grants_mighty_blow_to_the_chosen_player() {
        let h = KnuckleDustersHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        h.apply_selection(&mut state, &mut game, "h1");
        assert!(game.player("h1").unwrap().has_skill(SkillId::MightyBlow));
    }
    #[test]
    fn does_not_handle_other_prayers() {
        let h = KnuckleDustersHandler;
        assert!(!h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer(""));
    }
}
