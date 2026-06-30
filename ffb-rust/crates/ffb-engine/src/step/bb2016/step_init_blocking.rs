/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepInitBlocking`.
///
/// Initialises a block sequence:
/// - EndTurn / EndPlayerAction → publish flag + goto end label.
/// - `Action::Block { defender_id }` → record defender, then execute.
/// - Otherwise: set defender, publish OldDefenderState / DefenderPosition / UsingStab,
///   mark defender as BLOCKED, change BLITZ_MOVE → BLITZ if needed.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), BLOCK_DEFENDER_ID (optional),
///   USING_STAB (optional), MULTI_BLOCK_DEFENDER_ID (optional).
///
/// Note: `actingPlayer.isSufferingBloodLust()` and `actingPlayer.setStrength()` are not
/// yet ported to the Rust model — blood lust check is stubbed as `false`.
use ffb_model::enums::{PlayerAction, PS_BLOCKED};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitBlocking` (bb2016).
pub struct StepInitBlocking {
    /// Java: `fGotoLabelOnEnd` — init param (mandatory).
    goto_label_on_end: String,
    /// Java: `fBlockDefenderId`
    block_defender_id: Option<String>,
    /// Java: `fUsingStab`
    using_stab: bool,
    /// Java: `fMultiBlockDefenderId` — optional init param.
    multi_block_defender_id: Option<String>,
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
}

impl StepInitBlocking {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            block_defender_id: None,
            using_stab: false,
            multi_block_defender_id: None,
            end_turn: false,
            end_player_action: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if self.end_turn {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }

        // Java: actingPlayer.isSufferingBloodLust() && (action == MOVE || BLITZ_MOVE)
        // TODO: isSufferingBloodLust() not yet in Rust model; stub as false.
        let is_blood_lust = false;
        let action_is_move = matches!(
            game.acting_player.player_action,
            Some(PlayerAction::Move) | Some(PlayerAction::BlitzMove)
        );
        if is_blood_lust && action_is_move {
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        let defender_id = match &self.block_defender_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        if game.player(&defender_id).is_none() {
            return StepOutcome::next();
        }

        game.defender_id = Some(defender_id.clone());
        // Java: actingPlayer.setStrength(actingPlayer.getPlayer().getStrengthWithModifiers())
        // TODO: no strength field on ActingPlayer in Rust yet.

        let old_state = game.field_model.player_state(&defender_id).unwrap_or_default();
        let defender_pos = game.field_model.player_coordinate(&defender_id);

        game.field_model.set_player_state(&defender_id, old_state.change_base(PS_BLOCKED));

        // Java: if BLITZ_MOVE → changePlayerAction to BLITZ
        if game.acting_player.player_action == Some(PlayerAction::BlitzMove) {
            game.acting_player.player_action = Some(PlayerAction::Blitz);
        }

        let mut outcome = StepOutcome::next()
            .publish(StepParameter::OldDefenderState(old_state))
            .publish(StepParameter::UsingStab(self.using_stab));

        if let Some(pos) = defender_pos {
            outcome = outcome.publish(StepParameter::DefenderPosition(pos));
        }

        outcome
    }
}

impl Default for StepInitBlocking {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitBlocking {
    fn id(&self) -> StepId { StepId::InitBlocking }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            Action::Block { defender_id } => {
                // Java: CLIENT_BLOCK; skip if same as multi_block_defender_id
                let is_multi = self.multi_block_defender_id.as_deref() == Some(defender_id.as_str());
                if !is_multi {
                    self.block_defender_id = Some(defender_id.clone());
                    self.using_stab = false;
                    self.execute_step(game)
                } else {
                    StepOutcome::next()
                }
            }
            Action::Stab { defender_id } => {
                let is_multi = self.multi_block_defender_id.as_deref() == Some(defender_id.as_str());
                if !is_multi {
                    self.block_defender_id = Some(defender_id.clone());
                    self.using_stab = true;
                    self.execute_step(game)
                } else {
                    StepOutcome::next()
                }
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)         => { self.goto_label_on_end = s.clone(); true }
            StepParameter::BlockDefenderId(s)        => { self.block_defender_id = Some(s.clone()); true }
            StepParameter::UsingStab(v)              => { self.using_stab = *v; true }
            StepParameter::MultiBlockDefenderId(id)  => { self.multi_block_defender_id = id.clone(); true }
            StepParameter::EndTurn(v)                => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)        => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_init_blocking() {
        assert_eq!(StepInitBlocking::new().id(), StepId::InitBlocking);
    }

    #[test]
    fn end_turn_publishes_and_gotos_label() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::EndTurn(true));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn valid_defender_sets_state_to_blocked() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::BlockDefenderId("def".into()));
        let mut game = make_game();
        add_player(&mut game, "def");
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_BLOCKED);
        assert_eq!(game.defender_id.as_deref(), Some("def"));
    }

    #[test]
    fn publishes_old_defender_state() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::BlockDefenderId("def".into()));
        let mut game = make_game();
        add_player(&mut game, "def");
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_old_state = outcome.published.iter().any(|p| matches!(p, StepParameter::OldDefenderState(_)));
        assert!(has_old_state);
    }

    #[test]
    fn blitz_move_becomes_blitz() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::BlockDefenderId("def".into()));
        let mut game = make_game();
        add_player(&mut game, "def");
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
    }

    #[test]
    fn block_action_sets_defender_and_executes() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        add_player(&mut game, "def");
        let mut rng = GameRng::new(0);
        let outcome = step.handle_command(&Action::Block { defender_id: "def".into() }, &mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.defender_id.as_deref(), Some("def"));
    }

    #[test]
    fn no_defender_returns_next() {
        let mut step = StepInitBlocking::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }
}
