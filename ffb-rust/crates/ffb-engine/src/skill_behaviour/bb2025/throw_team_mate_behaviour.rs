/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.ThrowTeamMateBehaviour.
///
/// Java: registers a StepModifier<StepThrowTeamMate, StepState> with priority 2.
/// Modifier handleExecuteStepHook performs the TTM pass roll, applies pass modifiers,
/// evaluates pass result, handles a reroll prompt, and publishes PASS_RESULT.
///
/// Rust note: all hook logic is inlined into StepThrowTeamMate.execute_step.
/// The StepModifier here mirrors the Java hook shape and applies the pre-roll
/// game-state mutations (has_passed, thrower_id, concession, ttm/ktm_used).
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::{PassResult, SkillId};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::step::framework::StepOutcome;

// ── StepThrowTeamMateHookState ────────────────────────────────────────────────

/// Companion hook-state for ThrowTeamMateStepModifier.
///
/// Java: StepThrowTeamMate.StepState fields used by handleExecuteStepHook.
pub struct StepThrowTeamMateHookState {
    /// Java: state.thrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: state.kicked
    pub kicked: bool,
    /// Java: state.passResult
    pub pass_result: Option<PassResult>,
    /// Java: state.usingBullseye (tristate: None = not yet decided)
    pub using_bullseye: Option<bool>,
    /// Java: fReRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: fReRollSource
    pub re_roll_source: Option<String>,
    /// Output settled by the hook.
    pub outcome: Option<StepOutcome>,
}

impl StepThrowTeamMateHookState {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            kicked: false,
            pass_result: None,
            using_bullseye: None,
            re_rolled_action: None,
            re_roll_source: None,
            outcome: None,
        }
    }
}

impl Default for StepThrowTeamMateHookState {
    fn default() -> Self { Self::new() }
}

// ── ThrowTeamMateBehaviour ────────────────────────────────────────────────────

/// Throw Team-Mate: player may throw a Small teammate instead of the ball.
pub struct ThrowTeamMateBehaviour;

impl ThrowTeamMateBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(ThrowTeamMateStepModifier));
        registry.register(SkillId::ThrowTeamMate, sb);
    }
}

impl Default for ThrowTeamMateBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ThrowTeamMateBehaviour {
    fn name(&self) -> &'static str { "ThrowTeamMateBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::ThrowTeamMate))
            .unwrap_or(false);
        if !has_skill { return false; }
        // headless: TTM pass sequence inlined in StepThrowTeamMate.execute_step
        false
    }
}

// ── ThrowTeamMateStepModifier ─────────────────────────────────────────────────

/// Java: anonymous StepModifier<StepThrowTeamMate, StepState> with priority 2.
pub struct ThrowTeamMateStepModifier;

impl StepModifierTrait for ThrowTeamMateStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::ThrowTeamMate }
    fn priority(&self) -> i32 { 2 }

    /// Java: handleExecuteStepHook(StepThrowTeamMate step, StepState state)
    ///
    /// Ported mutations (pre-roll game-state setup):
    ///   actingPlayer.setHasPassed(true) | game.setThrowerId | game.setConcessionPossible(false)
    ///   turnData.setTtmUsed / setKtmUsed
    ///
    /// headless: UtilServerDialog.hideDialog -- no-op
    /// headless: PassModifierFactory, PassMechanic, TtmMechanic, dice roll,
    ///   ReportThrowTeamMateRoll, Bullseye dialog, reroll prompt -- inlined in step.
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepThrowTeamMateHookState>()
            .expect("ThrowTeamMateStepModifier: step_state must be StepThrowTeamMateHookState");

        // Java: actingPlayer.setHasPassed(true)
        game.acting_player.has_passed = true;
        // Java: game.setThrowerId(actingPlayer.getPlayerId())
        game.thrower_id = game.acting_player.player_id.clone();
        // Java: game.setConcessionPossible(false)
        game.concession_possible = false;

        // Java: if (state.kicked) { turnData.setKtmUsed(true); } else { turnData.setTtmUsed(true); }
        let turn_data = if game.home_playing {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };
        if state.kicked {
            turn_data.ktm_used = true;
        } else {
            turn_data.ttm_used = true;
        }

        // headless: UtilServerDialog.hideDialog -- no-op
        // headless: pass roll + modifiers + report + bullseye + reroll -- inlined in step

        state.outcome = Some(StepOutcome::next());
        false
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::{StepId, test_team};

    fn test_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!ThrowTeamMateBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false_no_player() {
        let b = ThrowTeamMateBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn execute_step_hook_returns_false_player_lacks_skill() {
        use ffb_model::model::player::Player;
        let b = ThrowTeamMateBehaviour::new();
        let mut game = test_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            ..Default::default()
        });
        game.acting_player.player_id = Some("p1".into());
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ThrowTeamMateBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn register_into_adds_entry_for_throw_team_mate() {
        let mut reg = SkillRegistry::empty();
        ThrowTeamMateBehaviour::register_into(&mut reg);
        assert!(reg.get(SkillId::ThrowTeamMate).is_some());
    }

    #[test]
    fn register_into_registers_one_step_modifier() {
        let mut reg = SkillRegistry::empty();
        ThrowTeamMateBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::ThrowTeamMate).unwrap();
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn modifier_applies_to_throw_team_mate_step() {
        assert!(ThrowTeamMateStepModifier.applies_to(StepId::ThrowTeamMate));
    }

    #[test]
    fn modifier_does_not_apply_to_block_roll() {
        assert!(!ThrowTeamMateStepModifier.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn modifier_does_not_apply_to_bone_head() {
        assert!(!ThrowTeamMateStepModifier.applies_to(StepId::BoneHead));
    }

    #[test]
    fn modifier_priority_is_2() {
        assert_eq!(ThrowTeamMateStepModifier.priority(), 2);
    }

    #[test]
    fn handle_execute_step_sets_has_passed_true() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(game.acting_player.has_passed);
    }

    #[test]
    fn handle_execute_step_sets_thrower_id() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("thrower1".into());
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert_eq!(game.thrower_id.as_deref(), Some("thrower1"));
    }

    #[test]
    fn handle_execute_step_clears_concession_possible() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        game.concession_possible = true;
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!game.concession_possible);
    }

    #[test]
    fn handle_execute_step_marks_ttm_used_for_non_kicked() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(game.turn_data_home.ttm_used);
        assert!(!game.turn_data_home.ktm_used);
    }

    #[test]
    fn handle_execute_step_marks_ktm_used_for_kicked() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        let mut hook = StepThrowTeamMateHookState::new();
        hook.kicked = true;
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(game.turn_data_home.ktm_used);
        assert!(!game.turn_data_home.ttm_used);
    }

    #[test]
    fn handle_execute_step_returns_false() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        let mut hook = StepThrowTeamMateHookState::new();
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook));
    }

    #[test]
    fn handle_execute_step_sets_outcome() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = true;
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(hook.outcome.is_some());
    }

    #[test]
    fn hook_state_default_fields() {
        let h = StepThrowTeamMateHookState::default();
        assert!(!h.kicked);
        assert!(h.pass_result.is_none());
        assert!(h.using_bullseye.is_none());
        assert!(h.re_rolled_action.is_none());
        assert!(h.re_roll_source.is_none());
        assert!(h.thrown_player_id.is_none());
        assert!(h.outcome.is_none());
    }

    #[test]
    fn handle_execute_step_uses_away_turn_data_when_away_playing() {
        let m = ThrowTeamMateStepModifier;
        let mut game = test_game();
        game.home_playing = false;
        let mut hook = StepThrowTeamMateHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(game.turn_data_away.ttm_used);
        assert!(!game.turn_data_home.ttm_used);
    }
}
