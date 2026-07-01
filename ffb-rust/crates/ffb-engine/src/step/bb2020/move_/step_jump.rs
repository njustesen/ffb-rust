use ffb_model::types::FieldCoordinate;
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
use ffb_mechanics::bb2020::jump_mechanic::JumpMechanic;
use ffb_mechanics::jump_mechanic::JumpMechanic as JumpMechanicTrait;
use ffb_mechanics::mechanics::minimum_roll_jump;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepJump.
///
/// BB2020 uses the BB2020 JumpMechanic for canStillJump checks.
/// Otherwise logic is identical to BB2025.
///
/// DEFERRED: DivingTackle dialog not yet ported.
/// DEFERRED(divingTackle): checkDivingTackle/usingDivingTackle dialog not yet ported.
pub struct StepJump {
    /// Java: goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: moveStart
    pub move_start: Option<FieldCoordinate>,
    /// Java: roll
    pub roll: i32,
    /// Java: usingDivingTackle (Boolean tristate)
    pub using_diving_tackle: Option<bool>,
    /// Java: alreadyReported
    pub already_reported: bool,
    /// Java: useIgnoreModifierAfterRollSkill (Boolean tristate)
    pub use_ignore_modifier_after_roll_skill: Option<bool>,
    /// Java: useIgnoreModifierSkill
    pub use_ignore_modifier_skill: bool,
    /// Java: dtRerollAsked
    pub dt_reroll_asked: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepJump {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            move_start: None,
            roll: 0,
            using_diving_tackle: None,
            already_reported: false,
            use_ignore_modifier_after_roll_skill: None,
            use_ignore_modifier_skill: false,
            dt_reroll_asked: false,
            re_roll_state: ReRollState::new(),
        }
    }
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
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::MoveStart(v) => { self.move_start = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            _ => false,
        }
    }
}

impl StepJump {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
        let agility = player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.agility_with_modifiers())
            .unwrap_or(3);

        let minimum_roll = minimum_roll_jump(agility, &[]);
        let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        if successful {
            game.acting_player.jumping = false;
            StepOutcome::next()
                .publish(StepParameter::Jumped(true))
        } else {
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("JUMP"));

                if let Some(prompt) = ask_for_reroll_if_available(game, "JUMP", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }

            self.handle_failure(game)
        }
    }

    fn handle_failure(&mut self, game: &mut Game) -> StepOutcome {
        game.acting_player.jumping = false;
        let label = self.goto_label_on_failure.clone();
        let coord_from = if self.roll > 1 {
            self.move_start
        } else {
            if let (Some(pid), Some(start)) = (
                game.acting_player.player_id.clone(),
                self.move_start,
            ) {
                let old_pos = game.field_model.player_coordinate(&pid);
                if !game.field_model.ball_moving {
                    if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                        if old == ball {
                            game.field_model.ball_coordinate = Some(start);
                        }
                    }
                }
                game.field_model.set_player_coordinate(&pid, start);
            }
            None
        };
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropJump".into());
        let mut out = StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        if let Some(c) = coord_from {
            out = out.publish(StepParameter::CoordinateFrom(c));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn make_game_with_leaper() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        let mut player = Player::default();
        player.id = "p1".into();
        player.agility = 3;
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.jumping = true;
        game
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
    fn jumping_but_cannot_still_jump_returns_next_step() {
        let mut game = make_game();
        game.acting_player.jumping = true;
        let mut step = StepJump::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_clears_jumping_flag() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.jumping);
    }

    #[test]
    fn success_publishes_jumped_true() {
        let mut game = make_game_with_leaper();
        let mut step = StepJump::new("fail".into());
        step.roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Jumped(true))));
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game_with_leaper();
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
    fn set_parameter_move_start_accepted() {
        let mut step = StepJump::new("fail".into());
        let coord = FieldCoordinate::new(4, 4);
        assert!(step.set_parameter(&StepParameter::MoveStart(coord)));
        assert_eq!(step.move_start, Some(coord));
    }

    #[test]
    fn failure_publishes_steady_footing_context_drop_jump() {
        let mut game = make_game_with_leaper();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        let mut step = StepJump::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }
}
