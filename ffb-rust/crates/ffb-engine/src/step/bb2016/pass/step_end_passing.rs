/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepEndPassing`.
///
/// Final step of the pass sequence.
///
/// Routing logic:
/// - bomb turn → push Bomb generator
/// - animosity re-try → push Pass generator
/// - end turn / end player action → push EndPlayerAction generator
/// - interceptor (dump-off) → set ball coordinate, EndPlayerAction
/// - otherwise continue (move-after-pass handling)
///
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::spp_calc::SppCalc;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2016::bomb::{Bomb, BombParams};
use crate::step::generator::bb2016::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2016::pass::{Pass, PassParams};
use crate::step::util_server_steps::check_touchdown;

/// Java: `StepEndPassing` (bb2016/pass).
pub struct StepEndPassing {
    /// Java: `fInterceptorId`
    pub interceptor_id: Option<String>,
    /// Java: `fCatcherId`
    pub catcher_id: Option<String>,
    /// Java: `fPassAccurate`
    pub pass_accurate: bool,
    /// Java: `fPassFumble`
    pub pass_fumble: bool,
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
    /// Java: `fBombOutOfBounds`
    pub bomb_out_of_bounds: bool,
    /// Java: `dontDropFumble`
    pub dont_drop_fumble: bool,
}

impl StepEndPassing {
    pub fn new() -> Self {
        Self {
            interceptor_id: None,
            catcher_id: None,
            pass_accurate: false,
            pass_fumble: false,
            end_turn: false,
            end_player_action: false,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        game.field_model.range_ruler = None;
        game.field_model.out_of_bounds = false;

        let is_bomb = game.turn_mode.is_bomb_turn();

        // Java path 1: bomb turn → Bomb sequence
        if is_bomb {
            let seq = Bomb::build_sequence(&BombParams {
                catcher_id: self.catcher_id.clone(),
                pass_fumble: self.pass_fumble,
                allow_move_after_pass: false,
                dont_drop_fumble: self.dont_drop_fumble,
            });
            let mut outcome = StepOutcome::next().push_seq(seq);
            if self.bomb_out_of_bounds {
                outcome = outcome.publish(StepParameter::BombOutOfBounds(true));
            }
            return outcome;
        }

        // Java path 2: animosity re-try → Pass generator
        if game.acting_player.suffering_animosity
            && !self.end_player_action
            && game.pass_coordinate.is_none()
        {
            let seq = Pass::build_sequence(&PassParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java: completions SPP and passing yards — accurate, non-intercepted pass.
        // BB2016 has no prayer system, so no additional_spp check.
        if self.pass_accurate && !self.pass_fumble && self.interceptor_id.is_none() {
            if let Some(ref thrower_id) = game.thrower_id.clone() {
                let is_home = game.team_home.has_player(thrower_id);
                let team_result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
                let pr = team_result.player_results.entry(thrower_id.clone()).or_default();
                pr.completions += 1;
                pr.spp_gained += SppCalc::completion_spp();
                // Java: deltaX = endCoord.x - startCoord.x (home) or reversed (away)
                if let (Some(thrower_coord), Some(end_coord)) = (
                    game.field_model.player_coordinate(thrower_id),
                    game.pass_coordinate,
                ) {
                    let delta_x = if game.home_playing {
                        end_coord.x - thrower_coord.x
                    } else {
                        thrower_coord.x - end_coord.x
                    };
                    pr.passing += delta_x;
                }
            }
        }

        // Java path 3: determine end_turn
        let thrower_is_acting = game.thrower_id.is_some()
            && game.acting_player.player_id.is_some()
            && game.thrower_id == game.acting_player.player_id;

        if self.end_turn || self.end_player_action {
            let no_suffering = !game.acting_player.suffering_animosity
                && !game.acting_player.suffering_blood_lust;
            self.end_turn |= check_touchdown(game)
                || (self.catcher_id.is_none() && no_suffering && game.acting_player.has_passed)
                || self.pass_fumble;
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java path 4: interceptor / dump-off path (thrower is NOT the acting player)
        if !thrower_is_acting {
            if let Some(ref interceptor_id) = self.interceptor_id.clone() {
                if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
                    game.field_model.ball_coordinate = Some(coord);
                    game.field_model.ball_moving = false;
                }
            }
            game.defender_action = None;
            return StepOutcome::next();
        }

        // Java path 5: continue (move-after-pass)
        StepOutcome::next()
    }
}

impl Default for StepEndPassing {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndPassing {
    fn id(&self) -> StepId { StepId::EndPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v)       => { self.catcher_id = v.clone(); true }
            StepParameter::InterceptorId(v)   => { self.interceptor_id = v.clone(); true }
            StepParameter::PassAccurate(v)    => { self.pass_accurate = *v; true }
            StepParameter::PassFumble(v)      => { self.pass_fumble = *v; true }
            StepParameter::EndTurn(v)         => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::BombOutOfBounds(v) => { self.bomb_out_of_bounds = *v; true }
            StepParameter::DontDropFumble(v)  => { self.dont_drop_fumble = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_end_passing() {
        assert_eq!(StepEndPassing::new().id(), StepId::EndPassing);
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id, Some("p1".into()));
    }

    #[test]
    fn set_parameter_pass_accurate() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::PassAccurate(true)));
        assert!(step.pass_accurate);
    }

    #[test]
    fn set_parameter_interceptor_id_none() {
        let mut step = StepEndPassing::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(None)));
        assert!(step.interceptor_id.is_none());
    }

    #[test]
    fn clears_range_ruler_and_out_of_bounds() {
        let mut game = make_game();
        game.field_model.out_of_bounds = true;
        let mut step = StepEndPassing::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.field_model.out_of_bounds);
        assert!(game.field_model.range_ruler.is_none());
    }

    #[test]
    fn bomb_turn_pushes_bomb_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndPassing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn animosity_retry_pushes_pass_sequence() {
        let mut game = make_game();
        game.acting_player.suffering_animosity = true;
        let mut step = StepEndPassing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn end_player_action_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn interceptor_sets_ball_coordinate() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("int1", FieldCoordinate::new(10, 7));
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some("int1".into());
        // thrower_id == None → thrower_is_acting = false → interceptor path
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(10, 7)));
    }

    #[test]
    fn end_turn_path_uses_suffering_flags() {
        let mut game = make_game();
        game.acting_player.suffering_animosity = false;
        game.acting_player.suffering_blood_lust = false;
        game.acting_player.has_passed = true;
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        // No catcher → should set end_turn
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty());
    }

    fn make_game_with_thrower(thrower_id: &str) -> Game {
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: thrower_id.into(), nr: 1, name: thrower_id.into(),
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.thrower_id = Some(thrower_id.into());
        game.acting_player.player_id = Some(thrower_id.into());
        game
    }

    #[test]
    fn accurate_pass_awards_completion_spp() {
        let mut game = make_game_with_thrower("thrower1");
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        step.interceptor_id = None;
        step.pass_fumble = false;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("thrower1").unwrap();
        assert_eq!(pr.completions, 1);
        assert_eq!(pr.spp_gained, 1);
    }

    #[test]
    fn fumble_does_not_award_completion_spp() {
        let mut game = make_game_with_thrower("thrower1");
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        step.pass_fumble = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.player_results.get("thrower1").map(|pr| pr.completions).unwrap_or(0) == 0);
    }

    #[test]
    fn intercepted_pass_does_not_award_completion_spp() {
        let mut game = make_game_with_thrower("thrower1");
        game.field_model.set_player_coordinate("int1", FieldCoordinate::new(10, 7));
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        step.interceptor_id = Some("int1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.player_results.get("thrower1").map(|pr| pr.completions).unwrap_or(0) == 0);
    }

    #[test]
    fn inaccurate_pass_does_not_award_spp() {
        let mut game = make_game_with_thrower("thrower1");
        let mut step = StepEndPassing::new();
        step.pass_accurate = false;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.player_results.get("thrower1").map(|pr| pr.spp_gained).unwrap_or(0) == 0);
    }

    #[test]
    fn passing_yards_calculated_on_accurate_pass() {
        let mut game = make_game_with_thrower("thrower1");
        game.field_model.set_player_coordinate("thrower1", FieldCoordinate::new(3, 7));
        game.pass_coordinate = Some(FieldCoordinate::new(10, 7));
        game.home_playing = true;
        let mut step = StepEndPassing::new();
        step.pass_accurate = true;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("thrower1").unwrap();
        assert_eq!(pr.passing, 7); // 10 - 3 = 7 yards
    }
}
