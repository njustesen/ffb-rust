/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepIntercept`.
///
/// Step in the pass sequence to handle interceptions (BB2016).
///  1. Guard: no thrower → return.
///  2. Guard: HailMaryPass/HailMaryBomb → goto failure.
///  3. Find possible interceptors via geometric corridor check (`UtilPassing.findInterceptors`).
///  4. If none → goto failure.
///  5. Wait for CLIENT_INTERCEPTOR_CHOICE dialog (headless: shows dialog, waits).
///  6. Roll agility for the chosen interceptor.
///  7. Success → publish InterceptorId + NEXT_STEP; failure → goto failure.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Publishes: INTERCEPTOR_ID.
///
/// headless(Intercept-roll): AgilityMechanic.minimumRollInterception + modifier factory not yet ported — uses raw D6 >= 4.
/// headless(re-roll): AbstractStepWithReRoll re-roll flow not yet ported — intercept is not re-rollable here.
/// client-only(Intercept-rangeRuler): UtilRangeRuler.createRangeRuler — range ruler is client-side display.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::passing::can_intercept;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Java: `StepIntercept` (bb2016/pass).
pub struct StepIntercept {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    pub goto_label_on_failure: String,
    /// Java: `fInterceptorId` — set from CLIENT_INTERCEPTOR_CHOICE command.
    pub interceptor_id: Option<String>,
    /// Java: `fInterceptorChosen` — set when CLIENT_INTERCEPTOR_CHOICE received.
    pub interceptor_chosen: bool,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepIntercept {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            interceptor_id: None,
            interceptor_chosen: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    /// Java: `UtilPassing.findInterceptors(game, thrower, passCoordinate)`
    fn find_interceptors(game: &Game) -> Vec<String> {
        let thrower_id = match &game.thrower_id {
            Some(id) => id.clone(),
            None => return Vec::new(),
        };
        let thrower_coord = match game.field_model.player_coordinate(&thrower_id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let pass_coord = match game.pass_coordinate {
            Some(c) => c,
            None => return Vec::new(),
        };
        let opponent_team = game.inactive_team();
        opponent_team.players.iter()
            .filter(|player| {
                let coord = match game.field_model.player_coordinate(&player.id) {
                    Some(c) => c,
                    None => return false,
                };
                let state = game.field_model.player_state(&player.id);
                let has_tacklezones = state.map(|s| s.has_tacklezones()).unwrap_or(false);
                if !has_tacklezones {
                    return false;
                }
                if coord == thrower_coord || coord == pass_coord {
                    return false;
                }
                can_intercept(thrower_coord, pass_coord, coord)
            })
            .map(|p| p.id.clone())
            .collect()
    }

    /// Java: `intercept(pInterceptor)` — rolls D6 agility check.
    /// headless: AgilityMechanic.minimumRollInterception not yet ported — uses D6 >= 4.
    fn intercept(&self, _interceptor_id: &str, _game: &Game, rng: &mut GameRng) -> bool {
        rng.d6() >= 4
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_failure.clone();

        // Java: if (game.getThrower() == null) return;
        if game.thrower_id.is_none() {
            return StepOutcome::goto(&label);
        }

        // Java: HailMaryPass / HailMaryBomb → no interception possible
        if matches!(
            game.thrower_action,
            Some(PlayerAction::HailMaryPass) | Some(PlayerAction::HailMaryBomb)
        ) {
            return StepOutcome::goto(&label);
        }

        let possible_interceptors = Self::find_interceptors(game);
        let do_intercept = !possible_interceptors.is_empty();

        if !do_intercept {
            return StepOutcome::goto(&label)
                .publish(StepParameter::InterceptorId(None));
        }

        // Java: if (!fInterceptorChosen) → showDialog, TurnMode=INTERCEPTION, doNextStep=false
        if !self.interceptor_chosen {
            // client-only: DialogInterceptionParameter — headless waits for CLIENT_INTERCEPTOR_CHOICE
            return StepOutcome::cont();
        }

        // Java: else if (interceptor != null) → switch(intercept(interceptor))
        if let Some(ref interceptor_id) = self.interceptor_id.clone() {
            if !interceptor_id.is_empty() {
                let success = self.intercept(interceptor_id, game, rng);
                if success {
                    return StepOutcome::next()
                        .publish(StepParameter::InterceptorId(Some(interceptor_id.clone())));
                }
            }
        }

        StepOutcome::goto(&label)
            .publish(StepParameter::InterceptorId(None))
    }
}

impl Default for StepIntercept {
    fn default() -> Self { Self::new() }
}

impl Step for StepIntercept {
    fn id(&self) -> StepId { StepId::Intercept }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_INTERCEPTOR_CHOICE → fInterceptorId, fInterceptorChosen = true
        match action {
            Action::SelectPlayer { player_id } => {
                self.interceptor_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
                self.interceptor_chosen = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::InterceptorId(v)       => { self.interceptor_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_intercept() {
        assert_eq!(StepIntercept::new().id(), StepId::Intercept);
    }

    #[test]
    fn no_thrower_goes_to_failure() {
        let mut game = make_game();
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn hail_mary_pass_skips_interception() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::HailMaryPass);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn hail_mary_bomb_skips_interception() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::HailMaryBomb);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepIntercept::new();
        step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into()));
        assert_eq!(step.goto_label_on_failure.as_str(), "new");
    }

    #[test]
    fn set_parameter_interceptor_id_accepted() {
        let mut step = StepIntercept::new();
        step.set_parameter(&StepParameter::InterceptorId(Some("p1".into())));
        assert_eq!(step.interceptor_id.as_deref(), Some("p1"));
    }

    #[test]
    fn select_player_marks_interceptor_chosen() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let action = Action::SelectPlayer { player_id: "p2".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.interceptor_chosen);
        assert_eq!(step.interceptor_id.as_deref(), Some("p2"));
    }

    #[test]
    fn no_possible_interceptors_goes_to_failure() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn no_interceptors_publishes_interceptor_id_null() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InterceptorId(None))));
    }

    #[test]
    fn empty_player_id_on_select_sets_interceptor_id_none() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let action = Action::SelectPlayer { player_id: "".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.interceptor_id.is_none());
    }

    #[test]
    fn find_interceptors_none_when_no_thrower() {
        let game = make_game();
        let interceptors = StepIntercept::find_interceptors(&game);
        assert!(interceptors.is_empty());
    }
}
