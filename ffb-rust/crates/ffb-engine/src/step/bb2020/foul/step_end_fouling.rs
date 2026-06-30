use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::{EndPlayerAction, Move, Select};
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2020::move_::MoveParams;
use crate::step::generator::bb2020::select::SelectParams;
use crate::step::util_server_steps;

/// Final step of the foul sequence (BB2020). Consumes EndTurn/EndPlayerAction/BloodLustAction.
/// Handles move-after-foul (SneakyGit) and bloodlust follow-up.
/// No check_forgo field (BB2020 EndPlayerAction does not have it).
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.foul.StepEndFouling.
pub struct StepEndFouling {
    pub end_turn: bool,
    pub end_player_action: bool,
    pub bloodlust_action: Option<PlayerAction>,
}

impl StepEndFouling {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            bloodlust_action: None,
        }
    }
}

impl Default for StepEndFouling {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFouling {
    fn id(&self) -> StepId { StepId::EndFouling }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            _ => false,
        }
    }
}

impl StepEndFouling {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: ActingPlayer actingPlayer = game.getActingPlayer()
        // Java: Player player = actingPlayer.getPlayer()
        // Java: boolean isOnPitch = FieldCoordinateBounds.FIELD.isInBounds(game.getFieldModel().getPlayerCoordinate(player))
        let player_id = game.acting_player.player_id.clone();
        let is_on_pitch = player_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id))
            .is_some();

        let suffering_blood_lust = game.acting_player.suffering_blood_lust;
        if suffering_blood_lust {
            if let Some(action) = self.bloodlust_action {
                // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), bloodlustAction, false)
                if let Some(ref pid) = player_id {
                    util_server_steps::change_player_action(game, pid, action, false);
                }
                // Java: push Move sequence (bb2020)
                let seq = Move::build_sequence(&MoveParams::default());
                // Java: actingPlayer.setHasFouled(false)
                game.acting_player.has_fouled = false;
                return StepOutcome::next().push_seq(seq);
            }
        }

        // Java: else if (!fEndTurn && isOnPitch
        //           && player.hasSkillProperty(NamedProperties.canMoveAfterFoul)
        //           && UtilPlayer.isNextMovePossible(game, false))
        let can_move_after_foul = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_FOUL))
            .unwrap_or(false);

        if !self.end_turn && is_on_pitch && can_move_after_foul && UtilPlayer::is_next_move_possible(game, false) {
            // Java: Select.pushSequence(new Select.SequenceParams(getGameState(), true))
            let seq = Select::build_sequence(&SelectParams { update_persistence: true, is_blitz_move: false, block_targets: vec![] });
            // Java: UtilServerSteps.changePlayerAction(this, player.getId(), PlayerAction.MOVE, false)
            if let Some(ref pid) = player_id {
                util_server_steps::change_player_action(game, pid, PlayerAction::Move, false);
            }
            // Java: actingPlayer.setStandingUp(false)
            game.acting_player.standing_up = false;
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else: push EndPlayerAction sequence (BB2020 — no check_forgo)
        let params = EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
        };
        StepOutcome::next().push_seq(EndPlayerAction::build_sequence(&params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, PlayerAction, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn default_start_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndFouling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_turn_true_pushes_end_player_action_with_end_turn() {
        let mut game = make_game();
        let mut step = StepEndFouling::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepEndFouling::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepEndFouling::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_bloodlust_action() {
        let mut step = StepEndFouling::new();
        assert!(step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move))));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    // BB2020 has no check_forgo field — verify set_parameter returns false for it
    #[test]
    fn set_parameter_check_forgo_not_consumed() {
        let mut step = StepEndFouling::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }

    #[test]
    fn can_move_after_foul_with_sneaky_git_pushes_select() {
        use ffb_model::enums::PlayerState;
        use ffb_model::model::player::Player;
        // PS_STANDING(0x1) | BIT_ACTIVE(0x100) = 0x101
        const ACTIVE_STANDING: PlayerState = PlayerState(0x101);
        use ffb_model::types::FieldCoordinate;

        let mut game = make_game();
        // Put a SneakyGit player on the pitch with moves remaining
        let mut p = Player::default();
        p.id = "p1".into();
        p.movement = 6;
        p.starting_skills.push(SkillWithValue::new(SkillId::QuickFoul));
        game.team_home.players.push(p);
        game.field_model.set_player_state("p1", ACTIVE_STANDING);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.current_move = 0; // hasn't moved yet → is_next_move_possible = true

        let mut step = StepEndFouling::new();
        step.end_turn = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // should push Select sequence (update_persistence=true)
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }
}
