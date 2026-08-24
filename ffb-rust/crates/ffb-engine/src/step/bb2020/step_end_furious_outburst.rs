/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepEndFuriousOutburst` (BB2020).
///
/// Cleans up after a Furious Outburst sequence. Clears the target selection state,
/// marks blitz used if the acting player has acted, then pushes the EndPlayerAction sequence.
///
/// Differs from BB2025: has only `end_turn` field (no `check_forgo`); uses BB2020 generator.
///
/// Runtime params: END_TURN.
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::EndPlayerAction;
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;

pub struct StepEndFuriousOutburst {
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
}

impl StepEndFuriousOutburst {
    pub fn new() -> Self { Self { end_turn: false } }
}

impl Default for StepEndFuriousOutburst {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFuriousOutburst {
    fn id(&self) -> StepId { StepId::EndFuriousOutburst }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepEndFuriousOutburst {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: selectedPlayerId = fieldModel.getTargetSelectionState().getSelectedPlayerId()
        // Java: if (StringTool.isProvided(selectedPlayerId)) { ... changeSelectedStabTarget(false) }
        // Java: fieldModel.setTargetSelectionState(null)
        let selected_player_id: Option<String> = game.field_model.target_selection_state
            .as_ref()
            .and_then(|ts| ts.selected_player_id.clone());
        if let Some(ref pid) = selected_player_id {
            if let Some(state) = game.field_model.player_state(pid) {
                game.field_model.set_player_state(pid, state.change_selected_stab_target(false));
            }
        }
        game.field_model.target_selection_state = None;

        // Java: if (actingPlayer.hasActed()) { markSkillUsed(canTeleportBeforeAndAfterAvRollAttack); setBlitzUsed(true) }
        // Java: `actingPlayer.hasActed()` — a COMPUTED predicate
        // (hasMoved||hasFouled||hasBlocked||hasPassed||hasTriggeredEffect||!usedSkills.isEmpty()
        // ||isForgone), not a stored flag. The Furious Outburst stab sets has_blocked, never the
        // bare `has_acted` field, so reading the field skipped `setBlitzUsed(true)` entirely —
        // Java `f0000,1100` vs Rust `f0000,0100` (wood_elf bb2025 seed 1 i=24). `acted()` is the
        // documented mirror; the bare field alone is always wrong here.
        if game.acting_player.acted() {
            if let Some(pid) = game.acting_player.player_id.as_deref() {
                let pid = pid.to_owned();
                let sid = game.player(&pid).and_then(|p| UtilCards::get_unused_skill_with_property(
                    p, NamedProperties::CAN_TELEPORT_BEFORE_AND_AFTER_AV_ROLL_ATTACK));
                if let Some(sid) = sid {
                    let is_home = game.team_home.player(&pid).is_some();
                    if is_home { game.team_home.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                    else { game.team_away.player_mut(&pid).map(|p| p.used_skills.insert(sid)); }
                }
            }
            game.turn_data_mut().blitz_used = true;
        }

        // Java: endPlayerAction.pushSequence(new EndPlayerAction.SequenceParams(gs, true, true, endTurn))
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
        });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn default_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndFuriousOutburst::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn clears_target_selection_state() {
        use ffb_model::model::TargetSelectionState;
        let mut game = make_game();
        game.field_model.target_selection_state = Some(TargetSelectionState::default());
        let mut step = StepEndFuriousOutburst::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.target_selection_state.is_none());
    }

    #[test]
    fn marks_blitz_used_when_has_acted() {
        let mut game = make_game();
        game.acting_player.has_acted = true;
        let mut step = StepEndFuriousOutburst::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used);
    }

    /// Java's `actingPlayer.hasActed()` is COMPUTED
    /// (hasMoved||hasFouled||hasBlocked||hasPassed||hasTriggeredEffect||!usedSkills.isEmpty()
    /// ||isForgone), not a stored flag. The Furious Outburst stab sets `has_blocked` and NEVER the
    /// bare `has_acted` field, so reading the field skipped `setBlitzUsed(true)` on the real path
    /// and the star could outburst every turn (wood_elf bb2025 seed 1 i=24: Java `f0000,1100` vs
    /// Rust `f0000,0100`). The sibling test above sets `has_acted` directly and so never
    /// exercised this — which is exactly why the bug survived with tests passing.
    #[test]
    fn marks_blitz_used_from_the_computed_acted_predicate_not_the_bare_flag() {
        let mut game = make_game();
        game.acting_player.has_acted = false;
        game.acting_player.has_blocked = true;
        assert!(game.acting_player.acted(), "the stab makes the COMPUTED predicate true");
        let mut step = StepEndFuriousOutburst::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used,
            "Furious Outburst counts as the team's Blitz Action");
    }

    #[test]
    fn does_not_mark_blitz_used_when_not_acted() {
        let mut game = make_game();
        game.acting_player.has_acted = false;
        assert!(!game.acting_player.acted(), "no sub-flag may be set either");
        let mut step = StepEndFuriousOutburst::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.turn_data().blitz_used);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepEndFuriousOutburst::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_check_forgo_not_accepted() {
        let mut step = StepEndFuriousOutburst::new();
        // BB2020 has no check_forgo field
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }
}
