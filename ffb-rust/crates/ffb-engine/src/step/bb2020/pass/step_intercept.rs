use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_interception_roll::ReportInterceptionRoll;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::passing::can_intercept;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::modifiers::interception_modifier_factory::InterceptionModifierFactory;
use ffb_mechanics::pass_result::PassResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.pass.StepIntercept.
///
/// Interception roll step. Flow:
///  1. Guard: no thrower, or HailMaryPass/HailMaryBomb → `goto_label_on_failure`.
///  2. Find possible interceptors via geometric corridor check.
///  3. If none found → `goto_label_on_failure`.
///  4. If not yet chosen → show DialogInterceptionParameter, set TurnMode=INTERCEPTION, wait.
///  5. Roll agility (AgilityMechanic.minimumRollInterception).
///  6. Success → publish InterceptorId, NEXT_STEP.
///     Failure → `goto_label_on_failure`.
///
/// BB2020 vs BB2025: uses BB2020 InterceptionModifierCollection.
///
/// Needs init param: `GotoLabelOnFailure`.
/// Publishes: `InterceptorId` on success.
pub struct StepIntercept {
    /// Java: fGotoLabelOnFailure (init param, mandatory)
    pub goto_label_on_failure: String,
    /// Java: interceptionSkill (stored as skill name until Skill fully ported)
    pub interception_skill_name: Option<String>,
    /// Java: PassState.interceptorId — set from CLIENT_INTERCEPTOR_CHOICE command
    pub interceptor_id: Option<String>,
    /// Java: PassState.interceptorChosen — set when CLIENT_INTERCEPTOR_CHOICE received
    pub interceptor_chosen: bool,
    /// Java: PassState.originalBombardier — non-empty means throw was a bomb from a bombardier
    pub original_bombardier: Option<String>,
    /// Java: PassState.result — the PassResult from StepPass
    pub pass_result: PassResult,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepIntercept {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            interception_skill_name: None,
            interceptor_id: None,
            interceptor_chosen: false,
            original_bombardier: None,
            pass_result: PassResult::INACCURATE,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    /// Java: UtilPassing.findInterceptors(game, thrower, passCoordinate)
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

    /// Java: intercept(pInterceptor, passState) — rolls agility, checks modifiers.
    ///
    /// Returns `(successful, easy_intercept)`. Java only sets
    /// `passState.setInterceptionSuccessful(true)` when `successful && easyIntercept`
    /// (see `StepIntercept.intercept()`); a normal (non-easy) successful interception
    /// only flows into `state.setDeflectionSuccessful(true)` in `executeStep()` and
    /// still requires a catch roll via `StepResolvePass`'s DEFLECTED path.
    fn intercept(&self, interceptor_id: &str, game: &mut Game, rng: &mut GameRng) -> (bool, bool) {
        let (easy_intercept, minimum_roll, roll) = {
            let interceptor = match game.player(interceptor_id) {
                Some(p) => p,
                None => return (false, false),
            };

            // Java: easyIntercept = interceptionSkill != null && pInterceptor.hasUnused(interceptionSkill)
            let easy_intercept = self.interception_skill_name
                .as_deref()
                .map(|_| interceptor.has_skill_property(NamedProperties::CAN_INTERCEPT_EASILY))
                .unwrap_or(false);

            let roll = rng.d6();

            let minimum_roll = if easy_intercept {
                // Java: minimumRoll = 2, no modifiers applied
                2
            } else {
                // Java: BB2020 InterceptionModifierFactory.findModifiers(new InterceptionContext(...))
                let factory = InterceptionModifierFactory::for_rules(game.rules);
                let is_bomb = self.original_bombardier.is_some();
                let mods = factory.find_applicable(game, interceptor, self.pass_result, is_bomb);
                let skill_mods = factory.find_skill_modifiers(game, interceptor);
                let card_mods = factory.find_card_modifiers(game, interceptor);
                let all: Vec<&ffb_mechanics::modifiers::interception_modifier::InterceptionModifier> = mods.iter().copied().chain(skill_mods.iter()).chain(card_mods.iter()).collect();
                // Java: AgilityMechanic.minimumRollInterception(pInterceptor, interceptionModifiers)
                InterceptionModifierFactory::minimum_roll_bb2020(interceptor, &all)
            };
            (easy_intercept, minimum_roll, roll)
        };

        let successful = roll >= minimum_roll;
        let re_rolled = self.re_rolled_action.is_some() && self.re_roll_source.is_some();
        let is_bomb = matches!(
            game.thrower_action,
            Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
        );

        // Java: if (easyIntercept && !reRolled) addReport(new ReportSkillUse(..., SkillUse.EASY_INTERCEPT))
        if easy_intercept && !re_rolled {
            let skill_id = game.player(interceptor_id)
                .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_INTERCEPT_EASILY));
            if let Some(sid) = skill_id {
                game.report_list.add(ReportSkillUse::new(
                    Some(interceptor_id.to_string()),
                    sid,
                    true,
                    SkillUse::EASY_INTERCEPT,
                ));
            }
        }

        // Java: getResult().addReport(new ReportInterceptionRoll(interceptor_id, successful, roll, minimumRoll,
        //   reRolled, modifiers, isBomb, easyIntercept))
        game.report_list.add(ReportInterceptionRoll::new(
            Some(interceptor_id.to_string()),
            successful,
            roll,
            minimum_roll,
            re_rolled,
            vec![],
            is_bomb,
            easy_intercept,
        ));

        (successful, easy_intercept)
    }
}

impl Step for StepIntercept {
    fn id(&self) -> StepId { StepId::Intercept }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_INTERCEPTOR_CHOICE → passState.setInterceptorId, setInterceptorChosen(true),
        //       interceptionSkill = command.getInterceptionSkill()
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
            StepParameter::InterceptorId(v) => { self.interceptor_id = v.clone(); true }
            StepParameter::PassResultParam(v) => {
                self.pass_result = match v {
                    ffb_model::enums::PassOutcome::Complete => PassResult::ACCURATE,
                    ffb_model::enums::PassOutcome::Inaccurate => PassResult::INACCURATE,
                    ffb_model::enums::PassOutcome::WildlyInaccurate => PassResult::WILDLY_INACCURATE,
                    ffb_model::enums::PassOutcome::Fumble
                    | ffb_model::enums::PassOutcome::Caught
                    | ffb_model::enums::PassOutcome::MissedCatch => PassResult::FUMBLE,
                };
                true
            }
            _ => false,
        }
    }
}

impl StepIntercept {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_failure.clone();

        // Java guard: no thrower → goto failure
        if game.thrower_id.is_none() {
            return StepOutcome::goto(&label);
        }
        // Java guard: HailMaryPass / HailMaryBomb → no interception possible
        if matches!(
            game.thrower_action,
            Some(PlayerAction::HailMaryBomb) | Some(PlayerAction::HailMaryPass)
        ) {
            return StepOutcome::goto(&label);
        }

        // Java: possibleInterceptors = UtilPassing.findInterceptors(game, thrower, passCoordinate)
        let possible_interceptors = Self::find_interceptors(game);

        // Java: boolean doIntercept = (possibleInterceptors.length > 0)
        if possible_interceptors.is_empty() {
            return StepOutcome::goto(&label);
        }

        // Java: if (!state.isInterceptorChosen()) → showDialog, TurnMode=INTERCEPTION, doNextStep=false
        if !self.interceptor_chosen {
            // client-only: DialogInterceptionParameter — headless waits for CLIENT_INTERCEPTOR_CHOICE command
            return StepOutcome::cont();
        }

        // Java: else if (interceptor != null) → intercept(interceptor, state)
        let (do_intercept, easy_intercept) = if let Some(ref interceptor_id) = self.interceptor_id.clone() {
            let (success, easy) = self.intercept(interceptor_id, game, rng);
            if success {
                let is_bomb = matches!(
                    game.thrower_action,
                    Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
                );
                if is_bomb {
                    game.field_model.bomb_moving = false;
                } else {
                    game.field_model.ball_moving = false;
                }
            }
            (success, easy)
        } else {
            (false, false)
        };

        if do_intercept {
            let interceptor_id = self.interceptor_id.clone();
            let mut outcome = StepOutcome::next()
                .publish(StepParameter::InterceptorId(interceptor_id));
            // Java: passState.setInterceptionSuccessful(true) is set ONLY inside the
            // `easyIntercept` branch of `intercept()` on success — a normal successful
            // interception leaves it false (and only sets deflectionSuccessful), so
            // StepResolvePass still routes it through a catch roll (DEFLECTED path).
            if easy_intercept {
                outcome = outcome.publish(StepParameter::InterceptionSuccessful(true));
            }
            outcome
        } else {
            StepOutcome::goto(&label)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_thrower_goes_to_failure() {
        let mut game = make_game();
        let mut step = StepIntercept::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn hail_mary_pass_skips_interception() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::HailMaryPass);
        let mut step = StepIntercept::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn hail_mary_bomb_skips_interception() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::HailMaryBomb);
        let mut step = StepIntercept::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepIntercept::new("old".into());
        step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into()));
        assert_eq!(step.goto_label_on_failure.as_str(), "new");
    }

    #[test]
    fn select_player_marks_interceptor_chosen() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut step = StepIntercept::new("fail".into());
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
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        let mut step = StepIntercept::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn decline_interception_goes_to_failure() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn set_parameter_pass_result_accepted() {
        let mut step = StepIntercept::new("fail".into());
        let accepted = step.set_parameter(&StepParameter::PassResultParam(ffb_model::enums::PassOutcome::Complete));
        assert!(accepted);
        assert_eq!(step.pass_result, PassResult::ACCURATE);
    }

    #[test]
    fn find_interceptors_finds_corridor_player() {
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        home.players.push(thrower);

        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 2;
        away.players.push(opp);

        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));

        let interceptors = StepIntercept::find_interceptors(&game);
        assert_eq!(interceptors.len(), 1);
        assert_eq!(interceptors[0], "opp1");
    }

    #[test]
    fn easy_intercept_success_publishes_interception_successful() {
        // Java: passState.setInterceptionSuccessful(true) only fires inside the
        // `easyIntercept` branch of `intercept()` on a successful roll (minimumRoll=2,
        // so any d6 >= 2 succeeds — 5/6 chance per seed).
        use ffb_model::enums::{PlayerState as PS, PS_STANDING, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        home.players.push(thrower);

        let mut interceptor = ffb_model::model::player::Player::default();
        interceptor.id = "opp1".into();
        interceptor.agility = 1;
        interceptor.starting_skills.push(SkillWithValue::new(SkillId::Yoink));
        away.players.push(interceptor);

        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(PS_STANDING));

        let mut found_success = false;
        for seed in 0u64..30 {
            let mut game2 = game.clone();
            let mut step = StepIntercept::new("fail".into());
            step.interceptor_chosen = true;
            step.interceptor_id = Some("opp1".into());
            step.interception_skill_name = Some("Yoink".into());
            let out = step.start(&mut game2, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                let has_success = out.published.iter().any(|p| {
                    matches!(p, StepParameter::InterceptionSuccessful(true))
                });
                assert!(has_success, "seed {seed}: expected InterceptionSuccessful(true) for easy intercept success");
                found_success = true;
                break;
            }
        }
        assert!(found_success, "no seed in 0..30 produced a successful easy intercept roll");
    }

    #[test]
    fn non_easy_intercept_success_does_not_publish_interception_successful() {
        // A normal (non-easy) successful interception must NOT publish
        // InterceptionSuccessful — only DeflectionSuccessful semantics apply (handled by
        // StepResolvePass defaulting deflection_successful=true off InterceptorId alone).
        use ffb_model::enums::{PlayerState as PS, PS_STANDING};
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        home.players.push(thrower);

        let mut interceptor = ffb_model::model::player::Player::default();
        interceptor.id = "opp1".into();
        interceptor.agility = 1; // low minimum roll without easy-intercept skill
        away.players.push(interceptor);

        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(PS_STANDING));

        let mut found_success = false;
        for seed in 0u64..30 {
            let mut game2 = game.clone();
            let mut step = StepIntercept::new("fail".into());
            step.interceptor_chosen = true;
            step.interceptor_id = Some("opp1".into());
            // no interception_skill_name set → easy_intercept is always false
            let out = step.start(&mut game2, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                let has_interceptor = out.published.iter().any(|p| matches!(p, StepParameter::InterceptorId(Some(id)) if id == "opp1"));
                assert!(has_interceptor, "seed {seed}: expected InterceptorId published");
                let has_success = out.published.iter().any(|p| matches!(p, StepParameter::InterceptionSuccessful(_)));
                assert!(!has_success, "seed {seed}: non-easy successful intercept must not publish InterceptionSuccessful");
                found_success = true;
                break;
            }
        }
        assert!(found_success, "no seed in 0..30 produced a successful non-easy intercept roll");
    }

    #[test]
    fn interception_roll_report_added_when_interceptor_chosen() {
        use ffb_model::enums::{PlayerState as PS, PS_STANDING};
        use ffb_model::report::report_id::ReportId;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        home.players.push(thrower);

        let mut interceptor = ffb_model::model::player::Player::default();
        interceptor.id = "opp1".into();
        interceptor.agility = 4;
        away.players.push(interceptor);

        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(PS_STANDING));

        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = Some("opp1".into());
        step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.report_list.has_report(ReportId::INTERCEPTION_ROLL),
            "expected ReportInterceptionRoll in report_list after intercept attempt"
        );
    }

    #[test]
    fn interception_failure_adds_report_to_report_list() {
        use ffb_model::enums::{PlayerState as PS, PS_STANDING};
        use ffb_model::report::report_id::ReportId;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        home.players.push(thrower);

        // interceptor with low agility (likely to fail)
        let mut interceptor = ffb_model::model::player::Player::default();
        interceptor.id = "opp1".into();
        interceptor.agility = 1;
        away.players.push(interceptor);

        let mut game = Game::new(home, away, Rules::Bb2020);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(PS_STANDING));

        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = Some("opp1".into());
        step.start(&mut game, &mut GameRng::new(0));

        assert!(
            game.report_list.has_report(ReportId::INTERCEPTION_ROLL),
            "expected ReportInterceptionRoll even on failed intercept"
        );
    }
}
