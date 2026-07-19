/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepThrowTeamMate`.
///
/// Step in TTM sequence to perform the actual throw roll. Logic is inlined from
/// `ThrowTeamMateBehaviour.handleExecuteStepHook()` (Java hook dispatch).
///
/// Init param: GOTO_LABEL_ON_FAILURE (mandatory).
/// Consumed params: THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL.
use ffb_model::enums::{PassingDistance, PlayerState, ReRollSource};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2016::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_mechanics::bb2016::pass_mechanic::PassMechanic as Bb2016PassMechanic;
use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2016::scatter_player::{ScatterPlayer, ScatterPlayerParams};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepThrowTeamMate` inner `StepState` — fields promoted to struct level.
pub struct StepThrowTeamMate {
    /// Java: `state.goToLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `state.thrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `state.thrownPlayerState`
    thrown_player_state: Option<PlayerState>,
    /// Java: `state.thrownPlayerHasBall`
    thrown_player_has_ball: bool,
    /// Java: `fReRolledAction` — set when a re-roll is in progress ("THROW_TEAM_MATE")
    re_rolled_action: Option<String>,
    /// Java: `fReRollSource`
    re_roll_source: Option<String>,
    /// Java: stored minimumRoll for re-roll prompt
    minimum_roll: i32,
}

impl StepThrowTeamMate {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            re_rolled_action: None,
            re_roll_source: None,
            minimum_roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.setHasPassed(true); game.setConcessionPossible(false); turnData.setPassUsed(true)
        game.acting_player.has_passed = true;
        game.concession_possible = false;
        let turn_data = if game.home_playing {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };
        turn_data.pass_used = true;

        // Java: if (ReRolledActions.THROW_TEAM_MATE == getReRolledAction()) { useReRoll or goto failure }
        let mut do_roll = true;
        if self.re_rolled_action.as_deref() == Some("THROW_TEAM_MATE") {
            do_roll = false;
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let thrower_id = game.acting_player.player_id.clone().unwrap_or_default();
                let source = ReRollSource::new(source_name);
                if use_reroll(game, &source, &thrower_id) {
                    do_roll = true;
                }
            }
            if !do_roll {
                return StepOutcome::goto(&self.goto_label_on_failure);
            }
        }

        if do_roll {
            let thrower_id = match game.acting_player.player_id.clone() {
                Some(id) => id,
                None => return StepOutcome::next(),
            };

            let thrower_coord = game.field_model.player_coordinate(&thrower_id);
            let pass_coord = game.pass_coordinate;

            let pass_mechanic = Bb2016PassMechanic::new();
            let passing_distance = match pass_mechanic.find_passing_distance(game, thrower_coord, pass_coord, true) {
                Some(d) => d,
                None => return StepOutcome::next(),
            };

            // Java: passModifierFactory.findModifiers(new PassContext(game, thrower, passingDistance, true))
            // — REGULAR (weather, e.g. Blizzard/Very Sunny) + TACKLEZONE (isAffectedByTackleZones is
            // always true when context.isTtm()) + DISTURBING_PRESENCE + skill modifiers (notably
            // bb2016.ThrowTeamMate itself registers a "+1 Throw Team-Mate" penalty whenever isTtm(),
            // plus Accurate/StrongArm/Stunty) + card modifiers.
            let thrower_player = match game.player(&thrower_id) {
                Some(p) => p,
                None => return StepOutcome::next(),
            };
            let factory = PassModifierFactory::for_rules(game.rules);
            let ctx = PassContext::new(game, thrower_player, passing_distance, true);
            let collection_mods = factory.find_modifiers(&ctx);
            let skill_mods = factory.find_skill_modifiers(&ctx);
            let card_mods = factory.find_card_modifiers(&ctx);
            let modifier_total: i32 = collection_mods.iter().map(|m| m.get_modifier()).sum::<i32>()
                + skill_mods.iter().map(|m| m.get_modifier()).sum::<i32>()
                + card_mods.iter().map(|m| m.get_modifier()).sum::<i32>();
            let modifier_names: Vec<String> = collection_mods.iter().map(|m| m.get_report_string().to_string())
                .chain(skill_mods.iter().map(|m| m.get_report_string().to_string()))
                .chain(card_mods.iter().map(|m| m.get_report_string().to_string()))
                .collect();

            // Java: TtmMechanic.minimumRoll(distance, modifiers) = max(2, 2 + modifierSum)
            //       modifierSum = sum(modifiers) - distance.getModifier2016()
            self.minimum_roll = (2 + (modifier_total - passing_distance.modifier_2016())).max(2);

            let roll = rng.d6();

            // Java: successful = !DiceInterpreter.isPassFumble(roll, passingDistance, passModifiers)
            let successful = !is_ttm_fumble(roll, passing_distance, modifier_total);

            // Java: boolean reRolled = (getReRolledAction() == THROW_TEAM_MATE && getReRollSource() != null);
            //       getResult().addReport(new ReportThrowTeamMateRoll(...))
            let re_rolled = self.re_rolled_action.as_deref() == Some("THROW_TEAM_MATE") && self.re_roll_source.is_some();
            game.report_list.add(ReportThrowTeamMateRoll::new(
                Some(thrower_id.clone()),
                successful,
                roll,
                self.minimum_roll,
                re_rolled,
                modifier_names,
                Some(format!("{:?}", passing_distance)),
                self.thrown_player_id.clone().unwrap_or_default(),
            ));

            if successful {
                // Java: scattersSingleDirection = thrownPlayer.hasSkillProperty(ttmScattersInSingleDirection)
                let scatters_single = self.thrown_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION))
                    .unwrap_or(false);

                // Java: push ScatterPlayer sequence
                let scatter_params = ScatterPlayerParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    thrown_player_state: self.thrown_player_state,
                    thrown_player_has_ball: self.thrown_player_has_ball,
                    thrown_player_coordinate: thrower_coord,
                    throw_scatter: true,
                    has_swoop: scatters_single,
                };
                let seq = ScatterPlayer::build_sequence(&scatter_params);
                return StepOutcome::next().push_seq(seq);
            } else {
                // Java: if (getReRolledAction() != ReRolledActions.THROW_TEAM_MATE) → try reroll
                if self.re_rolled_action.is_none() {
                    // Java: check unused passing reroll source (skill-based)
                    // client-only: DialogSkillUseParameter for skill re-roll source — headless auto-uses skill re-roll
                    // Fall through to team reroll check.
                    if let Some(prompt) = ask_for_reroll_if_available(game, "THROW_TEAM_MATE", self.minimum_roll, true) {
                        self.re_rolled_action = Some("THROW_TEAM_MATE".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
                return StepOutcome::goto(&self.goto_label_on_failure);
            }
        }

        StepOutcome::next()
    }
}

/// Java: `DiceInterpreter.isPassFumble(roll, passingDistance, passModifiers)` for TTM.
/// Roll 1 → fumble. Roll 6 → not fumble.
/// Otherwise: (roll + distance_modifier2016 - modifierTotal) <= 1.
fn is_ttm_fumble(roll: i32, distance: PassingDistance, modifier_total: i32) -> bool {
    if roll == 1 { return true; }
    if roll == 6 { return false; }
    (roll + distance.modifier_2016() - modifier_total) <= 1
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
            Action::UseSkill { use_skill: false, .. } => {
                self.re_rolled_action = None;
                self.re_roll_source = None;
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_rolled_action = None;
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)  => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)=> { self.thrown_player_has_ball = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(player); }
        else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_throw_team_mate() {
        assert_eq!(StepThrowTeamMate::new().id(), StepId::ThrowTeamMate);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
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
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn no_thrower_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        // No acting player → returns next
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_throw_or_fumble_returns_valid_outcome() {
        // Test that with a thrower and pass coord set, the step produces a valid outcome.
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7));
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(42));
        // Valid outcome: next (success with scatter), goto (failure), or cont (reroll prompt)
        assert!(
            matches!(out.action, StepAction::NextStep | StepAction::GotoLabel | StepAction::Continue),
            "throw should produce a valid outcome"
        );
    }

    #[test]
    fn already_rerolled_with_null_source_goes_to_failure() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7));
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail_label".into();
        step.thrown_player_id = Some("tp1".into());
        // Simulate: already rerolled, no reroll source
        step.re_rolled_action = Some("THROW_TEAM_MATE".into());
        step.re_roll_source = None;

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel, "no reroll source → goto failure");
    }

    #[test]
    fn is_ttm_fumble_roll_1() {
        assert!(is_ttm_fumble(1, PassingDistance::QuickPass, 0));
    }

    #[test]
    fn is_ttm_fumble_roll_6() {
        assert!(!is_ttm_fumble(6, PassingDistance::QuickPass, 0));
    }

    #[test]
    fn is_ttm_fumble_borderline() {
        // roll=2, QuickPass.modifier_2016 = 1, no modifiers → 2+1-0=3 > 1 → not fumble
        assert!(!is_ttm_fumble(2, PassingDistance::QuickPass, 0));
        // roll=2, LongBomb.modifier_2016 = -2, no modifiers → 2+(-2)-0=0 <= 1 → fumble
        assert!(is_ttm_fumble(2, PassingDistance::LongBomb, 0));
    }

    #[test]
    fn is_ttm_fumble_modifier_total_reduces_fumble_chance() {
        // Java: DiceInterpreter.isPassFumble subtracts the modifier total from the
        // roll-check: (roll + distance.modifier2016 - modifierTotal) <= 1.
        // roll=2, QuickPass (+1), modifierTotal=-1 (e.g. "Throw Team-Mate" penalty) →
        // 2 + 1 - (-1) = 4 > 1 → not fumble even though it would fumble with total=2:
        // 2 + 1 - 2 = 1 <= 1 → fumble.
        assert!(!is_ttm_fumble(2, PassingDistance::QuickPass, -1));
        assert!(is_ttm_fumble(2, PassingDistance::QuickPass, 2));
    }

    #[test]
    fn turn_data_pass_used_set_on_throw() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7));
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        step.start(&mut game, &mut GameRng::new(0));

        assert!(game.turn_data_home.pass_used);
        assert!(game.acting_player.has_passed);
    }

    #[test]
    fn throw_team_mate_skill_adds_plus_one_penalty_to_minimum_roll() {
        // Java: ThrowTeamMateBehaviour.handleExecuteStepHook computes
        // `passModifierFactory.findModifiers(new PassContext(game, thrower, distance, true))`
        // which — via GenerifiedModifierFactory's per-skill iteration — always includes the
        // thrower's own `skill.bb2016.ThrowTeamMate` modifier ("Throw Team-Mate", +1, REGULAR,
        // applies when isTtm()). That +1 must raise `minimumRoll` by 1 relative to a thrower
        // without the skill; previously the step hardcoded an empty modifier set so this
        // penalty (and tacklezone/disturbing-presence/other skill modifiers) never applied.
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId as ModelSkillId;

        fn add_player_with_skills(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, skills: Vec<ModelSkillId>) {
            let player = Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
                starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
                extra_skills: vec![], temporary_skills: vec![],
                used_skills: Default::default(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default()
            };
            if home { game.team_home.players.push(player); }
            else { game.team_away.players.push(player); }
            game.field_model.set_player_coordinate(id, coord);
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }

        // delta_x=4, delta_y=0 → bb2016 throwing_range_table row0 col4 = 'S' (ShortPass,
        // modifier_2016 = 0), so the minimum-roll difference below is purely the skill's +1
        // penalty and isn't obscured by the distance modifier or the `max(2, ..)` floor.
        let coord = FieldCoordinate::new(10, 7);
        let pass_coord = FieldCoordinate::new(14, 7);

        let mut game_without_skill = make_game();
        game_without_skill.home_playing = true;
        add_player(&mut game_without_skill, true, "thrower", coord);
        game_without_skill.acting_player.player_id = Some("thrower".into());
        game_without_skill.pass_coordinate = Some(pass_coord);
        let mut step_without_skill = StepThrowTeamMate::new();
        step_without_skill.goto_label_on_failure = "fail".into();
        step_without_skill.start(&mut game_without_skill, &mut GameRng::new(0));

        let mut game_with_skill = make_game();
        game_with_skill.home_playing = true;
        add_player_with_skills(&mut game_with_skill, true, "thrower", coord, vec![ModelSkillId::ThrowTeamMate]);
        game_with_skill.acting_player.player_id = Some("thrower".into());
        game_with_skill.pass_coordinate = Some(pass_coord);
        let mut step_with_skill = StepThrowTeamMate::new();
        step_with_skill.goto_label_on_failure = "fail".into();
        step_with_skill.start(&mut game_with_skill, &mut GameRng::new(0));

        assert_eq!(
            step_with_skill.minimum_roll,
            step_without_skill.minimum_roll + 1,
            "the thrower's own ThrowTeamMate skill must add a +1 minimum-roll penalty"
        );
    }

    #[test]
    fn report_throw_team_mate_roll_added() {
        // Java: ThrowTeamMateBehaviour always calls
        // `step.getResult().addReport(new ReportThrowTeamMateRoll(...))` right after the roll,
        // regardless of success/failure. The Rust translation previously never added this report.
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7));
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        step.thrown_player_id = Some("tp1".into());
        step.start(&mut game, &mut GameRng::new(0));

        assert!(game.report_list.has_report(ReportId::THROW_TEAM_MATE_ROLL),
            "THROW_TEAM_MATE_ROLL report must be added after rolling");
    }

    #[test]
    fn re_rolled_action_cleared_on_use_skill_false() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7));
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        step.re_rolled_action = Some("THROW_TEAM_MATE".into());
        step.re_roll_source = Some("TRR".into());

        // UseSkill false → clear reroll state → no reroll → GOTO_FAILURE
        use ffb_mechanics::skills::SkillId;
        let _ = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::ThrowTeamMate, use_skill: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(step.re_rolled_action.is_none());
    }
}
