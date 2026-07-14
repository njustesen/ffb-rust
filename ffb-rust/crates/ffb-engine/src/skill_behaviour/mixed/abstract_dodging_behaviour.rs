use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::framework::StepId;
use crate::step::mixed::step_block_dodge::StepBlockDodgeHookState;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;

/// Abstract base for dodge-modifying skill behaviours across editions.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.AbstractDodgingBehaviour`.
/// Used by BB2020/BB2025 `DodgeBehaviour` (priority 1, `requireUnusedSkill=false`) and
/// `WatchOutBehaviour` (priority 2, `requireUnusedSkill=true`) — both registered against
/// `StepId::BlockDodge` via [`AbstractDodgingStepModifier`].
pub struct AbstractDodgingBehaviour;

impl AbstractDodgingBehaviour {
    pub fn new() -> Self { Self }

    /// Register a dodge-family skill (Dodge or WatchOut) into the registry.
    /// Java: `AbstractDodgingBehaviour(int priority, boolean requireUnusedSkill)`.
    pub fn register_into(registry: &mut SkillRegistry, skill_id: SkillId, priority: i32, require_unused_skill: bool) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(AbstractDodgingStepModifier { skill_id, priority, require_unused_skill }));
        registry.register(skill_id, sb);
    }
}

impl Default for AbstractDodgingBehaviour {
    fn default() -> Self { Self::new() }
}

// ── AbstractDodgingStepModifier ────────────────────────────────────────────────

/// Java: anonymous `StepModifier<StepBlockDodge, StepState>` registered in
/// `AbstractDodgingBehaviour`'s constructor.
pub struct AbstractDodgingStepModifier {
    pub skill_id: SkillId,
    pub priority: i32,
    pub require_unused_skill: bool,
}

impl StepModifierTrait for AbstractDodgingStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::BlockDodge }

    fn priority(&self) -> i32 { self.priority }

    /// Java: `AbstractDodgingBehaviour.handleExecuteStepHook(StepBlockDodge step, StepState state)`
    ///
    /// 1. Skip if the defender doesn't have this skill, or (`requireUnusedSkill`) has already
    ///    used it.
    /// 2. If `usingDodge` is null, default it to `oldDefenderState.hasTacklezones()`.
    /// 3. Java shows a skill-use dialog when `askForSkill && hasTacklezones` and returns `true`
    ///    (wait for a command). This codebase has no live dialog channel through the step-hook
    ///    dispatch path (see `StandFirmStepModifier`'s and BB2016 `StepBlockDodge`'s identical
    ///    precedent) — headless mode resolves immediately using the already-computed default
    ///    instead of waiting, so this never returns `true`.
    /// 4. Add a `ReportSkillUse` report entry (AVOID_FALLING or NO_TACKLEZONE).
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepBlockDodgeHookState>()
            .expect("AbstractDodgingStepModifier: step_state must be StepBlockDodgeHookState");

        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return false,
        };

        let has_skill = game.player(&defender_id).map(|p| p.has_skill(self.skill_id)).unwrap_or(false);
        if !has_skill {
            return false;
        }
        if self.require_unused_skill {
            let used = game.player(&defender_id)
                .map(|p| p.used_skills.contains(&self.skill_id))
                .unwrap_or(false);
            if used {
                return false;
            }
        }

        let has_tacklezones = state.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(false);

        if state.using_dodge.is_none() {
            state.using_dodge = Some(has_tacklezones);
        }

        // Java: if (state.askForSkill && hasTacklezones) { showDialog(...); return true; }
        // Headless: no dialog channel here — resolve with the already-computed default instead
        // (see doc comment above).

        game.report_list.add(ReportSkillUse::new(
            Some(defender_id),
            self.skill_id,
            state.using_dodge.unwrap_or(false),
            if has_tacklezones { SkillUse::AVOID_FALLING } else { SkillUse::NO_TACKLEZONE },
        ));

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn player_with_skills(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    fn hook_state(old_defender_state: Option<PlayerState>) -> StepBlockDodgeHookState {
        StepBlockDodgeHookState { using_dodge: None, ask_for_skill: false, old_defender_state }
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        AbstractDodgingBehaviour::register_into(&mut reg, SkillId::Dodge, 1, false);
        let sb = reg.get(SkillId::Dodge).expect("Dodge must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_block_dodge_step() {
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        assert!(m.applies_to(StepId::BlockDodge));
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn priority_matches_configured_value() {
        let m = AbstractDodgingStepModifier { skill_id: SkillId::WatchOut, priority: 2, require_unused_skill: true };
        assert_eq!(m.priority(), 2);
    }

    #[test]
    fn no_skill_returns_false_and_leaves_state_untouched() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
        assert!(hs.using_dodge.is_none());
    }

    #[test]
    fn require_unused_skill_gate_blocks_already_used() {
        let mut game = make_game();
        let mut p = player_with_skills("def", vec![SkillId::WatchOut]);
        p.used_skills.insert(SkillId::WatchOut);
        game.team_away.players.push(p);
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::WatchOut, priority: 2, require_unused_skill: true };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
        assert!(hs.using_dodge.is_none(), "gated out before defaulting usingDodge");
    }

    #[test]
    fn unused_skill_allowed_through_gate() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::WatchOut]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::WatchOut, priority: 2, require_unused_skill: true };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.using_dodge, Some(true));
    }

    #[test]
    fn defaults_using_dodge_true_when_has_tacklezones() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::Dodge]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.using_dodge, Some(true));
    }

    #[test]
    fn defaults_using_dodge_false_when_no_tacklezones() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::Dodge]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_PRONE)));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.using_dodge, Some(false));
    }

    #[test]
    fn does_not_override_already_decided_using_dodge() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::Dodge]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_PRONE)));
        hs.using_dodge = Some(true); // already answered via a prior command
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.using_dodge, Some(true));
    }

    #[test]
    fn adds_skill_use_report() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::Dodge]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn always_returns_false_never_waits() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def", vec![SkillId::Dodge]));
        game.defender_id = Some("def".into());
        let m = AbstractDodgingStepModifier { skill_id: SkillId::Dodge, priority: 1, require_unused_skill: false };
        let mut hs = hook_state(Some(PlayerState::new(PS_STANDING)));
        hs.ask_for_skill = true;
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
    }
}
