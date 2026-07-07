/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepThrowTeamMate`.
///
/// Step in the TTM/KTM sequence to perform the actual throw/kick roll. Logic is
/// inlined from `ThrowTeamMateBehaviour.handleExecuteStepHook()` (BB2020 hook).
///
/// BB2020 differences vs BB2016:
///  - Adds `kicked` flag (KTM path uses `ktm_used` + KICK_TEAM_MATE rerolled action).
///  - Evaluates pass result using BB2020 logic (ACCURATE/INACCURATE/FUMBLE/WILDLY_INACCURATE).
///  - Publishes `PassResult` for downstream steps.
///
/// Init param: IS_KICKED_PLAYER (optional).
/// Consumed params: THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL.
use std::collections::HashSet;
use ffb_model::enums::{PassingDistance, PassResult, PlayerState, ReRollSource};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_mechanics::bb2020::pass_mechanic::PassMechanic as Bb2020PassMechanic;
use ffb_mechanics::bb2020::ttm_mechanic::TtmMechanic as Bb2020TtmMechanic;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2020::scatter_player::{ScatterPlayer, ScatterPlayerParams};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java `StepThrowTeamMate.StepState` — fields promoted to struct level.
pub struct StepThrowTeamMate {
    /// Java: `state.thrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `state.thrownPlayerState`
    thrown_player_state: Option<PlayerState>,
    /// Java: `state.thrownPlayerHasBall`
    thrown_player_has_ball: bool,
    /// Java: `state.passResult` — BB2020 addition.
    pass_result: Option<PassResult>,
    /// Java: `state.kicked` — BB2020 addition (IS_KICKED_PLAYER init param).
    kicked: bool,
    /// Java: `fReRolledAction`
    re_rolled_action: Option<String>,
    /// Java: `fReRollSource`
    re_roll_source: Option<String>,
    /// Java: stored minimumRoll for re-roll prompt
    minimum_roll: i32,
}

impl StepThrowTeamMate {
    pub fn new() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
            pass_result: None,
            kicked: false,
            re_rolled_action: None,
            re_roll_source: None,
            minimum_roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.setHasPassed(true); game.setThrowerId(actingPlayer.getPlayerId())
        game.acting_player.has_passed = true;
        game.thrower_id = game.acting_player.player_id.clone();
        game.concession_possible = false;

        // Java: if kicked → ktmUsed; else passUsed
        let rerolled_action_key = if self.kicked { "KICK_TEAM_MATE" } else { "THROW_TEAM_MATE" };
        let turn_data = if game.home_playing {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };
        if self.kicked {
            turn_data.ktm_used = true;
        } else {
            turn_data.pass_used = true;
        }

        // Java: if (rerolledAction == getReRolledAction()) → useReRoll or handlePassResult
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
                return self.handle_pass_result(game);
            }
        }

        if do_roll {
            let thrower_id = match game.acting_player.player_id.clone() {
                Some(id) => id,
                None => return StepOutcome::next(),
            };

            let thrower_coord = game.field_model.player_coordinate(&thrower_id);
            let pass_coord = game.pass_coordinate;

            let pass_mechanic = Bb2020PassMechanic::new();
            let passing_distance = match pass_mechanic.find_passing_distance(game, thrower_coord, pass_coord, true) {
                Some(d) => d,
                None => return StepOutcome::next(),
            };

            let modifiers: HashSet<PassModifier> = HashSet::new();
            let ttm_mechanic = Bb2020TtmMechanic::new();
            self.minimum_roll = ttm_mechanic.minimum_roll(passing_distance, &modifiers);
            let modifier_sum = ttm_mechanic.modifier_sum(passing_distance, &modifiers);

            let roll = rng.d6();

            // Java: playerCanPass = thrower.getPassing() != 0
            // Java: state.passResult = evaluatePass(playerCanPass, thrower.getPassingWithModifiers(), roll, modifierSum)
            let player_can_pass = game.player(&thrower_id)
                .map(|p| p.passing != 0)
                .unwrap_or(false);
            let passing_value = game.player(&thrower_id)
                .map(|p| p.passing as i32)
                .unwrap_or(0);

            self.pass_result = Some(evaluate_ttm_pass(player_can_pass, passing_value, roll, modifier_sum));
            let pass_result = self.pass_result.unwrap();

            // Java: successful = passResult == ACCURATE || passResult == INACCURATE
            let successful = pass_result == PassResult::Complete || pass_result == PassResult::Inaccurate;

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

            if successful {
                // Push ScatterPlayer sequence
                let scatters_single = self.thrown_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION))
                    .unwrap_or(false);

                let scatter_params = ScatterPlayerParams {
                    thrown_player_id: self.thrown_player_id.clone(),
                    thrown_player_state: self.thrown_player_state,
                    thrown_player_has_ball: self.thrown_player_has_ball,
                    thrown_player_coordinate: thrower_coord,
                    throw_scatter: pass_result == PassResult::Complete,
                    has_swoop: scatters_single,
                    deviates: pass_result == PassResult::Inaccurate,
                    ..Default::default()
                };
                let seq = ScatterPlayer::build_sequence(&scatter_params);
                return self.handle_pass_result(game).push_seq(seq);
            } else {
                // Java: if (getReRolledAction() != rerolledAction && playerCanPass) → try reroll
                if self.re_rolled_action.is_none() && player_can_pass {
                    let is_fumble = pass_result == PassResult::Fumble;
                    if let Some(prompt) = ask_for_reroll_if_available(game, rerolled_action_key, self.minimum_roll, is_fumble) {
                        self.re_rolled_action = Some(rerolled_action_key.into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
                return self.handle_pass_result(game);
            }
        }

        StepOutcome::next()
    }

    /// Java: handlePassResult — publish PassResult and set NEXT_STEP.
    fn handle_pass_result(&self, _game: &mut Game) -> StepOutcome {
        let result = self.pass_result.unwrap_or(PassResult::Fumble);
        StepOutcome::next().publish(StepParameter::PassResultParam(result))
    }
}

/// Java: ThrowTeamMateBehaviour.evaluatePass (BB2020 version).
/// Returns FUMBLE/WILDLY_INACCURATE/INACCURATE/ACCURATE based on roll + modifiers.
fn evaluate_ttm_pass(player_can_pass: bool, passing_value: i32, roll: i32, modifier_sum: i32) -> PassResult {
    if !player_can_pass || passing_value <= 0 {
        return PassResult::Fumble;
    }
    if roll == 1 {
        return PassResult::Fumble;
    }
    let result_after_modifiers = roll - modifier_sum;
    if roll == 6 || result_after_modifiers >= passing_value {
        PassResult::Complete    // Java: ACCURATE → maps to Complete in our model
    } else if result_after_modifiers <= 1 {
        PassResult::WildlyInaccurate
    } else {
        PassResult::Inaccurate
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, passing: i32) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
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
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Fumble)));
        assert_eq!(step.pass_result, Some(PassResult::Fumble));
    }

    #[test]
    fn set_parameter_kicked() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.kicked);
    }

    #[test]
    fn unknown_parameter_returns_false() {
        let mut step = StepThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::ThrowScatter(true)));
    }

    #[test]
    fn no_thrower_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn valid_throw_produces_pass_result_param() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(42));
        // Should always publish PassResultParam
        let has_result = out.published.iter().any(|p| matches!(p, StepParameter::PassResultParam(_)));
        let has_sequence = !out.pushes.is_empty();
        // Either a scatter sequence was pushed (success) or pass_result published (failure)
        assert!(has_result || has_sequence, "throw should produce outcome");
    }

    #[test]
    fn already_rerolled_no_source_goes_to_next_via_pass_result() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.re_rolled_action = Some("THROW_TEAM_MATE".into());
        step.re_roll_source = None;

        let out = step.start(&mut game, &mut GameRng::new(0));
        // No reroll source → handlePassResult → publishes PassResultParam, NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kicked_sets_ktm_used() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.kicked = true;

        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.turn_data_home.ktm_used);
    }

    #[test]
    fn evaluate_ttm_pass_roll_1_is_fumble() {
        assert_eq!(evaluate_ttm_pass(true, 4, 1, 0), PassResult::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_roll_6_is_complete() {
        assert_eq!(evaluate_ttm_pass(true, 4, 6, 0), PassResult::Complete);
    }

    #[test]
    fn evaluate_ttm_pass_no_passing_stat_is_fumble() {
        assert_eq!(evaluate_ttm_pass(false, 0, 5, 0), PassResult::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_wildly_inaccurate() {
        // passing=4, roll=3, modifier_sum=2 → 3-2=1 <= 1 → WILDLY_INACCURATE
        assert_eq!(evaluate_ttm_pass(true, 4, 3, 2), PassResult::WildlyInaccurate);
    }

    #[test]
    fn evaluate_ttm_pass_inaccurate() {
        // passing=4, roll=3, modifier_sum=0 → 3 < 4, > 1 → INACCURATE
        assert_eq!(evaluate_ttm_pass(true, 4, 3, 0), PassResult::Inaccurate);
    }

    #[test]
    fn successful_throw_emits_throw_team_mate_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7), 4);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.thrown_player_state = Some(PlayerState::new(PS_STANDING));
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.report_list.has_report(ReportId::THROW_TEAM_MATE_ROLL));
    }

    #[test]
    fn failed_throw_emits_throw_team_mate_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        // passing=0 → evaluate_ttm_pass returns Fumble (player_can_pass=false)
        add_player(&mut game, true, "thrower", FieldCoordinate::new(10, 7), 0);
        game.acting_player.player_id = Some("thrower".into());
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));

        let mut step = StepThrowTeamMate::new();
        step.thrown_player_id = Some("tp1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::THROW_TEAM_MATE_ROLL));
    }
}
