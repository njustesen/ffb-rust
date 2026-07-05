/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.ttm.StepThrowTeamMate`.
///
/// Step in the TTM/KTM sequence to perform the actual throw/kick roll. Logic is
/// inlined from `ThrowTeamMateBehaviour.handleExecuteStepHook()` (BB2025 hook).
///
/// BB2025 differences vs BB2020:
///  - Uses `ttm_used` instead of `pass_used` for non-kicked throws.
///  - `evaluatePass`: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE).
///  - Adds Bullseye (canSkipTtmScatterOnSuperbThrow) — headless: auto-decide = false.
///
/// Init param: IS_KICKED_PLAYER (optional).
/// Consumed params: THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL.
use std::collections::HashSet;
use ffb_model::enums::{PassingDistance, PassResult, PlayerState, ReRollSource};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2025::pass_mechanic::PassMechanic as Bb2025PassMechanic;
use ffb_mechanics::bb2025::ttm_mechanic::TtmMechanic as Bb2025TtmMechanic;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2025::scatter_player::{ScatterPlayer, ScatterPlayerParams};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java `StepThrowTeamMate.StepState` — fields promoted to struct level.
pub struct StepThrowTeamMate {
    /// Java: `state.thrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `state.thrownPlayerState`
    pub thrown_player_state: Option<PlayerState>,
    /// Java: `state.thrownPlayerHasBall`
    pub thrown_player_has_ball: bool,
    /// Java: `state.passResult`
    pub pass_result: Option<PassResult>,
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
        let turn_data = if game.home_playing {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };
        if self.kicked {
            turn_data.ktm_used = true;
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

            let modifiers: HashSet<PassModifier> = HashSet::new();
            let ttm_mechanic = Bb2025TtmMechanic::new();
            self.minimum_roll = ttm_mechanic.minimum_roll(passing_distance, &modifiers);
            let modifier_sum = ttm_mechanic.modifier_sum(passing_distance, &modifiers);

            let roll = rng.d6();

            let player_can_pass = game.player(&thrower_id)
                .map(|p| p.passing != 0)
                .unwrap_or(false);
            let passing_value = game.player(&thrower_id)
                .map(|p| p.passing as i32)
                .unwrap_or(0);

            self.pass_result = Some(evaluate_ttm_pass_bb2025(player_can_pass, passing_value, roll, modifier_sum));
            let pass_result = self.pass_result.unwrap();

            // Java: successful = ACCURATE || INACCURATE
            let successful = pass_result == PassResult::Complete || pass_result == PassResult::Inaccurate;

            if successful {
                // Java: if ACCURATE && hasSkillProperty(canSkipTtmScatterOnSuperbThrow) && usingBullseye == null
                //   show dialog → Continue (wait for Bullseye decision)
                // Headless: auto-decide false (don't use Bullseye)
                let has_bullseye = game.player(&thrower_id)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_SKIP_TTM_SCATTER_ON_SUPERB_THROW))
                    .unwrap_or(false);
                if pass_result == PassResult::Complete && has_bullseye && self.using_bullseye.is_none() {
                    // Server-side auto-decide: don't use Bullseye
                    self.using_bullseye = Some(false);
                }

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
                    throw_scatter: pass_result == PassResult::Complete && self.using_bullseye != Some(true),
                    has_swoop: scatters_single,
                    ..Default::default()
                };
                let seq = ScatterPlayer::build_sequence(&scatter_params);
                return self.handle_pass_result().push_seq(seq);
            } else {
                if self.re_rolled_action.is_none() && player_can_pass {
                    let is_fumble = pass_result == PassResult::Fumble;
                    if let Some(prompt) = ask_for_reroll_if_available(game, rerolled_action_key, self.minimum_roll, is_fumble) {
                        self.re_rolled_action = Some(rerolled_action_key.into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
                return self.handle_pass_result();
            }
        }

        StepOutcome::next()
    }

    fn handle_pass_result(&self) -> StepOutcome {
        let result = self.pass_result.unwrap_or(PassResult::Fumble);
        StepOutcome::next().publish(StepParameter::PassResultParam(result))
    }
}

/// Java: ThrowTeamMateBehaviour.evaluatePass (BB2025 version).
/// BB2025 difference: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE).
fn evaluate_ttm_pass_bb2025(player_can_pass: bool, passing_value: i32, roll: i32, modifier_sum: i32) -> PassResult {
    if !player_can_pass || passing_value <= 0 {
        return PassResult::Fumble;
    }
    if roll == 1 {
        return PassResult::Fumble;
    }
    let result_after_modifiers = roll - modifier_sum;
    if roll == 6 || result_after_modifiers >= passing_value {
        PassResult::Complete
    } else if result_after_modifiers <= 1 {
        PassResult::Fumble // BB2025: fumble (not wildly inaccurate)
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
            Action::UseSkill { use_skill, .. } => {
                if !use_skill {
                    self.re_rolled_action = None;
                    self.re_roll_source = None;
                } else {
                    // UseSkill true during Bullseye dialog: set using_bullseye
                    if self.pass_result == Some(PassResult::Complete) && self.using_bullseye.is_none() {
                        self.using_bullseye = Some(true);
                    }
                }
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
            ..Default::default()
        };
        game.team_home.players.push(player);
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
    fn unknown_parameter_rejected() {
        let mut step = StepThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
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
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 1, 0), PassResult::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_roll_6_complete() {
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 6, 0), PassResult::Complete);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_low_result_fumble_not_wildly_inaccurate() {
        // BB2025 specific: resultAfterModifiers <= 1 → FUMBLE (not WILDLY_INACCURATE)
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 3, 2), PassResult::Fumble);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_inaccurate() {
        assert_eq!(evaluate_ttm_pass_bb2025(true, 4, 3, 0), PassResult::Inaccurate);
    }

    #[test]
    fn evaluate_ttm_pass_bb2025_no_passing_stat_fumble() {
        assert_eq!(evaluate_ttm_pass_bb2025(false, 0, 5, 0), PassResult::Fumble);
    }
}
