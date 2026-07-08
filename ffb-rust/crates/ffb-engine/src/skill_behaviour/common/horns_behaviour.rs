/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.common.HornsBehaviour.
///
/// Horns: attacker gets +1 ST when making a Blitz action. The actual ST bonus is applied
/// by ServerUtilBlock#getAttackerStrength; this modifier just marks the skill used and
/// emits a ReportSkillUse.
///
/// Java registration pattern:
///   `registerModifier(new StepModifier<StepHorns, StepState>() { ... })`
/// Rust equivalent:
///   `HornsBehaviour::register_into(sb)` adds a `HornsStepModifier` to the container.
use crate::skill_behaviour::SkillBehaviour as SkillBehaviourTrait;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::action::block::step_horns::StepHornsHookState;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;

// ── HornsStepModifier ─────────────────────────────────────────────────────────

/// Java: anonymous StepModifier<StepHorns, StepHorns.StepState> registered in
///       HornsBehaviour constructor.
pub struct HornsStepModifier;

impl StepModifierTrait for HornsStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Horns }

    /// Java: `handleExecuteStepHook(StepHorns step, StepState state)`
    ///
    /// 1. Checks acting player has Horns AND is performing a Blitz.
    /// 2. If yes: marks skill used, adds ReportSkillUse(true, INCREASE_STRENGTH_BY_1).
    /// 3. Sets `state.using_horns` accordingly.
    /// 4. Always returns false (does NOT stop further hook processing).
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepHornsHookState>()
            .expect("HornsStepModifier: step_state must be StepHornsHookState");

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => {
                state.using_horns = Some(false);
                return false;
            }
        };

        let has_horns = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::Horns))
            .unwrap_or(false);
        let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);

        // Java: state.usingHorns = UtilCards.hasSkill(actingPlayer, Horns) && BLITZ == action
        state.using_horns = Some(has_horns && is_blitz);

        if state.using_horns == Some(true) {
            // Java: actingPlayer.markSkillUsed(skill)
            let is_home = game.team_home.player(&player_id).is_some();
            if is_home {
                if let Some(p) = game.team_home.player_mut(&player_id) {
                    p.used_skills.insert(SkillId::Horns);
                }
            } else if let Some(p) = game.team_away.player_mut(&player_id) {
                p.used_skills.insert(SkillId::Horns);
            }

            // Java: step.getResult().addReport(new ReportSkillUse(id, skill, true, INCREASE_STRENGTH_BY_1))
            game.report_list.add(ReportSkillUse::new(
                Some(player_id),
                SkillId::Horns,
                true,
                SkillUse::INCREASE_STRENGTH_BY_1,
            ));
        }

        // Java: step.getResult().setNextAction(NEXT_STEP) → handled by StepHorns after hook returns
        false // don't stop further hook processing
    }
}

// ── HornsBehaviour (marker + registration) ────────────────────────────────────

/// Java: com.fumbbl.ffb.server.skillbehaviour.common.HornsBehaviour
pub struct HornsBehaviour;

impl HornsBehaviour {
    pub fn new() -> Self { Self }

    /// Register HornsStepModifier into the given SkillBehaviourContainer, then insert
    /// it into the SkillRegistry under SkillId::Horns.
    ///
    /// Java: HornsBehaviour constructor calls `registerModifier(new StepModifier<>() {...})`.
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(HornsStepModifier));
        registry.register(SkillId::Horns, sb);
    }
}

impl Default for HornsBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviourTrait for HornsBehaviour {
    fn name(&self) -> &'static str { "HornsBehaviour" }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerAction, PlayerState, Rules, PS_STANDING};
    use ffb_model::model::game::Game;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game_with_horns(action: PlayerAction) -> (Game, String) {
        let pid = "att".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: pid.clone(),
            name: pid.clone(),
            nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Horns, value: None }],
            ..Default::default()
        });
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(action);
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        (game, pid)
    }

    #[test]
    fn register_into_adds_one_step_modifier() {
        let mut reg = SkillRegistry::empty();
        HornsBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Horns).expect("Horns must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_horns_step() {
        let m = HornsStepModifier;
        assert!(m.applies_to(StepId::Horns));
    }

    #[test]
    fn step_modifier_does_not_apply_to_other_steps() {
        let m = HornsStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
        assert!(!m.applies_to(StepId::GoForIt));
        assert!(!m.applies_to(StepId::Wrestle));
    }

    #[test]
    fn modifier_sets_using_horns_true_when_blitzing() {
        let (mut game, _) = make_game_with_horns(PlayerAction::Blitz);
        let mut hook_state = StepHornsHookState::default();
        let m = HornsStepModifier;
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state);
        assert_eq!(hook_state.using_horns, Some(true));
    }

    #[test]
    fn modifier_sets_using_horns_false_when_blocking() {
        let (mut game, _) = make_game_with_horns(PlayerAction::Block);
        let mut hook_state = StepHornsHookState::default();
        HornsStepModifier.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state);
        assert_eq!(hook_state.using_horns, Some(false));
    }

    #[test]
    fn modifier_marks_skill_used_when_blitzing() {
        let (mut game, pid) = make_game_with_horns(PlayerAction::Blitz);
        let mut hook_state = StepHornsHookState::default();
        HornsStepModifier.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state);
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::Horns));
    }

    #[test]
    fn modifier_adds_report_when_blitzing() {
        let (mut game, _) = make_game_with_horns(PlayerAction::Blitz);
        let mut hook_state = StepHornsHookState::default();
        HornsStepModifier.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn modifier_no_report_when_not_blitzing() {
        let (mut game, _) = make_game_with_horns(PlayerAction::Block);
        let mut hook_state = StepHornsHookState::default();
        HornsStepModifier.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn modifier_returns_false_always() {
        let (mut game, _) = make_game_with_horns(PlayerAction::Blitz);
        let mut hook_state = StepHornsHookState::default();
        assert!(!HornsStepModifier.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook_state));
    }

    #[test]
    fn name_returns_correct_string() {
        assert_eq!(HornsBehaviour::new().name(), "HornsBehaviour");
    }

    #[test]
    fn default_same_as_new() {
        let _a = HornsBehaviour::new();
        let _b = HornsBehaviour::default();
    }
}
