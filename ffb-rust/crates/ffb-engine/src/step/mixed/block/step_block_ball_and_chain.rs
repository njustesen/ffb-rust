/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.block.StepBlockBallAndChain`.
///
/// Step in the block sequence to handle skill BALL_AND_CHAIN.
///
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_PUSHBACK.
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// Sets stepParameter CATCH_SCATTER_THROW_IN_MODE for all steps on the stack.
/// Sets stepParameter STARTING_PUSHBACK_SQUARE for all steps on the stack.
use ffb_model::enums::{PlayerState, PS_FALLING, PS_HIT_ON_GROUND, PS_PRONE, PS_STUNNED};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::action::block::init_pushback;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBlockBallAndChain` (mixed/block, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepBlockBallAndChain {
    /// Java: `fGotoLabelOnPushback` — mandatory init parameter.
    goto_label_on_pushback: String,
    /// Java: `fOldDefenderState` — published by a preceding step.
    old_defender_state: Option<PlayerState>,
    /// Java: `endTurn`
    end_turn: bool,
}

impl StepBlockBallAndChain {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let moves_randomly = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);

        // Java: if (UtilCards.hasSkillWithProperty(actingPlayer.getPlayer(), NamedProperties.movesRandomly) && endTurn)
        if moves_randomly && self.end_turn {
            let pushback_params = init_pushback(game);

            // Java: playerState.changeBase(PlayerState.FALLING)
            if let Some(state) = game.field_model.player_state(&acting_id) {
                game.field_model.set_player_state(&acting_id, state.change_base(PS_FALLING));
            }

            // Java: if (fOldDefenderState.getBase() == PRONE || fOldDefenderState.getBase() == STUNNED)
            //         setPlayerState(defender, fOldDefenderState.changeBase(HIT_ON_GROUND))
            if let Some(old) = self.old_defender_state {
                let base = old.base();
                if base == PS_PRONE || base == PS_STUNNED {
                    if let Some(defender_id) = game.defender_id.clone() {
                        game.field_model.set_player_state(&defender_id, old.change_base(PS_HIT_ON_GROUND));
                    }
                }
            }

            let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
            for p in pushback_params {
                outcome = outcome.publish(p);
            }
            return outcome;
        }

        // Java: else if (movesRandomly && fOldDefenderState != null && fOldDefenderState.isProneOrStunned())
        if moves_randomly && self.old_defender_state.map(|s| s.is_prone_or_stunned()).unwrap_or(false) {
            let pushback_params = init_pushback(game);

            // Java: setPlayerState(defender, fOldDefenderState.changeBase(HIT_ON_GROUND))
            if let Some(old) = self.old_defender_state {
                if let Some(defender_id) = game.defender_id.clone() {
                    game.field_model.set_player_state(&defender_id, old.change_base(PS_HIT_ON_GROUND));
                }
            }

            let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
            for p in pushback_params {
                outcome = outcome.publish(p);
            }
            return outcome;
        }

        // Java: else { getResult().setNextAction(StepAction.NEXT_STEP) }
        StepOutcome::next()
    }
}

impl Step for StepBlockBallAndChain {
    fn id(&self) -> StepId { StepId::BlockBallAndChain }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; false }
            StepParameter::GotoLabelOnPushback(s) => { self.goto_label_on_pushback = s.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state_bits: u32, skills: &[SkillId]) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state_bits));
    }

    #[test]
    fn id_is_block_ball_and_chain() {
        assert_eq!(StepBlockBallAndChain::new().id(), StepId::BlockBallAndChain);
    }

    #[test]
    fn without_moves_randomly_returns_next() {
        let mut step = StepBlockBallAndChain {
            goto_label_on_pushback: "pushback".into(),
            ..Default::default()
        };
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        // Acting player without Ball & Chain (Block skill has no movesRandomly property)
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::Block);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut step = StepBlockBallAndChain {
            goto_label_on_pushback: "label".into(),
            ..Default::default()
        };
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepBlockBallAndChain::new();
        step.set_parameter(&StepParameter::EndTurn(true));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBlockBallAndChain::new();
        let state = PlayerState::new(PS_PRONE);
        step.set_parameter(&StepParameter::OldDefenderState(state));
        assert_eq!(step.old_defender_state, Some(state));
    }

    #[test]
    fn set_parameter_goto_label_on_pushback() {
        let mut step = StepBlockBallAndChain::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("myLabel".into()));
        assert_eq!(step.goto_label_on_pushback, "myLabel");
    }
}
