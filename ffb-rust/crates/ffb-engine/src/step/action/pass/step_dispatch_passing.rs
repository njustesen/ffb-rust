/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepDispatchPassing (COMMON).
///
/// Routing step that dispatches to different passing sub-sequences based on the thrower action.
///
/// Mandatory init params: GOTO_LABEL_ON_END, GOTO_LABEL_ON_HAIL_MARY_PASS, GOTO_LABEL_ON_HAND_OVER.
/// Expected preceding param: CATCHER_ID.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepDispatchPassing {
    /// Java: fGotoLabelOnEnd — mandatory.
    pub goto_label_on_end: String,
    /// Java: fGotoLabelOnHailMaryPass — mandatory.
    pub goto_label_on_hail_mary_pass: String,
    /// Java: fGotoLabelOnHandOver — mandatory.
    pub goto_label_on_hand_over: String,
    /// Java: fCatcherId — set by preceding step.
    pub catcher_id: Option<String>,
}

impl StepDispatchPassing {
    pub fn new(
        goto_label_on_end: impl Into<String>,
        goto_label_on_hail_mary_pass: impl Into<String>,
        goto_label_on_hand_over: impl Into<String>,
    ) -> Self {
        Self {
            goto_label_on_end: goto_label_on_end.into(),
            goto_label_on_hail_mary_pass: goto_label_on_hail_mary_pass.into(),
            goto_label_on_hand_over: goto_label_on_hand_over.into(),
            catcher_id: None,
        }
    }
}

impl Step for StepDispatchPassing {
    fn id(&self) -> StepId { StepId::DispatchPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::GotoLabelOnHailMaryPass(v) => { self.goto_label_on_hail_mary_pass = v.clone(); true }
            StepParameter::GotoLabelOnHandOver(v) => { self.goto_label_on_hand_over = v.clone(); true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepDispatchPassing {
    fn execute_step(&self, game: &Game) -> StepOutcome {
        // Java: if (thrower == null || throwerAction == null) { return; } — a bare return with no
        // `setNextAction` call at all, i.e. the step stays waiting (StepAction.CONTINUE), NOT a
        // GOTO to the end label.
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            return StepOutcome::cont();
        }

        match game.thrower_action {
            // Java: PASS, THROW_BOMB, DUMP_OFF → NEXT_STEP
            Some(PlayerAction::Pass)
            | Some(PlayerAction::ThrowBomb)
            | Some(PlayerAction::DumpOff) => StepOutcome::next(),

            // Java: HAIL_MARY_PASS, HAIL_MARY_BOMB → GOTO hail_mary label
            Some(PlayerAction::HailMaryPass)
            | Some(PlayerAction::HailMaryBomb) => StepOutcome::goto(&self.goto_label_on_hail_mary_pass),

            // Java: HAND_OVER → GOTO hand_over label
            Some(PlayerAction::HandOver) => StepOutcome::goto(&self.goto_label_on_hand_over),

            // Java: default → GOTO end label
            _ => StepOutcome::goto(&self.goto_label_on_end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerAction, Rules};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.thrower_id = Some("t1".into());
        game
    }

    fn make_step() -> StepDispatchPassing {
        StepDispatchPassing::new("end", "hailmary", "handover")
    }

    #[test]
    fn pass_action_returns_next() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Pass);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn throw_bomb_action_returns_next() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn dump_off_action_returns_next() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::DumpOff);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn hail_mary_pass_gotos_hailmary_label() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::HailMaryPass);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("hailmary"));
    }

    #[test]
    fn hail_mary_bomb_gotos_hailmary_label() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::HailMaryBomb);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("hailmary"));
    }

    #[test]
    fn hand_over_gotos_handover_label() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::HandOver);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("handover"));
    }

    #[test]
    fn unknown_action_gotos_end_label() {
        let mut game = make_game();
        game.thrower_action = Some(PlayerAction::Move);
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn no_thrower_continues_waiting() {
        // Java: `if (thrower == null || throwerAction == null) { return; }` — a bare return with no
        // setNextAction call, so the step just stays waiting rather than jumping anywhere.
        let mut game = make_game();
        game.thrower_id = None;
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn no_thrower_action_continues_waiting() {
        let mut game = make_game();
        game.thrower_action = None;
        let out = make_step().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = make_step();
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }
}
