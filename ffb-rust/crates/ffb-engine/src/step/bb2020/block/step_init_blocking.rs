use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PlayerAction, PlayerState, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.block.StepInitBlocking.
/// Initializes the block sequence: sets defender, marks skills used, publishes OldDefenderState.
pub struct StepInitBlocking {
    pub goto_label_on_end: String,
    pub block_defender_id: Option<String>,
    pub using_stab: bool,
    pub using_chainsaw: bool,
    pub using_vomit: bool,
    pub using_breathe_fire: bool,
    pub multi_block_defender_id: Option<String>,
    pub end_turn: bool,
    pub end_player_action: bool,
    pub ask_for_block_kind: bool,
    pub publish_defender: bool,
}

impl StepInitBlocking {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            block_defender_id: None,
            using_stab: false,
            using_chainsaw: false,
            using_vomit: false,
            using_breathe_fire: false,
            multi_block_defender_id: None,
            end_turn: false,
            end_player_action: false,
            ask_for_block_kind: false,
            publish_defender: false,
        }
    }
}

impl Step for StepInitBlocking {
    fn id(&self) -> StepId { StepId::InitBlocking }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            Action::Block { defender_id } => {
                // If multiBlockDefenderId is set and matches this defender, skip (Java: don't override)
                let is_multi_block_same = self.multi_block_defender_id.as_deref() == Some(defender_id.as_str());
                if !is_multi_block_same {
                    self.block_defender_id = Some(defender_id.clone());
                    self.using_stab = false;
                    self.using_chainsaw = false;
                    self.using_vomit = false;
                    self.using_breathe_fire = false;
                }
            }
            Action::Stab { defender_id } => {
                self.block_defender_id = Some(defender_id.clone());
                self.using_stab = true;
                self.using_chainsaw = false;
                self.using_vomit = false;
                self.using_breathe_fire = false;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::BlockDefenderId(v) => { self.block_defender_id = Some(v.clone()); true }
            StepParameter::UsingStab(v) => { self.using_stab = *v; true }
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; true }
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepInitBlocking {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        // If coming back from SELECT_BLOCK_KIND, restore the last turn mode
        if game.turn_mode == TurnMode::SelectBlockKind {
            game.turn_mode = game.last_turn_mode.unwrap_or(TurnMode::Regular);
        }

        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        let action_is_move = matches!(
            game.acting_player.player_action,
            Some(PlayerAction::Move) | Some(PlayerAction::BlitzMove)
        );
        if game.acting_player.suffering_blood_lust && action_is_move {
            return StepOutcome::goto(&label);
        }

        let defender_id = match self.block_defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::cont(),
        };

        // Ask for block kind (Trickster etc.) before proceeding
        if self.ask_for_block_kind {
            game.turn_mode = TurnMode::SelectBlockKind;
            self.ask_for_block_kind = false;
            return StepOutcome::cont();
        }

        // Java: actingPlayer.markSkillUsed(NamedProperties.canUseChainsawOnDownedOpponents/canUseVomitAfterBlock)
        if let Some(player_id) = game.acting_player.player_id.as_deref() {
            let player_id = player_id.to_owned();
            let chainsaw_sid = game.player(&player_id)
                .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_USE_CHAINSAW_ON_DOWNED_OPPONENTS));
            let vomit_sid = game.player(&player_id)
                .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_USE_VOMIT_AFTER_BLOCK));
            let is_home = game.team_home.player(&player_id).is_some();
            let player_mut = if is_home { game.team_home.player_mut(&player_id) }
                             else { game.team_away.player_mut(&player_id) };
            if let Some(p) = player_mut {
                if let Some(sid) = chainsaw_sid { p.used_skills.insert(sid); }
                if let Some(sid) = vomit_sid { p.used_skills.insert(sid); }
            }
        }

        game.acting_player.defender_id = Some(defender_id.clone());

        // Java: actingPlayer.setStrength(actingPlayer.getPlayer().getStrengthWithModifiers())
        if let Some(player) = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
        {
            game.acting_player.strength = player.strength_with_modifiers();
        }

        let old_state = game.field_model.player_state(&defender_id)
            .unwrap_or(PlayerState::new(ffb_model::enums::PS_STANDING));

        // Set defender state to BLOCKED
        game.field_model.set_player_state(&defender_id, old_state.change_base(ffb_model::enums::PS_BLOCKED));

        // If BlitzMove, transition to Blitz action
        if game.acting_player.player_action == Some(PlayerAction::BlitzMove) {
            game.acting_player.player_action = Some(PlayerAction::Blitz);
        }

        let mut outcome = StepOutcome::next()
            .publish(StepParameter::OldDefenderState(old_state))
            .publish(StepParameter::DefenderPosition(
                game.field_model.player_coordinate(&defender_id)
                    .unwrap_or(FieldCoordinate::new(0, 0))
            ))
            .publish(StepParameter::UsingStab(self.using_stab))
            .publish(StepParameter::UsingChainsaw(self.using_chainsaw))
            .publish(StepParameter::UsingVomit(self.using_vomit))
            .publish(StepParameter::UsingBreatheFire(self.using_breathe_fire));

        if self.publish_defender {
            outcome = outcome.publish(StepParameter::BlockDefenderId(defender_id));
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING, PS_BLOCKED};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_defender_id_stays_cont() {
        let mut step = StepInitBlocking::new("end".into());
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No block_defender_id → CONTINUE waiting for block command
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_publishes_end_turn_and_gotos_label() {
        let mut step = StepInitBlocking::new("end_label".into());
        step.end_turn = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_gotos_label() {
        let mut step = StepInitBlocking::new("end_label".into());
        step.end_player_action = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn set_parameter_block_defender_id_accepted() {
        let mut step = StepInitBlocking::new("end".into());
        step.set_parameter(&StepParameter::BlockDefenderId("p1".into()));
        assert_eq!(step.block_defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_using_flags_accepted() {
        let mut step = StepInitBlocking::new("end".into());
        step.set_parameter(&StepParameter::UsingStab(true));
        step.set_parameter(&StepParameter::UsingChainsaw(true));
        step.set_parameter(&StepParameter::UsingVomit(true));
        step.set_parameter(&StepParameter::UsingBreatheFire(true));
        assert!(step.using_stab);
        assert!(step.using_chainsaw);
        assert!(step.using_vomit);
        assert!(step.using_breathe_fire);
    }

    #[test]
    fn block_action_with_multi_block_same_defender_ignores_override() {
        let mut step = StepInitBlocking::new("end".into());
        step.multi_block_defender_id = Some("p1".into());
        step.block_defender_id = Some("p2".into()); // previously set to different player
        let mut game = make_game();
        // Sending a Block action for the multi-block defender should be ignored
        step.handle_command(
            &Action::Block { defender_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        // Should not have changed block_defender_id to p1
        // (it stays as whatever it was before — in this case p2 is untouched when is_multi_block_same == true)
        assert_eq!(step.block_defender_id.as_deref(), Some("p2"));
    }

    #[test]
    fn stab_action_sets_using_stab() {
        let mut step = StepInitBlocking::new("end".into());
        let mut game = make_game();
        step.handle_command(
            &Action::Stab { defender_id: "p3".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(step.using_stab);
        assert_eq!(step.block_defender_id.as_deref(), Some("p3"));
    }

    #[test]
    fn full_defender_flow_publishes_old_state_and_position() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def1".into());
        let mut game = make_game();
        // Place defender on the field
        game.field_model.set_player_state("def1", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("def1", ffb_model::types::FieldCoordinate::new(5, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // OldDefenderState should be published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::OldDefenderState(_))));
        // DefenderPosition should be published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DefenderPosition(_))));
        // UsingStab should be published (false)
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingStab(false))));
    }

    #[test]
    fn defender_state_set_to_blocked_after_init() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def2".into());
        let mut game = make_game();
        game.field_model.set_player_state("def2", ffb_model::enums::PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(0));
        // Defender state should be changed to BLOCKED
        let state = game.field_model.player_state("def2").expect("state should exist");
        assert_eq!(state.base(), PS_BLOCKED);
    }

    #[test]
    fn publish_defender_true_publishes_block_defender_id() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def3".into());
        step.publish_defender = true;
        let mut game = make_game();
        game.field_model.set_player_state("def3", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(id) if id == "def3")));
    }

    #[test]
    fn publish_defender_false_does_not_publish_block_defender_id() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def4".into());
        step.publish_defender = false;
        let mut game = make_game();
        game.field_model.set_player_state("def4", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(_))));
    }

    #[test]
    fn blitz_move_transitions_to_blitz() {
        use ffb_model::enums::PlayerAction;
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def5".into());
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.field_model.set_player_state("def5", ffb_model::enums::PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
    }

    #[test]
    fn ask_for_block_kind_sets_turn_mode_and_cont() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def6".into());
        step.ask_for_block_kind = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::SelectBlockKind);
        // ask_for_block_kind should have been reset to false
        assert!(!step.ask_for_block_kind);
        // Should cont() waiting for a block-kind choice
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn select_block_kind_turn_mode_restored_before_execution() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def7".into());
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::SelectBlockKind;
        game.last_turn_mode = Some(ffb_model::enums::TurnMode::Regular);
        game.field_model.set_player_state("def7", ffb_model::enums::PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(0));
        // TurnMode should have been restored from SelectBlockKind before execution
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::Regular);
    }

    #[test]
    fn using_flags_published_correctly() {
        let mut step = StepInitBlocking::new("end".into());
        step.block_defender_id = Some("def8".into());
        step.using_stab = true;
        step.using_chainsaw = true;
        step.using_vomit = true;
        step.using_breathe_fire = true;
        let mut game = make_game();
        game.field_model.set_player_state("def8", ffb_model::enums::PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingStab(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingChainsaw(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingVomit(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingBreatheFire(true))));
    }
}
