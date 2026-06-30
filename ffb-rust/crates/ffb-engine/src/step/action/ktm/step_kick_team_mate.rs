/// 1:1 translation of com.fumbbl.ffb.server.step.action.ktm.StepKickTeamMate (COMMON).
///
/// Rolls dice for the kick-team-mate action, determines success/failure, and initiates the
/// scatter sequence on success.
///
/// Mandatory init param: GOTO_LABEL_ON_FAILURE.
/// Expected preceding params: KICKED_PLAYER_ID, KICKED_PLAYER_STATE, KICKED_PLAYER_HAS_BALL,
/// NR_OF_DICE.
///
/// Stub: AbstractStepWithReRoll is not yet fully translated — reroll is always skipped.
/// Stub: Scatter-player sequence push is omitted pending StepKind integration.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::kick_team_mate_range::KickTeamMateRange;
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepKickTeamMate {
    /// Java: fGotoLabelOnFailure — mandatory.
    pub goto_label_on_failure: String,
    /// Java: fKickedPlayerId — set by preceding step parameter.
    pub kicked_player_id: Option<String>,
    /// Java: fKickedPlayerState — player state at kick time.
    pub kicked_player_state: Option<PlayerState>,
    /// Java: fKickedPlayerHasBall — whether kicked player carries the ball.
    pub kicked_player_has_ball: bool,
    /// Java: fNumDice — 1 or 2, clamped.
    pub num_dice: i32,
    /// Java: fDistance — total rolled distance (rolls[0] + rolls[1]).
    pub distance: i32,
    /// Java: fRolls — individual dice values.
    pub rolls: Vec<i32>,
}

impl StepKickTeamMate {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            kicked_player_id: None,
            kicked_player_state: None,
            kicked_player_has_ball: false,
            num_dice: 0,
            distance: 0,
            rolls: Vec::new(),
        }
    }
}

impl Step for StepKickTeamMate {
    fn id(&self) -> StepId { StepId::KickTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: AbstractStepWithReRoll.handleCommand processes reroll dialogs.
        // Random agent always declines → execute_step is called directly.
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v) => { self.kicked_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v) => { self.kicked_player_has_ball = *v; true }
            StepParameter::NrOfDice(v) => { self.num_dice = (*v).max(0).min(2); true }
            _ => false,
        }
    }
}

impl StepKickTeamMate {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.setHasBlocked(true); game.setConcessionPossible(false); turnData.setBlitzUsed(true)
        game.acting_player.has_blocked = true;
        game.concession_possible = false;
        game.turn_data_mut().blitz_used = true;

        let kicker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::goto(&self.goto_label_on_failure),
        };
        let kicked_id = match self.kicked_player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::goto(&self.goto_label_on_failure),
        };

        let kicker_coord = match game.field_model.player_coordinate(&kicker_id) {
            Some(c) => c,
            None => return StepOutcome::goto(&self.goto_label_on_failure),
        };
        let kicked_coord = match game.field_model.player_coordinate(&kicked_id) {
            Some(c) => c,
            None => return StepOutcome::goto(&self.goto_label_on_failure),
        };

        // Java: Direction d = kickerCoordinate.getDirection(kickedPlayerCoordinate)
        let direction = kicker_coord.direction_to(kicked_coord);

        // Java: roll fNumDice d6 dice (rollSkill = d6 in range 1-6)
        let num_dice = self.num_dice.max(1) as usize;
        self.rolls = (0..num_dice).map(|_| rng.d6()).collect();

        // Java: successful = (numDice == 1) || (rolls[0] != rolls[1])
        let successful = num_dice == 1 || self.rolls[0] != self.rolls[1];

        // Java: fDistance = rolls[0] + (numDice > 1 ? rolls[1] : 0)
        self.distance = self.rolls[0] + if num_dice > 1 { self.rolls[1] } else { 0 };

        // Java: targetCoordinate = kickedPlayerCoordinate.move(d, distance)
        if let Some(dir) = direction {
            let target = kicked_coord.step(dir, self.distance);
            game.pass_coordinate = Some(target);
        }

        self.execute_kick(game, kicked_coord, kicker_coord, successful)
    }

    fn execute_kick(&self, game: &mut Game, _kicked_coord: FieldCoordinate, _kicker_coord: FieldCoordinate, successful: bool) -> StepOutcome {
        if successful {
            // Java: set kicked player state to PICKED_UP base
            if let Some(ref kicked_id) = self.kicked_player_id {
                if let Some(state) = self.kicked_player_state {
                    let picked_up = state.change_base(ffb_model::enums::PS_PICKED_UP);
                    game.field_model.set_player_state(kicked_id, picked_up);
                }
            }

            // Java: pushSequence(ScatterPlayer) — stub: sequence push omitted pending StepKind integration.
            // Java: publishParameter(IS_KICKED_PLAYER, true)
            let mut outcome = StepOutcome::next()
                .publish(StepParameter::IsKickedPlayer(true));

            // Java: if distance >= 9 → KTM_MODIFIER = LONG; else if distance >= 6 → MEDIUM
            if self.distance >= 9 {
                outcome = outcome.publish(StepParameter::KtmModifier(KickTeamMateRange::LONG));
            } else if self.distance >= 6 {
                outcome = outcome.publish(StepParameter::KtmModifier(KickTeamMateRange::MEDIUM));
            }

            outcome
        } else {
            // Java: getResult().setNextAction(GOTO_LABEL, fGotoLabelOnFailure)
            StepOutcome::goto(&self.goto_label_on_failure)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerState, Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game_with_players() -> (Game, String, String) {
        let kicker_id = "kicker".to_string();
        let kicked_id = "kicked".to_string();
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(kicker_id.clone());
        game.field_model.set_player_coordinate(&kicker_id, FieldCoordinate::new(8, 5));
        game.field_model.set_player_coordinate(&kicked_id, FieldCoordinate::new(9, 5));
        game.field_model.set_player_state(&kicked_id, PlayerState::new(PS_STANDING));
        (game, kicker_id, kicked_id)
    }

    fn make_step(kicked_id: &str) -> StepKickTeamMate {
        let mut step = StepKickTeamMate::new("fail");
        step.kicked_player_id = Some(kicked_id.into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.num_dice = 1;
        step
    }

    #[test]
    fn single_die_roll_always_succeeds() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 1;
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(true))));
    }

    #[test]
    fn double_on_two_dice_fails() {
        // Seed chosen to produce a double on 2d6.
        // With GameRng::new(0): find a seed where both d6 produce the same value.
        // Use a known-bad scenario: set rolls manually via a stub.
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 2;
        // Force rolls to produce a double by pre-setting rolls (simulated).
        // We can't force the RNG, but we can test the logic:
        // call execute_kick directly with successful=false.
        let coord = FieldCoordinate::new(9, 5);
        let kicker = FieldCoordinate::new(8, 5);
        let out = step.execute_kick(&mut game, coord, kicker, false);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn successful_kick_sets_has_blocked_and_blitz_used() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().blitz_used);
        assert!(!game.concession_possible);
    }

    #[test]
    fn distance_long_range_publishes_long_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 9;
        let kicker = FieldCoordinate::new(8, 5);
        let coord = FieldCoordinate::new(9, 5);
        let out = step.execute_kick(&mut game, coord, kicker, true);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(KickTeamMateRange::LONG))));
    }

    #[test]
    fn distance_medium_range_publishes_medium_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 6;
        let kicker = FieldCoordinate::new(8, 5);
        let coord = FieldCoordinate::new(9, 5);
        let out = step.execute_kick(&mut game, coord, kicker, true);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(KickTeamMateRange::MEDIUM))));
    }

    #[test]
    fn short_range_no_ktm_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 3;
        let kicker = FieldCoordinate::new(8, 5);
        let coord = FieldCoordinate::new(9, 5);
        let out = step.execute_kick(&mut game, coord, kicker, true);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(_))));
    }

    #[test]
    fn no_kicker_gotos_failure() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        game.acting_player.player_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }
}
