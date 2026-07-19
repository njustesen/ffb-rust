/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepFumbleTtmPass`.
///
/// When a TTM throw is fumbled: places the thrown player at the fumble coordinate,
/// restores their state, clears the defender, then resets the coordinate parameter.
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepFumbleTtmPass` (bb2016/ttm).
pub struct StepFumbleTtmPass {
    /// Java: `fThrownPlayerCoordinate`
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fThrownPlayerState`
    thrown_player_state: Option<PlayerState>,
}

impl StepFumbleTtmPass {
    pub fn new() -> Self {
        Self {
            thrown_player_coordinate: None,
            thrown_player_id: None,
            thrown_player_state: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if let (Some(player_id), Some(coord), Some(state)) = (
            &self.thrown_player_id,
            self.thrown_player_coordinate,
            self.thrown_player_state,
        ) {
            // Java: fThrownPlayerState.getId() > 0 (raw id, not just the base component)
            if state.id() > 0 && game.player(player_id).is_some() {
                game.field_model.set_player_coordinate(player_id, coord);
                // Java: sets game.getDefender()'s state — defender == thrown player in TTM
                if let Some(defender_id) = game.defender_id.clone() {
                    game.field_model.set_player_state(&defender_id, state);
                }
                game.defender_id = None;
            }
        }
        // Java: publishParameter(ThrownPlayerCoordinate, null) — avoid reset in end step
        StepOutcome::next()
            .publish(StepParameter::ThrownPlayerCoordinate(None))
    }
}

impl Default for StepFumbleTtmPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepFumbleTtmPass {
    fn id(&self) -> StepId { StepId::FumbleTtmPass }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerCoordinate(c) => { self.thrown_player_coordinate = *c; false }
            StepParameter::ThrownPlayerId(id)        => { self.thrown_player_id = id.clone(); true }
            StepParameter::ThrownPlayerState(s)      => { self.thrown_player_state = Some(*s); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE, PS_UNKNOWN};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, state_base: u32) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state_base));
    }

    #[test]
    fn id_is_fumble_ttm_pass() {
        assert_eq!(StepFumbleTtmPass::new().id(), StepId::FumbleTtmPass);
    }

    #[test]
    fn always_publishes_coordinate_reset() {
        let mut step = StepFumbleTtmPass::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_reset = outcome.published.iter().any(|p| {
            matches!(p, StepParameter::ThrownPlayerCoordinate(None))
        });
        assert!(has_reset);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn no_thrown_player_clears_nothing() {
        let mut step = StepFumbleTtmPass::new();
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        game.defender_id = Some("p1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // defender_id remains because no thrown_player_id was set
        assert!(game.defender_id.is_some());
    }

    #[test]
    fn unknown_state_zero_skips_placement() {
        let mut step = StepFumbleTtmPass::new();
        let target_coord = FieldCoordinate::new(10, 7);
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::ThrownPlayerCoordinate(Some(target_coord)));
        step.set_parameter(&StepParameter::ThrownPlayerState(PlayerState::new(PS_UNKNOWN)));
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        game.defender_id = Some("p1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // state.base() == 0 → skip, defender_id still set
        assert!(game.defender_id.is_some());
    }

    #[test]
    fn nonzero_raw_id_with_zero_base_still_places() {
        // Java guard is `fThrownPlayerState.getId() > 0`, which checks the FULL raw
        // encoded int (base state + flag bits), not just the low-byte base state.
        // A PlayerState with base == PS_UNKNOWN (0) but a nonzero flag bit set still
        // has getId() > 0, so the placement/defender-clear logic must still run.
        let mut step = StepFumbleTtmPass::new();
        let target_coord = FieldCoordinate::new(10, 7);
        // base = PS_UNKNOWN (0), but raw id is nonzero due to a set flag bit.
        let state_with_zero_base_nonzero_id = PlayerState::new(0x100);
        assert_eq!(state_with_zero_base_nonzero_id.base(), 0);
        assert!(state_with_zero_base_nonzero_id.id() > 0);
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::ThrownPlayerCoordinate(Some(target_coord)));
        step.set_parameter(&StepParameter::ThrownPlayerState(state_with_zero_base_nonzero_id));
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        game.defender_id = Some("p1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let coord = game.field_model.player_coordinate("p1").unwrap();
        assert_eq!(coord, target_coord);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn with_valid_state_places_and_clears_defender() {
        let mut step = StepFumbleTtmPass::new();
        let target_coord = FieldCoordinate::new(10, 7);
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::ThrownPlayerCoordinate(Some(target_coord)));
        step.set_parameter(&StepParameter::ThrownPlayerState(PlayerState::new(PS_PRONE)));
        let mut game = make_game();
        add_player(&mut game, "p1", PS_STANDING);
        game.defender_id = Some("p1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // Coordinate was updated
        let coord = game.field_model.player_coordinate("p1").unwrap();
        assert_eq!(coord, target_coord);
        // Defender cleared
        assert!(game.defender_id.is_none());
        // State updated to PRONE
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
    }
}
