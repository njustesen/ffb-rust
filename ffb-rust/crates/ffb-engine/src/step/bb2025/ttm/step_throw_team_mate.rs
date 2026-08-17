/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.ttm.StepThrowTeamMate`.
///
/// Step in the TTM/KTM sequence to perform the actual throw/kick roll. Logic is
/// inlined from `ThrowTeamMateBehaviour.handleExecuteStepHook()` (BB2025 hook).
///
/// BB2025 differences vs BB2020:
///  - Uses `ttm_used` instead of `pass_used` for non-kicked throws.
///  - `evaluatePass`: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE).
///  - Adds Bullseye (canSkipTtmScatterOnSuperbThrow) — client-only: dialog auto-skipped in headless (default = no skip).
///
/// Init param: IS_KICKED_PLAYER (optional).
/// Consumed params: THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL.
use std::collections::HashSet;
use ffb_model::enums::{PassingDistance, PassOutcome, PlayerState, ReRollSource, Rules, SkillId};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_model::events::GameEvent;
use ffb_mechanics::bb2025::pass_mechanic::PassMechanic as Bb2025PassMechanic;
use ffb_mechanics::bb2025::ttm_mechanic::TtmMechanic as Bb2025TtmMechanic;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use crate::action::Action;
use crate::model::step_modifier::RerollHookState;
use crate::skill_behaviour::dispatch;
use crate::step::framework::{Step, StepCommandStatus, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2025::scatter_player::{ScatterPlayer, ScatterPlayerParams};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java `StepThrowTeamMate.StepState` — fields promoted to struct level.
pub struct StepThrowTeamMate {
    /// Java (bb2016): `state.goToLabelOnFailure` — mandatory init param GOTO_LABEL_ON_FAILURE
    /// (stored; only used by the bb2016 edition — this struct serves all editions via make_step).
    pub goto_label_on_failure: String,
    /// Java: `state.thrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `state.thrownPlayerState`
    pub thrown_player_state: Option<PlayerState>,
    /// Java: `state.thrownPlayerHasBall`
    pub thrown_player_has_ball: bool,
    /// Java: `state.passResult`
    pub pass_result: Option<PassOutcome>,
    /// Java: `state.kicked`
    pub kicked: bool,
    /// Java: `state.usingBullseye` — tristate (None = not yet decided)
    pub using_bullseye: Option<bool>,
    /// Java: `fReRolledAction`
    pub re_rolled_action: Option<String>,
    /// Java: `fReRollSource`
    pub re_roll_source: Option<String>,
    /// stored for re-roll prompt
    minimum_roll: i32,
}

impl StepThrowTeamMate {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            pass_result: None,
            kicked: false,
            using_bullseye: None,
            re_rolled_action: None,
            re_roll_source: None,
            minimum_roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.setHasPassed(true); game.setThrowerId; game.setConcessionPossible(false)
        game.acting_player.has_passed = true;
        game.thrower_id = game.acting_player.player_id.clone();
        game.concession_possible = false;

        let rerolled_action_key = if self.kicked { "KICK_TEAM_MATE" } else { "THROW_TEAM_MATE" };
        // BB2020 spends the team's once-per-turn PASS on a Throw Team-Mate
        // (`bb2020/ThrowTeamMateBehaviour` calls setPassUsed, and its `TtmMechanic.isTtmAvailable`
        // is literally `!turnData.isPassUsed()`); BB2025 tracks TTM on its own flag and leaves the
        // pass intact. This shared step runs for both, so under BB2020 a team could throw a
        // team-mate AND still pass in the same turn — visible at the very next activation as
        // java `f0001` vs rust `f0000` (ogre bb2020 seed 6).
        let bb2020 = game.rules == Rules::Bb2020;
        let turn_data = if game.home_playing {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };
        if self.kicked {
            turn_data.ktm_used = true;
        } else if bb2020 {
            turn_data.pass_used = true;
        } else {
            turn_data.ttm_used = true; // BB2025 uses ttm_used, not pass_used
        }

        let mut do_roll = true;
        if self.re_rolled_action.as_deref() == Some(rerolled_action_key) {
            do_roll = false;
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let thrower_id = game.acting_player.player_id.clone().unwrap_or_default();
                let source = ReRollSource::new(source_name);
                if use_reroll(game, &source, &thrower_id) {
                    do_roll = true;
                }
            }
            if !do_roll {
                return self.handle_pass_result();
            }
        }

        if do_roll {
            let thrower_id = match game.acting_player.player_id.clone() {
                Some(id) => id,
                None => return StepOutcome::next(),
            };

            let thrower_coord = game.field_model.player_coordinate(&thrower_id);
            let pass_coord = game.pass_coordinate;

            let pass_mechanic = Bb2025PassMechanic::new();
            let passing_distance = match pass_mechanic.find_passing_distance(game, thrower_coord, pass_coord, true) {
                Some(d) => d,
                None => return StepOutcome::next(),
            };

            let ttm_mechanic = Bb2025TtmMechanic::new();
            // Java: passModifiers = PassModifierFactory.findModifiers(new PassContext(game, thrower,
            //       passingDistance, true)); minimumRoll = ttmMechanic.minimumRoll(distance, passModifiers).
            // Rust previously passed an EMPTY modifier set, so a thrower standing in an opposing tackle
            // zone (e.g. a Treeman at the line of scrimmage) got NO +1 modifier — a roll of 2 evaluated
            // as Inaccurate (throw-scatter 3×d8) instead of Java's Fumble (1×d8 bounce), desyncing the
            // whole TTM (halfling seed 2 i=1). The empty-set minimum_roll/modifier_sum already add the
            // distance modifier; add the factory modifiers (tackle zones, skills, cards) on top, with the
            // PassContext ttm flag = true (so TTM-specific modifier applicability matches Java).
            let factory_total: i32 = {
                let factory = PassModifierFactory::for_rules(game.rules);
                if let Some(thrower) = game.player(&thrower_id) {
                    let ctx = PassContext::new(game, thrower, passing_distance, true);
                    factory.find_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>()
                        + factory.find_skill_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>()
                        + factory.find_card_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>()
                } else { 0 }
            };
            let empty: HashSet<PassModifier> = HashSet::new();
            self.minimum_roll = ttm_mechanic.minimum_roll(passing_distance, &empty) + factory_total;
            let modifier_sum = ttm_mechanic.modifier_sum(passing_distance, &empty) + factory_total;

            let roll = rng.d6();

            // Java: playerCanPass = thrower.getPassing() != 0 (raw PA stat)
            let player_can_pass = game.player(&thrower_id)
                .map(|p| p.passing != 0)
                .unwrap_or(false);
            // Java: evaluatePass(..., thrower.getPassingWithModifiers(), ...) — must use the
            // modified passing value (e.g. temporary PA penalties/bonuses), not the raw stat.
            let passing_value = game.player(&thrower_id)
                .map(|p| p.passing_with_modifiers())
                .unwrap_or(0);

            self.pass_result = Some(if bb2020 {
                evaluate_ttm_pass_bb2020(player_can_pass, passing_value, roll, modifier_sum)
            } else {
                evaluate_ttm_pass_bb2025(player_can_pass, passing_value, roll, modifier_sum)
            });
            let pass_result = self.pass_result.unwrap();

            // Java: successful = ACCURATE || INACCURATE
            let successful = pass_result == PassOutcome::Complete || pass_result == PassOutcome::Inaccurate;

            // Java: ThrowTeamMateBehaviour.handleExecuteStepHook → addReport(new ReportThrowTeamMateRoll(...))
            let re_rolled = self.re_rolled_action.is_some() && self.re_roll_source.is_some();
            let pass_result_name = Some(format!("{:?}", pass_result));
            game.report_list.add(ReportThrowTeamMateRoll::new(
                game.thrower_id.clone(),
                successful,
                roll,
                self.minimum_roll,
                re_rolled,
                vec![],
                Some(format!("{:?}", passing_distance)),
                self.thrown_player_id.clone(),
                pass_result_name,
                self.kicked,
            ));

            // Coverage: `GameEvent::ThrowTeamMateRoll` had no construction site in the engine, so a
            // roll behind 14,762 ThrowTeamMate activations reported nothing. Report-only.
            let roll_event = GameEvent::ThrowTeamMateRoll {
                thrower_id: game.thrower_id.clone().unwrap_or_default(),
                thrown_id: self.thrown_player_id.clone().unwrap_or_default(),
                roll,
                result: pass_result,
            };

            if successful {
                // Java: if ACCURATE && hasSkillProperty(canSkipTtmScatterOnSuperbThrow) && usingBullseye == null
                //   show dialog → Continue (wait for Bullseye decision)
                // Headless: auto-decide false (don't use Bullseye)
                let has_bullseye = game.player(&thrower_id)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_SKIP_TTM_SCATTER_ON_SUPERB_THROW))
                    .unwrap_or(false);
                if pass_result == PassOutcome::Complete && has_bullseye && self.using_bullseye.is_none() {
                    // Server-side auto-decide: don't use Bullseye
                    self.using_bullseye = Some(false);
                }

                // Java StepThrowTeamMate only rolls the throw here and advances — the scatter itself
                // is pushed by the following StepDispatchScatterPlayer (which reads PASS_RESULT +
                // USING_BULLSEYE). Rust previously ALSO pushed a ScatterPlayer sequence here, so the
                // thrown player scattered TWICE (the first from the correct dice, the second an extra
                // that desynced the whole throw — ogre seed 1 i=2). Just publish the result params.
                return self.handle_pass_result()
                    .with_event(roll_event)
                    .publish(StepParameter::UsingBullseye(self.using_bullseye.unwrap_or(false)));
            } else {
                if self.re_rolled_action.is_none() && player_can_pass {
                    let is_fumble = pass_result == PassOutcome::Fumble;
                    if let Some(prompt) = ask_for_reroll_if_available(game, rerolled_action_key, self.minimum_roll, is_fumble) {
                        self.re_rolled_action = Some(rerolled_action_key.into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt).with_event(roll_event);
                    }
                }
                return self.handle_pass_result().with_event(roll_event);
            }
        }

        StepOutcome::next()
    }

    fn handle_pass_result(&self) -> StepOutcome {
        let result = self.pass_result.unwrap_or(PassOutcome::Fumble);
        StepOutcome::next().publish(StepParameter::PassResultParam(result))
    }
}

/// Java: `bb2020/ThrowTeamMateBehaviour.evaluatePass`. Identical to the BB2025 version except for
/// the bottom rung: a throw that lands at 1 or less after modifiers is WILDLY_INACCURATE here,
/// where BB2025 calls it a FUMBLE. The two diverge sharply downstream — BB2020's
/// DispatchScatterPlayer turns WILDLY_INACCURATE into a pass DEVIATE with no throw-scatter, while a
/// FUMBLE drops the player where it stands. Running BB2025's table under BB2020 turned a wild throw
/// into a fumble whose landing then succeeded, leaving a player standing that Java had KO'd
/// (ogre bb2020 seed 8 i=15: java `h08:-1,-1,Ko`, rust `h08:3,8,Standing`).
fn evaluate_ttm_pass_bb2020(player_can_pass: bool, passing_value: i32, roll: i32, modifier_sum: i32) -> PassOutcome {
    if !player_can_pass || passing_value <= 0 {
        return PassOutcome::Fumble;
    }
    if roll == 1 {
        return PassOutcome::Fumble;
    }
    let result_after_modifiers = roll - modifier_sum;
    if roll == 6 || result_after_modifiers >= passing_value {
        PassOutcome::Complete
    } else if result_after_modifiers <= 1 {
        PassOutcome::WildlyInaccurate
    } else {
        PassOutcome::Inaccurate
    }
}

/// Java: ThrowTeamMateBehaviour.evaluatePass (BB2025 version).
/// BB2025 difference: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE).
fn evaluate_ttm_pass_bb2025(player_can_pass: bool, passing_value: i32, roll: i32, modifier_sum: i32) -> PassOutcome {
    if !player_can_pass || passing_value <= 0 {
        return PassOutcome::Fumble;
    }
    if roll == 1 {
        return PassOutcome::Fumble;
    }
    let result_after_modifiers = roll - modifier_sum;
    if roll == 6 || result_after_modifiers >= passing_value {
        PassOutcome::Complete
    } else if result_after_modifiers <= 1 {
        PassOutcome::Fumble // BB2025: fumble (not wildly inaccurate)
    } else {
        PassOutcome::Inaccurate
    }
}

impl Default for StepThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepThrowTeamMate {
    fn id(&self) -> StepId { StepId::ThrowTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: AbstractStep.handleSkillCommand — dispatches CLIENT_USE_SKILL to the
            // registered SkillBehaviour's StepModifier (TheBallista's handleCommandHook
            // presets reRolledAction/reRollSource before the step re-executes).
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::TheBallista => {
                let mut hook_state = RerollHookState {
                    re_rolled_action: self.re_rolled_action.clone(),
                    re_roll_source: self.re_roll_source.clone(),
                    kicked: self.kicked,
                };
                let status = dispatch::handle_skill_command(
                    game, StepId::ThrowTeamMate, &mut hook_state, *skill_id, *use_skill,
                );
                if status == StepCommandStatus::ExecuteStep {
                    self.re_rolled_action = hook_state.re_rolled_action;
                    self.re_roll_source = hook_state.re_roll_source;
                } else if !*use_skill {
                    self.re_rolled_action = None;
                    self.re_roll_source = None;
                }
            }
            Action::UseSkill { use_skill, .. } => {
                if !use_skill {
                    self.re_rolled_action = None;
                    self.re_roll_source = None;
                } else {
                    // UseSkill true during Bullseye dialog: set using_bullseye
                    if self.pass_result == Some(PassOutcome::Complete) && self.using_bullseye.is_none() {
                        self.using_bullseye = Some(true);
                    }
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                // Decline: clear only the reroll SOURCE, keep re_rolled_action set. Java's
                // ThrowTeamMateBehaviour calls step.setReRolledAction(rerolledAction) BEFORE showing
                // the reroll dialog, so a declined reroll (reRollSource == null) re-enters with
                // reRolledAction still == the key and takes the "already rolled" branch —
                // handlePassResult(state.passResult) — accepting the ORIGINAL pass result rather than
                // rolling a fresh pass. Clearing re_rolled_action here made a declined TTM-fumble
                // reroll fall through to do_roll=true and re-roll the pass, turning a 1-scatter FUMBLE
                // into a fresh 3-scatter throw and desyncing the game (ogre seed 1 i=12: home_06's
                // fumbled throw of home_08).
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java (bb2016) init(): GOTO_LABEL_ON_FAILURE (mandatory; stored, only used by
            // the bb2016 edition — this struct serves all editions via make_step)
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)  => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)=> { self.thrown_player_has_ball = *v; true }
            StepParameter::PassResultParam(v)    => { self.pass_result = Some(*v); true }
            StepParameter::IsKickedPlayer(v)     => { self.kicked = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_thrower(game: &mut Game, id: &str, coord: FieldCoordinate, passing: i32) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "thrower".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_thrown_player_has_ball() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerHasBall(true)));
        assert!(step.thrown_player_has_ball);
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassOutcome::Fumble)));
        assert_eq!(step.pass_result, Some(PassOutcome::Fumble));
    }

    #[test]
    fn set_parameter_kicked() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.kicked);
    }

    #[test]
    fn unknown_parameter_rejected() {
        let mut step = StepThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn declined_reroll_keeps_rerolled_action_and_does_not_reroll() {
        // After a fumbled TTM throw the step offers a reroll (re_rolled_action + re_roll_source set).
        // Declining (UseReRoll{false}) must NOT roll a fresh pass: it clears only the source, keeps
        // re_rolled_action, and re-emits the ORIGINAL pass result (FUMBLE). Java keeps reRolledAction
        // set across the decline; clearing it made Rust re-roll the pass (ogre seed 1 i=12).
        let mut game = make_game();
        add_thrower(&mut game, "ogre1", FieldCoordinate::new(10, 7), 3);
        game.acting_player.player_id = Some("ogre1".into());
        game.home_playing = true;

        let mut step = StepThrowTeamMate::new();
        // Simulate the post-offer state: the pass was rolled (FUMBLE) and a reroll was offered.
        step.pass_result = Some(PassOutcome::Fumble);
        step.re_rolled_action = Some("THROW_TEAM_MATE".into());
        step.re_roll_source = Some("TRR".into());

        let mut rng = GameRng::new(0);
        let before = rng.call_count;
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);

        assert_eq!(rng.call_count, before, "declining a TTM reroll must not roll a fresh pass");
        assert_eq!(step.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"),
            "re_rolled_action must be kept so the 'already rolled' branch is taken");
        assert!(step.re_roll_source.is_none(), "the reroll source must be cleared on decline");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassResultParam(PassOutcome::Fumble))),
            "the original FUMBLE result must be re-emitted, not a fresh roll");
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn ttm_used_set_on_non_kicked_throw() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.turn_data_home.ttm_used);
    }

    /// BB2020 spends the team's once-per-turn PASS on a Throw Team-Mate; BB2025 tracks TTM on its
    /// own flag and leaves the pass intact. This shared step runs for both, so under BB2020 a team
    /// could throw a team-mate AND still pass in the same turn — and, worse, the harness kept
    /// re-offering a throw the engine then refused, spinning the turn until the game was abandoned
    /// (9 of 10 ogre bb2020 seeds died on `STUCK_STEP: INIT_SELECTING`).
    #[test]
    fn bb2020_throw_spends_the_pass_not_the_ttm_flag() {
        for (rules, expect_pass, expect_ttm) in
            [(Rules::Bb2020, true, false), (Rules::Bb2025, false, true)]
        {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            game.home_playing = true;
            add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
            game.acting_player.player_id = Some("thrower".into());
            game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
            let mut step = StepThrowTeamMate::new();
            step.thrown_player_id = Some("tp1".into());
            step.start(&mut game, &mut GameRng::new(42));
            assert_eq!(game.turn_data_home.pass_used, expect_pass, "{rules:?} pass_used");
            assert_eq!(game.turn_data_home.ttm_used, expect_ttm, "{rules:?} ttm_used");
        }
    }

    /// The bottom rung of the pass table differs by edition: BB2020 calls a throw that lands at 1 or
    /// less after modifiers WILDLY_INACCURATE, BB2025 calls it a FUMBLE. They diverge sharply
    /// downstream — BB2020 deviates the player (d8+d6 from the thrower) and crash-lands it, BB2025
    /// drops it where it stands. Running BB2025's table under BB2020 turned a wild throw into a
    /// fumble whose landing then succeeded, leaving a player standing that Java had KO'd
    /// (ogre bb2020 seed 8).
    #[test]
    fn evaluate_ttm_pass_bottom_rung_differs_by_edition() {
        // roll 3, modifier 2 → resultAfterModifiers = 1, and 1 <= 1 takes the bottom rung.
        assert_eq!(evaluate_ttm_pass_bb2020(true, 4, 3, 2), PassOutcome::WildlyInaccurate);
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 3, 2), PassOutcome::Fumble);
        // Every other rung agrees between the two editions.
        for (pv, roll, m) in [(4, 1, 0), (4, 6, 0), (4, 5, 0), (4, 4, 1)] {
            assert_eq!(evaluate_ttm_pass_bb2020(true, pv, roll, m),
                       evaluate_ttm_pass_bb2025(true, pv, roll, m),
                       "pv={pv} roll={roll} mod={m} must agree across editions");
        }
        // A player who cannot pass fumbles under both.
        assert_eq!(evaluate_ttm_pass_bb2020(false, 4, 5, 0), PassOutcome::Fumble);
    }

    #[test]
    fn ktm_used_set_on_kicked_throw() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.kicked = true;
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.turn_data_home.ktm_used);
    }

    #[test]
    fn the_ballista_use_skill_true_sets_kick_team_mate_when_kicked() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.kicked = true;

        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::TheBallista, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.re_rolled_action.as_deref(), Some("KICK_TEAM_MATE"));
        assert_eq!(step.re_roll_source.as_deref(), Some("TheBallista"));
    }

    #[test]
    fn the_ballista_use_skill_true_sets_throw_team_mate_when_not_kicked() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());

        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::TheBallista, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"));
    }

    #[test]
    fn the_ballista_use_skill_false_clears_source_but_keeps_action() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());

        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::TheBallista, use_skill: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.re_rolled_action.as_deref(), Some("THROW_TEAM_MATE"));
        assert!(step.re_roll_source.is_none());
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bullseye_use_skill_true_unaffected_by_ballista_wiring() {
        // A non-TheBallista skill-use command must still hit the pre-existing
        // Bullseye branch, not the new TheBallista dispatch path.
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.pass_result = Some(PassOutcome::Complete);

        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Bullseye, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.using_bullseye, Some(true));
        assert!(step.re_rolled_action.is_none());
    }

    #[test]
    fn already_rerolled_no_source_goes_to_next() {
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.re_rolled_action = Some("THROW_TEAM_MATE".into());
        step.re_roll_source = None;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_roll_1_fumble() {
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 1, 0), PassOutcome::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_roll_6_complete() {
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 6, 0), PassOutcome::Complete);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_low_result_fumble_not_wildly_inaccurate() {
        // BB2025 specific: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE)
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 3, 2), PassOutcome::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_inaccurate() {
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 3, 0), PassOutcome::Inaccurate);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_no_passing_stat_fumble() {
        assert_eq!(evaluate_ttm_pass_bb2025(false, 0, 5, 0), PassOutcome::Fumble);
    }

    // Java: ThrowTeamMateBehaviour.handleExecuteStepHook uses
    // `thrower.getPassingWithModifiers()` (not the raw PA stat) as the passValue
    // fed into evaluatePass. A temporary PA modifier must change the pass outcome:
    // with raw PA=6 a roll of 4 is INACCURATE (4 < 6), but with a -3 temporary PA
    // modifier (effective PA=3) the same roll of 4 becomes COMPLETE (4 >= 3).
    #[test]
    fn passing_with_modifiers_used_for_pass_evaluation_not_raw_passing() {
        use ffb_model::model::player::STAT_PA;

        // Find a seed whose first d6() roll is 4.
        let mut seed = 0u64;
        loop {
            if GameRng::new(seed).d6() == 4 { break; }
            seed += 1;
        }

        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 6);
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "thrower") {
            p.add_temporary_stat_mod("test", STAT_PA, -3);
        }
        assert_eq!(game.player("thrower").unwrap().passing_with_modifiers(), 3,
            "temporary PA modifier must lower the effective passing value");

        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(seed));

        assert_eq!(step.pass_result, Some(PassOutcome::Complete),
            "roll of 4 against effective PA 3 (passing_with_modifiers) must be COMPLETE, \
             not INACCURATE as it would be against the raw PA stat of 6");
    }

    #[test]
    fn successful_throw_emits_throw_team_mate_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.report_list.has_report(ReportId::THROW_TEAM_MATE_ROLL));
    }

    #[test]
    fn successful_throw_does_not_push_scatter_sequence() {
        // Regression (ogre seed 1 i=2): the scatter is pushed by the FOLLOWING
        // StepDispatchScatterPlayer, not here. Rust used to push a ScatterPlayer sequence on a
        // successful throw too, so the thrown player scattered TWICE and the throw desynced.
        // A successful throw must only publish the pass result (+ bullseye) and advance.
        let mut seed = 0u64;
        loop { if GameRng::new(seed).d6() == 6 { break; } seed += 1; } // roll 6 → always Complete
        let mut game = make_game();
        game.home_playing = true;
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(step.pass_result, Some(PassOutcome::Complete));
        assert!(out.pushes.is_empty(),
            "successful throw must NOT push a scatter sequence — DispatchScatterPlayer does");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassResultParam(_))),
            "must publish PassResultParam for DispatchScatterPlayer to read");
    }

    #[test]
    fn failed_throw_emits_throw_team_mate_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        // passing=0 → evaluate_ttm_pass_bb2025 returns Fumble (player_can_pass=false)
        add_thrower(&mut game, "thrower", FieldCoordinate::new(10, 7), 0);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::THROW_TEAM_MATE_ROLL));
    }
}
