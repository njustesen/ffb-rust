use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::bb2016::jump_mechanic::JumpMechanic;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::mechanics::minimum_roll_jump;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepJump.
///
/// BB2016 StepJump holds a single `StepState { goToLabelOnFailure }` and delegates
/// all logic to `executeStepHooks(this, state)` in Java.
///
/// In Rust, we have no hook infrastructure yet — this is implemented with the same
/// agility-roll logic as BB2025's StepJump but without BB2025-specific modifiers.
/// The hook infrastructure (JumpModifierFactory, canStillJump, checkDivingTackle,
/// ignoreModifiers) is stubbed out.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// Logic:
/// - If !actingPlayer.isJumping() → NEXT_STEP
/// - Roll 1d6 vs minimum_roll_jump(agility, [])
/// - Success → clear jumping, NEXT_STEP + JUMPED(true)
/// - Failure → TRR offer if available, else fail_jump: GOTO_LABEL + COORDINATE_FROM
///
/// BB2016 JumpModifierCollection is empty (confirmed Java source) → &[] is correct.
/// BB2016 agility_with_modifiers() == agility (no stat-injury pipeline yet).
/// no-op: DivingTackle executeStepHooks skipped in headless (SkillBehaviour registry not ported).
/// canStillJump: wired via BB2016 JumpMechanic.
/// client-only: checkDivingTackle dialog — headless auto-skips diving tackle activation.
pub struct StepJump {
    /// Java: StepState.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Internal roll (not a Java field — it's local in executeStepHooks)
    pub roll: i32,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepJump {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            roll: 0,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepJump {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepJump {
    fn id(&self) -> StepId { StepId::Jump }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        // client-only: CLIENT_PLAYER_CHOICE DIVING_TACKLE mode — headless never receives this
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepJump {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: doLeap = actingPlayer.isJumping() && mechanic.canStillJump(game, actingPlayer)
        let mechanic = JumpMechanic::new();
        let do_leap = game.acting_player.jumping
            && mechanic.can_still_jump(game, &game.acting_player.clone());

        if !do_leap {
            return StepOutcome::next();
        }

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "JUMP").unwrap_or(false);

        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                return self.handle_failure(game);
            }
        }

        if self.roll == 0 {
            self.roll = rng.d6();
        }

        let player_id = game.acting_player.player_id.clone();
        // BB2016 JumpModifierCollection is empty → no modifiers apply.
        // agility_with_modifiers() == agility in current model.
        let agility = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.agility_with_modifiers())
            .unwrap_or(3);
        let minimum_roll = minimum_roll_jump(agility, &[]);
        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        if successful {
            game.acting_player.jumping = false;
            return StepOutcome::next()
                .publish(StepParameter::Jumped(true));
        }

        // Try re-roll on first failure
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));

            // TRR offer (no skill re-roll for JUMP in BB2016)
            if let Some(prompt) = ask_for_reroll_if_available(game, "JUMP", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.handle_failure(game)
    }

    fn handle_failure(&mut self, game: &mut Game) -> StepOutcome {
        game.acting_player.jumping = false;
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropJump".into());
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, SkillId, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player_ag3(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn not_jumping_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = false;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_without_leap_skill_returns_next_step() {
        // canStillJump requires unused Leap skill — no skill → skip jump
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], // no Leap skill
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn jumping_success_clears_jumping_and_publishes_jumped() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        step.roll = 4; // ag=3, min=3, 4>=3 → success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.jumping);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Jumped(true))));
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_clears_jumping_flag() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.jumping);
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 5;
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepJump::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepJump::new("fail".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn failure_publishes_steady_footing_context_drop_jump() {
        let mut game = make_game();
        add_player_ag3(&mut game, "p1");
        game.acting_player.jumping = true;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }
}
