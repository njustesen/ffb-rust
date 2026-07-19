use ffb_model::enums::{ApothecaryMode, TurnMode, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{
    drop_player_no_sph, handle_injury_by_name, injury_type_causes_turnover,
};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepFallDown.
///
/// Drops the acting player after a failed dodge/GFI/jump.
/// Like the BB2025 version but adds a blood lust branch: if the acting player
/// is suffering blood lust, clear move squares and move them to RESERVE (the box).
///
/// Expects: `INJURY_TYPE_NAME` (stored as class name string), `COORDINATE_FROM`.
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

        // Java: publishParameters(UtilServerInjury.dropPlayer(this, actingPlayer, ATTACKER))
        // — the 3-arg overload, which defaults `eligibleForSafePairOfHands = false`. A
        // falling player (failed dodge/GFI/jump) never gets a Safe Pair of Hands reroll
        // offer for the ball they drop.
        let drop_params = drop_player_no_sph(game, &player_id);

        // Java: if (actingPlayer.isSufferingBloodLust())
        if game.acting_player.suffering_blood_lust {
            // Java: game.getFieldModel().clearMoveSquares()
            game.field_model.move_squares.clear();
            // Java: playerState.changeBase(PlayerState.RESERVE)
            if let Some(state) = game.field_model.player_state(&player_id) {
                game.field_model.set_player_state(&player_id, state.change_base(PS_RESERVE));
            }
            UtilBox::put_player_into_box(game, &player_id);
        }

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
    use ffb_model::enums::{Rules, PS_STANDING, PS_RESERVE};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::{FieldCoordinate, MoveSquare};
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
    fn blood_lust_sets_player_state_to_reserve() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_RESERVE);
    }

    #[test]
    fn blood_lust_clears_move_squares() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(3, 3), 0, 0));
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn no_blood_lust_does_not_change_state_to_reserve() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        game.acting_player.suffering_blood_lust = false;
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("p1").unwrap();
        assert_ne!(state.base(), PS_RESERVE);
    }

    /// Regression test: Java's `dropPlayer(this, actingPlayer, ATTACKER)` call is the 3-arg
    /// overload, which defaults `eligibleForSafePairOfHands = false` — so a player who drops
    /// the ball by falling down (failed dodge/GFI/jump) never gets a `DROPPED_BALL_CARRIER`
    /// (Safe Pair of Hands reroll offer) parameter published. A previous version of this file
    /// called the lower-level `drop_player(..., true)`, incorrectly enabling that reroll offer.
    #[test]
    fn falling_with_ball_does_not_offer_safe_pair_of_hands() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !out.published.iter().any(|p| matches!(p, StepParameter::DroppedBallCarrier(_))),
            "falling down must never offer Safe Pair of Hands"
        );
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
