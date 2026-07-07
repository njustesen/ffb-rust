use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{
    drop_player, handle_injury_by_name, injury_type_causes_turnover,
};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepFallDown.
///
/// Drops the acting player after a failed dodge/GFI/jump.
///
/// Expects: `INJURY_TYPE` (stored as InjuryTypeName string), `COORDINATE_FROM`.
/// Sets: `INJURY_RESULT`, `END_TURN` (if injury type causes turnover and not PASS_BLOCK),
///       plus the drop_player parameters (CATCH_SCATTER_THROW_IN_MODE, END_TURN from ball).
pub struct StepFallDown {
    /// Java: fInjuryType (InjuryTypeServer<?>) — stored as class name string.
    pub injury_type_name: Option<String>,
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
}

impl StepFallDown {
    pub fn new() -> Self { Self { injury_type_name: None, coordinate_from: None } }
}

impl Default for StepFallDown {
    fn default() -> Self { Self::new() }
}

impl Step for StepFallDown {
    fn id(&self) -> StepId { StepId::FallDown }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InjuryTypeName(v) => { self.injury_type_name = Some(v.clone()); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            _ => false,
        }
    }
}

impl StepFallDown {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let coord = game.field_model
            .player_coordinate(&player_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        let injury_type_name = self.injury_type_name
            .as_deref()
            .unwrap_or("InjuryTypeDropGFI");

        // Java: UtilServerInjury.handleInjury(this, fInjuryType, null, actingPlayer, coord, from, null, ATTACKER)
        let injury_result = handle_injury_by_name(
            game, rng,
            injury_type_name,
            None, &player_id,
            coord, self.coordinate_from,
            None, ApothecaryMode::Attacker,
        );

        // Java: publishParameters(UtilServerInjury.dropPlayer(this, actingPlayer, ATTACKER, true))
        let drop_params = drop_player(game, &player_id, true);

        // Java: if (fInjuryType.fallingDownCausesTurnover() && getTurnMode() != PASS_BLOCK)
        let causes_turnover = injury_type_causes_turnover(injury_type_name);
        let is_pass_block = game.turn_mode == TurnMode::PassBlock;

        let mut outcome = StepOutcome::next();
        for p in drop_params {
            outcome = outcome.publish(p);
        }
        outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
        if causes_turnover && !is_pass_block {
            outcome = outcome.publish(StepParameter::EndTurn(true));
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_acting_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_injury_result() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn publishes_end_turn_for_gfi_drop() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn no_end_turn_when_pass_block() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        add_acting_player(&mut game, "p1");
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_injury_type_name_accepted() {
        let mut step = StepFallDown::new();
        assert!(step.set_parameter(&StepParameter::InjuryTypeName("InjuryTypeDropDodge".into())));
        assert_eq!(step.injury_type_name.as_deref(), Some("InjuryTypeDropDodge"));
    }

    #[test]
    fn set_parameter_coordinate_from_accepted() {
        let mut step = StepFallDown::new();
        let coord = FieldCoordinate::new(3, 7);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepFallDown::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
