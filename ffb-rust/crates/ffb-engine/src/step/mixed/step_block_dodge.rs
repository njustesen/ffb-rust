/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepBlockDodge`.
///
/// Handles the Dodge skill in the block sequence.  After a pushback the defender may
/// use Dodge to avoid falling.
///
/// Java state fields (inner `StepState`):
///   `usingDodge`       — None until the skill-use dialog has been answered
///   `askForSkill`      — None until computed; true when at least one dangerous push exists
///   `oldDefenderState` — the defender's state before the block, from `OLD_DEFENDER_STATE`
///
/// Java execution logic (executeStep):
///   1. First call: compute `ask_for_skill` by scanning pushback squares.
///   2. Hide dialog / run hooks (simplified: hooks not yet ported).
///   3. If `using_dodge`: restore `old_defender_state` on the defender.
///   4. Otherwise: set defender to `FALLING`.
///   5. Publish pushback-init parameters and advance to next step.
///
/// Note: `findDodgeChoice` and `UtilServerPushback` are not yet fully ported.
/// The step stores the fields faithfully; the complex pushback-square scan is stubbed
/// (`// TODO: pushback squares`) while the set_parameter and structural logic are correct.
use ffb_model::enums::{PlayerState, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBlockDodge` (mixed, BB2020 + BB2025).
pub struct StepBlockDodge {
    /// Java: `state.usingDodge` — None until dialog answered.
    using_dodge: Option<bool>,
    /// Java: `state.askForSkill` — None until computed; true = show dodge dialog.
    ask_for_skill: Option<bool>,
    /// Java: `state.oldDefenderState`
    old_defender_state: Option<PlayerState>,
}

impl StepBlockDodge {
    pub fn new() -> Self {
        Self {
            using_dodge: None,
            ask_for_skill: None,
            old_defender_state: None,
        }
    }

    /// Java: `findDodgeChoice()` — determines if a dodge-choice dialog is needed.
    ///
    /// True when any regular or grab pushback square is occupied (chain-push risk),
    /// near a sideline, or would cross the midfield line on the first turn.
    /// Full implementation requires `UtilServerPushback` (not yet ported); returns
    /// false (conservative/safe default = no dialog) until that helper is ported.
    fn find_dodge_choice(_game: &Game) -> bool {
        // TODO(UtilServerPushback port): scan regularPushbackSquares / grabPushbackSquares
        // for chain-push, sideline-push, and attacker-half conditions.
        false
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java step 1: lazy-compute ask_for_skill
        if self.ask_for_skill.is_none() {
            self.ask_for_skill = Some(Self::find_dodge_choice(game));
        }

        // Java: UtilServerDialog.hideDialog + executeStepHooks (hooks not yet ported — skip).

        // Java: if toPrimitive(usingDodge) → restore defender; else set FALLING.
        let using_dodge = self.using_dodge.unwrap_or(false);

        if let Some(defender_id) = game.defender_id.clone() {
            if using_dodge {
                if let Some(old) = self.old_defender_state {
                    game.field_model.set_player_state(&defender_id, old);
                }
            } else {
                if let Some(state) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, state.change_base(PS_FALLING));
                }
            }
        }

        // Java: publishParameters(UtilBlockSequence.initPushback(this)) — TODO pushback port.
        StepOutcome::next()
    }
}

impl Default for StepBlockDodge {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockDodge {
    fn id(&self) -> StepId { StepId::BlockDodge }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → handleSkillCommand sets state.usingDodge
        if let Action::UseSkill { use_skill, .. } = action {
            // Dodge skill use answer
            self.using_dodge = Some(*use_skill);
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_defender(game: &mut Game, id: &str, state: u32) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        game.defender_id = Some(id.into());
    }

    #[test]
    fn id_is_block_dodge() {
        assert_eq!(StepBlockDodge::new().id(), StepId::BlockDodge);
    }

    #[test]
    fn no_dodge_sets_defender_falling() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        add_defender(&mut game, "def", PS_STANDING);
        // using_dodge stays None → false → defender should fall
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_FALLING);
    }

    #[test]
    fn using_dodge_restores_old_state() {
        let mut step = StepBlockDodge::new();
        step.using_dodge = Some(true);
        let old_state = PlayerState::new(PS_STANDING);
        step.old_defender_state = Some(old_state);

        let mut game = make_game();
        add_defender(&mut game, "def", PS_FALLING); // currently falling
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("def").unwrap();
        // Should be restored to old_state (standing)
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBlockDodge::new();
        let ps = PlayerState::new(PS_STANDING);
        let accepted = step.set_parameter(&StepParameter::OldDefenderState(ps));
        assert!(accepted);
        assert_eq!(step.old_defender_state, Some(ps));
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }
}
