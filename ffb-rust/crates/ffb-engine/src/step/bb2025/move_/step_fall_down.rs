use ffb_model::enums::{ApothecaryMode, TurnMode};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::injuryType::injury_type_ball_and_chain::InjuryTypeBallAndChain;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{
    drop_player_rng, handle_injury_by_name, injury_type_causes_turnover,
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

        // Java line 88 (bb2025 ONLY): `publishParameters(UtilServerInjury.dropPlayer(this,
        // actingPlayer.getPlayer(), ApothecaryMode.ATTACKER, TRUE))` — the FOUR-arg overload, so
        // eligibleForSafePairOfHands is **TRUE** here. The bb2016 and bb2020 twins call the
        // three-arg overload (false); only bb2025 opts in. Passing false here conflated this file
        // with its bb2020 twin and silently suppressed the Safe Pair of Hands offer for a carrier
        // who falls during a MOVE — Java raised the dialog and spent two sampler draws answering
        // it, Rust raised nothing, and the two agents' streams split for the rest of the game
        // (lizardman bb2025 seed 59 i=100: the star Boa Kon'ssstriktr, who has Safe Pair of Hands,
        // fell at (17,3) holding the ball; Java SKILL_USE@315 vs Rust activate@313).
        //
        // `drop_player_rng` is the full port. It does the `placedProneCausesInjuryRoll` (Ball &
        // Chain) branch — a falling Fanatic takes a chain injury instead of being placed prone —
        // AND the ball handling that Java puts OUTSIDE that if/else. This step used to inline the
        // injury half and return no drop parameters, which lost the ball handling exactly as
        // `StepHandleDropPlayerContext` did before it.
        let drop_params = drop_player_rng(
            game, rng, &player_id, true, ApothecaryMode::Attacker,
        );

        // Java: if (fInjuryType.fallingDownCausesTurnover() && getTurnMode() != PASS_BLOCK)
        let causes_turnover = injury_type_causes_turnover(injury_type_name);
        let is_pass_block = game.turn_mode == TurnMode::PassBlock;

        let mut outcome = StepOutcome::next();
        // Coverage event: the acting player fell down at `coord` (failed dodge/GFI/jump).
        outcome = outcome.with_event(ffb_model::events::GameEvent::PlayerFellDown {
            player_id: player_id.clone(),
            coord,
        });
        for p in drop_params {
            outcome = outcome.publish(p);
        }
        // Java line 88's dropPlayer publishes the Ball & Chain INJURY_RESULT (inside drop_params,
        // above) BEFORE line 89's fInjuryType result.
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

    /// Regression (lizardman bb2025 seed 59 i=100): Java's **bb2025** StepFallDown:88 calls the
    /// FOUR-arg `dropPlayer(..., ApothecaryMode.ATTACKER, true)`, so a carrier who falls during a
    /// MOVE is eligible for Safe Pair of Hands; the bb2016/bb2020 twins call the three-arg
    /// overload (false). Rust passed false here, conflating this file with its bb2020 twin, so no
    /// DROPPED_BALL_CARRIER was published, StepPlaceBall bailed at its `playerId == null` guard,
    /// and the Safe Pair of Hands dialog Java shows never appeared — costing two sampler draws.
    #[test]
    fn falling_carrier_is_eligible_for_safe_pair_of_hands() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        // Give p1 the ball, standing on it: UtilPlayer::has_ball needs in-play, not moving, same square.
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;

        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeDropGFI".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert!(
            out.published.iter().any(|p| matches!(
                p,
                StepParameter::DroppedBallCarrier(Some(id)) if id == "p1"
            )),
            "a falling ball carrier must publish DROPPED_BALL_CARRIER in bb2025 so StepPlaceBall              can raise the Safe Pair of Hands offer (Java bb2025 StepFallDown:88 passes true)"
        );
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
    fn seed27_ball_and_chain_crowd_push_no_turnover_and_rolls_bc_injury() {
        // goblin seed 27 i=105: a Ball & Chain Fanatic (placedProneCausesInjuryRoll) wanders off the
        // pitch on its compulsory move → InjuryTypeCrowdPush. Two requirements matching Java:
        //  (1) InjuryTypeCrowdPush does NOT cause a turnover (fallingDownCausesTurnover=false), so
        //      StepFallDown must NOT publish END_TURN — the Fanatic's own wander is not a turnover.
        //      (Regression: the injury type wasn't propagated, StepFallDown defaulted to
        //      InjuryTypeDropGFI which DOES turn over.)
        //  (2) dropPlayer for a B&C player rolls InjuryTypeBallAndChain (Java dropPlayer:339-342) —
        //      an InjuryResult must be published (the chain injury), not a plain prone-drop.
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        let mut p = Player {
            id: "fanatic".into(), name: "fanatic".into(), nr: 1, position_id: "fanatic".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 7, agility: 1, passing: 0, armour: 8,
            ..Default::default()
        };
        p.starting_skills.push(ffb_model::model::skill_def::SkillWithValue::new(SkillId::BallAndChain));
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("fanatic", FieldCoordinate::new(0, 3));
        game.field_model.set_player_state("fanatic", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some("fanatic".into());

        let mut step = StepFallDown::new();
        step.injury_type_name = Some("InjuryTypeCrowdPush".into());
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))),
            "a Ball & Chain crowd-push fall must NOT publish END_TURN (InjuryTypeCrowdPush is not a turnover)");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))),
            "a Ball & Chain player's fall must roll and publish an InjuryResult (InjuryTypeBallAndChain)");
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
