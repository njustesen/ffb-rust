/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepHorns (COMMON rules).
///
/// Horns gives the attacker +1 ST during a Blitz action. The actual ST bonus is applied
/// in the block-dice calculation (ServerUtilBlock#getAttackerStrength); this step just
/// marks the skill used and emits the event so the UI can show it.
///
/// Java: StepHorns.executeStep() → GameState.executeStepHooks(this, state)
/// Rust: StepHorns.execute_step() → dispatch::execute_step_hooks(game, Horns, &mut hook_state)
///
/// All modifier logic lives in HornsBehaviour::HornsStepModifier (common/horns_behaviour.rs).
/// This file defines the hook state struct shared with that modifier.
use ffb_model::enums::SkillId;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::skill_behaviour::dispatch;

// ── Hook state ────────────────────────────────────────────────────────────────

/// Java: StepHorns.StepState — local mutable state passed through executeStepHooks.
/// Exported so HornsBehaviour's HornsStepModifier can downcast to it.
#[derive(Debug, Default)]
pub struct StepHornsHookState {
    /// Java: state.usingHorns
    pub using_horns: Option<bool>,
}

// ── Step ──────────────────────────────────────────────────────────────────────

pub struct StepHorns {
    /// Persisted after hook execution — readable by tests and callers.
    pub using_horns: bool,
}

impl StepHorns {
    pub fn new() -> Self { Self { using_horns: false } }
}

impl Default for StepHorns {
    fn default() -> Self { Self::new() }
}

impl Step for StepHorns {
    fn id(&self) -> StepId { StepId::Horns }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepHorns {
    /// Java: StepHorns.executeStep() — calls executeStepHooks, then sets NEXT_STEP.
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        let mut hook_state = StepHornsHookState::default();
        dispatch::execute_step_hooks(game, StepId::Horns, &mut hook_state);
        self.using_horns = hook_state.using_horns.unwrap_or(false);

        // Java: step.getResult().setNextAction(StepAction.NEXT_STEP) (set inside modifier)
        // If the modifier set using_horns=true it also wrote the report; emit the GameEvent here.
        if self.using_horns {
            let player_id = game.acting_player.player_id.clone().unwrap_or_default();
            let event = GameEvent::SkillUse {
                player_id,
                skill_id: SkillId::Horns as u16,
                used: true,
            };
            StepOutcome::next().with_event(event)
        } else {
            StepOutcome::next()
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{PlayerAction, PlayerState, Rules, PS_STANDING};
    use ffb_model::model::game::Game;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills
                .into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        });
    }

    fn make_game(skills: Vec<SkillId>, action: PlayerAction) -> (Game, String) {
        let pid = "att".to_string();
        let mut home = test_team("home", 0);
        add_player(&mut home, &pid, 1, skills);
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
    fn no_acting_player_returns_next() {
        let mut home = test_team("home", 0);
        add_player(&mut home, "att", 1, vec![SkillId::Horns]);
        let mut game = Game::new(home, test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = None;
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn no_horns_skill_returns_next_no_event() {
        let (mut game, _) = make_game(vec![], PlayerAction::Blitz);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn horns_with_block_action_skips() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Block);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn horns_with_blitz_emits_skill_used_event() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        let outcome = StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn horns_with_blitz_marks_skill_used() {
        let (mut game, pid) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::Horns));
    }

    #[test]
    fn horns_with_blitz_adds_skill_use_report() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE),
            "Horns during Blitz should add ReportSkillUse"
        );
    }

    #[test]
    fn horns_without_blitz_no_report_added() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Block);
        StepHorns::new().start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE),
            "Horns without Blitz should not add any report"
        );
    }

    #[test]
    fn horns_with_blitz_sets_using_horns_true() {
        let (mut game, _) = make_game(vec![SkillId::Horns], PlayerAction::Blitz);
        let mut step = StepHorns::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.using_horns);
    }
}
