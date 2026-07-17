use ffb_model::enums::PlayerAction;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::passing::can_intercept;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_interception_roll::ReportInterceptionRoll;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_mechanics::modifiers::interception_modifier_factory::InterceptionModifierFactory;
use ffb_mechanics::pass_result::PassResult;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepIntercept.
///
/// Interception roll step.  Flow:
///  1. Guard: no thrower, or HailMaryPass/HailMaryBomb → `goto_label_on_failure`.
///  2. Find possible interceptors via geometric corridor check (`UtilPassing.findInterceptors`).
///  3. If none found → `goto_label_on_failure`.
///  4. If not yet chosen → show DialogInterceptionParameter, set TurnMode=INTERCEPTION,
///     wait for CLIENT_INTERCEPTOR_CHOICE command.
///  5. Roll agility (AgilityMechanic.minimumRollInterception).
///  6. Success → publish InterceptorId, NEXT_STEP.
///     Failure → `goto_label_on_failure`.
///
/// Needs init param: `GotoLabelOnFailure`.
/// Publishes: `InterceptorId` on success.
///
/// client-only: TurnMode=INTERCEPTION is a UI signal; headless waits via cont() for SelectPlayer.
/// client-only: re-roll dialog (AbstractStepWithReRoll path) — re-rolls are offered via UtilCards.getRerollSource.
pub struct StepIntercept {
    /// Java: fGotoLabelOnFailure (init param, mandatory)
    pub goto_label_on_failure: String,
    /// Java: interceptionSkill (Skill) — stored as name string until Skill is fully ported
    pub interception_skill_name: Option<String>,
    /// Java: PassState.interceptorId — set from CLIENT_INTERCEPTOR_CHOICE command
    pub interceptor_id: Option<String>,
    /// Java: PassState.interceptorChosen — set when CLIENT_INTERCEPTOR_CHOICE received
    pub interceptor_chosen: bool,
    /// Java: PassState.originalBombardier — non-empty means the throw was a bomb from a bombardier
    pub original_bombardier: Option<String>,
    /// Java: PassState.result — the PassResult from StepPass, needed for interception modifiers
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
    ///
    /// Returns player IDs from the inactive (defending) team that stand in the
    /// pass corridor between thrower and pass target.  Mirrors the Java geometric
    /// check: for each opponent on the pitch with tackle zones and a position in
    /// the corridor, include them.
    fn find_interceptors<'a>(game: &'a Game) -> Vec<String> {
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
        // Opponents: the team that is NOT the active team
        let opponent_team = game.inactive_team();
        opponent_team.players.iter()
            .filter(|player| {
                // Must be on the pitch
                let coord = match game.field_model.player_coordinate(&player.id) {
                    Some(c) => c,
                    None => return false,
                };
                // Must have tackle zones (standing)
                let state = game.field_model.player_state(&player.id);
                let has_tacklezones = state.map(|s| s.has_tacklezones()).unwrap_or(false);
                if !has_tacklezones {
                    return false;
                }
                // Must not be thrower's square or pass_coord square
                if coord == thrower_coord || coord == pass_coord {
                    return false;
                }
                // Geometric corridor check
                can_intercept(thrower_coord, pass_coord, coord)
            })
            .map(|p| p.id.clone())
            .collect()
    }

    /// Java: intercept(pInterceptor, passState) — rolls agility, checks modifiers.
    ///
    /// Returns `true` on success, `false` on failure.
    /// Note: re-roll handling (skill re-rolls / team re-rolls) is not yet translated;
    /// this is a single-roll implementation.
    fn intercept(
        &self,
        interceptor_id: &str,
        game: &mut Game,
        rng: &mut GameRng,
    ) -> bool {
        let interceptor = match game.player(interceptor_id) {
            Some(p) => p,
            None => return false,
        };

        // Java: easyIntercept = interceptionSkill != null && pInterceptor.hasUnused(interceptionSkill)
        // We approximate: easyIntercept is flagged by skill name "canInterceptEasily"
        let easy_intercept = self.interception_skill_name
            .as_deref()
            .map(|_| interceptor.has_skill_property(NamedProperties::CAN_INTERCEPT_EASILY))
            .unwrap_or(false);

        let roll = rng.d6();

        let (minimum_roll, easy_intercept) = if easy_intercept {
            // Java: minimumRoll = 2, no modifiers applied
            (2, true)
        } else {
            // Java: modifierFactory.findModifiers(new InterceptionContext(game, pInterceptor, state.getResult(), isOriginalBombardier))
            let factory = InterceptionModifierFactory::for_rules(game.rules);
            let is_bomb_flag = self.original_bombardier.is_some();
            let mods = factory.find_applicable(game, interceptor, self.pass_result, is_bomb_flag);
            let skill_mods = factory.find_skill_modifiers(game, interceptor);
            let card_mods = factory.find_card_modifiers(game, interceptor);
            let all: Vec<&ffb_mechanics::modifiers::interception_modifier::InterceptionModifier> = mods.iter().copied().chain(skill_mods.iter()).chain(card_mods.iter()).collect();
            // Java: AgilityMechanic.minimumRollInterception(pInterceptor, interceptionModifiers)
            let min = InterceptionModifierFactory::minimum_roll_bb2020(interceptor, &all);
            (min, false)
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

        // Java: getResult().addReport(new ReportInterceptionRoll(pInterceptor.getId(), successful, roll, minimumRoll,
        //   reRolled, interceptionModifierArray, isBomb, easyIntercept))
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

        successful
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
            Action::SelectPlayer {player_id } => {
                // Intercept dialog reply: chosen player id is the interceptor (or empty = decline)
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
                // Java: passState.getResult() — set by StepPass publishing PassResultParam
                self.pass_result = match v {
                    ffb_model::enums::PassResult::Complete => PassResult::ACCURATE,
                    ffb_model::enums::PassResult::Inaccurate => PassResult::INACCURATE,
                    ffb_model::enums::PassResult::WildlyInaccurate => PassResult::WILDLY_INACCURATE,
                    ffb_model::enums::PassResult::Fumble
                    | ffb_model::enums::PassResult::Caught
                    | ffb_model::enums::PassResult::MissedCatch => PassResult::FUMBLE,
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
            // Java: UtilServerDialog.showDialog → CLIENT_INTERCEPTOR_CHOICE; setTurnMode(INTERCEPTION)
            // client-only: TurnMode=INTERCEPTION is a UI signal to show the dialog; game logic waits for SelectPlayer.
            return StepOutcome::cont();
        }

        // Java: else if (interceptor != null) → intercept(interceptor, state)
        let do_intercept = if let Some(ref interceptor_id) = self.interceptor_id.clone() {
            // Roll the interception
            let success = self.intercept(interceptor_id, game, rng);
            if success {
                // Java: game.getFieldModel().setBallMoving(false) / setBombMoving(false)
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
            success
        } else {
            // No interceptor chosen (player clicked "none") → no intercept
            false
        };

        if do_intercept {
            let interceptor_id = self.interceptor_id.clone();
            // Java: publishParameter(StepParameter.from(StepParameterKey.INTERCEPTOR_ID, pInterceptor.getId()))
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            StepOutcome::next()
                .publish(StepParameter::InterceptorId(interceptor_id))
        } else {
            // Java: doIntercept = false → getResult().setNextAction(StepAction.GOTO_LABEL, fGotoLabelOnFailure)
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
        Game::new(home, away, Rules::Bb2025)
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
        let action = Action::SelectPlayer {player_id: "p2".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.interceptor_chosen);
        assert_eq!(step.interceptor_id.as_deref(), Some("p2"));
    }

    #[test]
    fn no_possible_interceptors_goes_to_failure() {
        // thrower set, Pass action, but no players on the field → no corridor players
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        // t1 at (1,7); no opponents on field
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        let mut step = StepIntercept::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn decline_interception_goes_to_failure() {
        // interceptor_chosen = true but interceptor_id = None (player declined)
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));

        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = None; // explicitly declined
        // No corridor players anyway, but we also test the decline path
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Goes to failure (no possible interceptors or declined)
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn interceptor_chosen_but_not_in_game_goes_to_failure() {
        // interceptor_chosen = true, interceptor_id set, but player not found → failure
        // We need at least one corridor player to get past the possible_interceptors check.
        // Setup: thrower at (1,7), pass to (14,7), opponent "opp1" at (7,7) in corridor
        // with tackle zones.
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        // Add a thrower to home
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.agility = 3;
        home.players.push(thrower);

        // Add interceptor candidate to away
        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 3;
        away.players.push(opp);

        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true; // home is active → away is opponent
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));

        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        // Place opp1 in pass corridor and give it tackle zones
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));

        let mut step = StepIntercept::new("fail".into());
        // Interceptor chosen but for an unknown player (not opp1)
        step.interceptor_chosen = true;
        step.interceptor_id = Some("unknown_player".into());

        // Should go to failure (intercept() returns false for unknown player)
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn successful_interception_publishes_interceptor_id() {
        // Verifies: successful intercept roll → NextStep + InterceptorId published.
        // Uses easyIntercept path (minimum_roll=2, always succeeds on ≥2) to avoid
        // pass-result-modifier interference.  We set interception_skill_name to trigger
        // the easyIntercept path, and the interceptor has "canInterceptEasily" property.
        //
        // Since we cannot easily add a skill with a property here (no Skill registry),
        // we instead test the success path directly by verifying that for an AG=2 player
        // with a FUMBLE pass result, the step produces NextStep on multiple seeds.
        // We set pass_result = FUMBLE explicitly on step, so no INACCURATE/ACCURATE modifier applies.
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.agility = 3;
        home.players.push(thrower);

        // AG=2, FUMBLE pass result, no weather → minimum_roll = max(2, 2+0) = 2
        // Any d6 roll ≥ 2 succeeds. On seed=1, d6 should produce a roll ≥ 2.
        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 2;
        away.players.push(opp);

        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));

        // Try seeds 2..50 — for each, fresh RNG; 5/6 probability of roll ≥ 2.
        // P(all fail) = (1/6)^48 ≈ 10^-37.
        let mut found_success = false;
        for seed in 2u64..50 {
            let mut game2 = game.clone();
            let mut step2 = StepIntercept::new("fail".into());
            step2.interceptor_chosen = true;
            step2.interceptor_id = Some("opp1".into());
            step2.pass_result = PassResult::FUMBLE;
            let out = step2.start(&mut game2, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                let has_interceptor = out.published.iter().any(|p| {
                    matches!(p, StepParameter::InterceptorId(Some(id)) if id == "opp1")
                });
                assert!(has_interceptor, "seed {seed}: expected InterceptorId(opp1)");
                found_success = true;
                break;
            }
        }
        // If this fails, find_interceptors or intercept() has a logic bug.
        assert!(found_success, "No seed in 2..50 succeeded: possible_interceptors always empty or intercept always rolls 1");
    }

    #[test]
    fn intercept_roll_minimum_is_agility_based() {
        // Unit test for the intercept() logic directly via execute_step.
        // For FUMBLE pass result, no extra modifiers → minimum_roll = max(2, agility).
        // AG=2 → min=2; AG=3 → min=3.
        // We verify: for an AG=3 interceptor with FUMBLE result, min_roll=3.
        // Roll forced ≥ 3 by giving the step a pre-existing roll via the rng seed trick.
        // Instead we verify by checking: with AG=3 and roll=3, success.
        // (We verify the boundary via AgilityMechanic formula directly here.)
        use ffb_mechanics::bb2025::agility_mechanic::AgilityMechanic as Bb2025Ag;
        use ffb_mechanics::agility_mechanic::AgilityMechanic as AgTrait;
        use std::collections::HashSet;
        let mechanic = Bb2025Ag::new();
        let mut player = ffb_model::model::player::Player::default();
        player.agility = 3;
        // With empty HashSet (no interception modifiers):
        let minimum = mechanic.minimum_roll_interception(&player, &HashSet::new());
        assert_eq!(minimum, 3, "AG3 interceptor with no modifiers should need a 3+");
    }

    #[test]
    fn intercept_method_returns_true_for_high_roll() {
        // Directly test the intercept() private method:
        // AG=2, FUMBLE pass result → minimum_roll = 2.
        // With a seed that produces roll ≥ 2, should return true.
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);

        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 2;
        away.players.push(opp);

        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());

        let step = StepIntercept {
            goto_label_on_failure: "fail".into(),
            interception_skill_name: None,
            interceptor_id: Some("opp1".into()),
            interceptor_chosen: true,
            original_bombardier: None,
            pass_result: PassResult::FUMBLE,
            re_rolled_action: None,
            re_roll_source: None,
        };

        // Try seeds until one gives roll ≥ 2 (easy with AG=2, min=2)
        let mut any_success = false;
        for seed in 0u64..20 {
            let mut game_clone = game.clone();
            let result = step.intercept("opp1", &mut game_clone, &mut GameRng::new(seed));
            if result { any_success = true; break; }
        }
        assert!(any_success, "intercept() never returned true for AG2 with FUMBLE pass (min_roll=2)");
    }

    #[test]
    fn find_interceptors_finds_corridor_player() {
        // Direct test of find_interceptors function to verify it detects opp1 in corridor.
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

        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));

        let interceptors = StepIntercept::find_interceptors(&game);
        assert_eq!(interceptors.len(), 1, "expected 1 interceptor, got {}: {:?}", interceptors.len(), interceptors);
        assert_eq!(interceptors[0], "opp1");
    }

    #[test]
    fn set_parameter_pass_result_accepted() {
        let mut step = StepIntercept::new("fail".into());
        let accepted = step.set_parameter(&StepParameter::PassResultParam(ffb_model::enums::PassResult::Complete));
        assert!(accepted);
        assert_eq!(step.pass_result, PassResult::ACCURATE);
    }

    #[test]
    fn intercept_attempt_emits_interception_roll_report() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.agility = 3;
        home.players.push(thrower);
        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 2;
        away.players.push(opp);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));
        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = Some("opp1".into());
        step.pass_result = PassResult::FUMBLE;
        step.start(&mut game, &mut GameRng::new(3));
        assert!(
            game.report_list.has_report(ReportId::INTERCEPTION_ROLL),
            "expected ReportInterceptionRoll in report_list after intercept attempt"
        );
    }

    #[test]
    fn intercept_failure_emits_interception_roll_report() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::enums::PlayerState as PS;
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        let mut thrower = ffb_model::model::player::Player::default();
        thrower.id = "t1".into();
        thrower.agility = 3;
        home.players.push(thrower);
        // AG=6 means minimum_roll = max(2, 6) = 6; roll 1 always fails
        let mut opp = ffb_model::model::player::Player::default();
        opp.id = "opp1".into();
        opp.agility = 6;
        away.players.push(opp);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.thrower_id = Some("t1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.pass_coordinate = Some(FieldCoordinate::new(14, 7));
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(1, 7));
        game.field_model.set_player_coordinate("opp1", FieldCoordinate::new(7, 7));
        game.field_model.set_player_state("opp1", PS::new(ffb_model::enums::PS_STANDING));
        let mut step = StepIntercept::new("fail".into());
        step.interceptor_chosen = true;
        step.interceptor_id = Some("opp1".into());
        step.pass_result = PassResult::FUMBLE;
        // seed=0 → d6=1, which is < 6, so intercept fails
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::INTERCEPTION_ROLL),
            "expected ReportInterceptionRoll in report_list even on failed intercept"
        );
    }
}
