use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::skill_behaviour::dispatch;

// ── Hook state ────────────────────────────────────────────────────────────────

/// Java: StepWildAnimal.StepState (extended with AbstractStepWithReRoll fields).
/// Used by WildAnimalBehaviour.handleExecuteStepHook via dispatch::execute_step_hooks.
#[derive(Debug)]
pub struct StepWildAnimalHookState {
    pub goto_label_on_failure: String,
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub outcome: Option<StepOutcome>,
    pub updated_re_rolled_action: Option<String>,
    pub updated_re_roll_source: Option<String>,
}

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepWildAnimal`.
///
/// Resolves the Wild Animal negatrait check.
/// Java: `executeStep()` is a thin wrapper delegating to
/// `getGameState().executeStepHooks(this, state)` — the actual logic lives in
/// `WildAnimalBehaviour.handleExecuteStepHook`
/// (see `skill_behaviour::bb2016::wild_animal_behaviour::WildAnimalStepModifier`).
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
pub struct StepWildAnimal {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepWildAnimal {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepWildAnimal {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepWildAnimal {
    fn id(&self) -> StepId { StepId::WildAnimal }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepWildAnimal {
    /// Java: `StepWildAnimal.executeStep()` → `getGameState().executeStepHooks(this, state)`.
    /// Logic lives in `WildAnimalBehaviour.handleExecuteStepHook` (via dispatch).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut hook_state = StepWildAnimalHookState {
            goto_label_on_failure: self.goto_label_on_failure.clone(),
            re_rolled_action: self.re_rolled_action.clone(),
            re_roll_source: self.re_roll_source.clone(),
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        dispatch::execute_step_hooks(game, rng, StepId::WildAnimal, &mut hook_state);
        if let Some(rra) = hook_state.updated_re_rolled_action {
            self.re_rolled_action = Some(rra);
        }
        if let Some(rrs) = hook_state.updated_re_roll_source {
            self.re_roll_source = Some(rrs);
        }
        hook_state.outcome.unwrap_or_else(StepOutcome::next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{PlayerAction, Rules, SkillId, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
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

    fn make_game(skills: Vec<SkillId>, action: Option<PlayerAction>) -> Game {
        let mut home = test_team("home", 0);
        add_player(&mut home, "beast", skills);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.acting_player.player_id = Some("beast".into());
        game.acting_player.player_action = action;
        game.turn_mode = TurnMode::Regular;
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        game.turn_mode = TurnMode::KickoffReturn;
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_wild_animal_skill_returns_next() {
        let mut game = make_game(vec![], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn good_conditions_block_min_roll_2() {
        // In good conditions (BLOCK action), min roll = 2, so roll 1 fails
        let seed = seed_for_d6(1);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        // Roll 1 < 2 → failure
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn good_conditions_roll_2_succeeds() {
        let seed = seed_for_d6(2);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bad_conditions_roll_3_fails() {
        // Bad conditions (non-BLITZ/BLOCK), min roll = 4
        let seed = seed_for_d6(3);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Move));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        // Roll 3 < 4 → failure
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn bad_conditions_roll_4_succeeds() {
        let seed = seed_for_d6(4);
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Move));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn cancel_action_sets_blitz_used_for_blitz_action() {
        let seed = seed_for_d6(1); // roll 1, min_roll=2 for BLITZ (good cond) → fail
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Blitz));
        game.turn_data_home.blitz_used = false;
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn cancel_action_sets_pass_used_for_pass_action() {
        let seed = seed_for_d6(3); // roll 3, min_roll=4 for PASS (bad cond) → fail
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Pass));
        let out = StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.turn_data().pass_used);
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let seed = seed_for_d6(1); // definitely fails good_conditions min 2
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        game.turn_data_home.rerolls = 1;
        let mut step = StepWildAnimal::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn marks_skill_used_on_roll() {
        let seed = seed_for_d6(5); // success in good or bad conditions
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert!(game.player("beast").unwrap().used_skills.contains(&SkillId::WildAnimal));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepWildAnimal::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn successful_roll_adds_confusion_roll_report() {
        // Java: WildAnimalBehaviour.handleExecuteStepHook always calls
        // step.getResult().addReport(new ReportConfusionRoll(...)) whenever a roll happens.
        // The step must dispatch through execute_step_hooks (not reimplement the logic
        // inline) so that the canonical WildAnimalStepModifier records this report.
        use ffb_model::report::report_id::ReportId;
        let seed = seed_for_d6(5); // success in good or bad conditions
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "CONFUSION_ROLL report must be added after a Wild Animal roll"
        );
    }

    #[test]
    fn failed_roll_adds_confusion_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let seed = seed_for_d6(1); // fails good_conditions min 2
        let mut game = make_game(vec![SkillId::WildAnimal], Some(PlayerAction::Block));
        StepWildAnimal::new("fail".into()).start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "CONFUSION_ROLL report must be added after a failed Wild Animal roll"
        );
    }
}
