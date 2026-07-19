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
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
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

        // Java: failed confusion roll on throw bomb -> end player action
        if self.end_player_action
            && matches!(
                game.acting_player.player_action,
                Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
            )
        {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: throw bomb mode -> start bomb sequence
        if game.turn_mode.is_bomb_turn() {
            let catcher_for_bomb = if self.interceptor_id.is_some() {
                self.interceptor_id.clone()
            } else {
                self.catcher_id.clone()
            };
            let seq = Bomb::build_sequence(&BombParams {
                catcher_id: catcher_for_bomb,
                pass_fumble: self.pass_fumble,
                allow_move_after_pass: false,
                dont_drop_fumble: false,
            });
            let mut outcome = StepOutcome::next().push_seq(seq);
            if self.bomb_out_of_bounds {
                outcome = outcome.publish(StepParameter::BombOutOfBounds(true));
            }
            return outcome;
        }

        // Java: failed animosity may try to choose a new target
        if game.acting_player.suffering_animosity
            && !self.end_player_action
            && game.pass_coordinate.is_none()
        {
            let seq = Pass::build_sequence(&PassParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        let mut catcher_id = self.catcher_id.clone();

        // Java: completions and passing statistic
        if let Some(ref thrower_id) = game.thrower_id.clone() {
            let has_ball = catcher_id
                .as_deref()
                .map(|id| UtilPlayer::has_ball(game, id))
                .unwrap_or(false);
            let on_thrower_team = catcher_id
                .as_deref()
                .map(|id| {
                    if game.team_home.has_player(thrower_id) {
                        game.team_home.has_player(id)
                    } else {
                        game.team_away.has_player(id)
                    }
                })
                .unwrap_or(false);
            let coord_matches = catcher_id
                .as_deref()
                .and_then(|id| game.field_model.player_coordinate(id))
                .zip(game.pass_coordinate)
                .map(|(catcher_coord, pass_coord)| catcher_coord == pass_coord)
                .unwrap_or(false);

            if has_ball && on_thrower_team && coord_matches {
                let start_coord = game.field_model.player_coordinate(thrower_id);
                let end_coord = catcher_id
                    .as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id));
                let is_home = game.team_home.has_player(thrower_id);
                let team_result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
                let pr = team_result.player_results.entry(thrower_id.clone()).or_default();
                if self.pass_accurate {
                    pr.completions += 1;
                }
                if let (Some(start), Some(end)) = (start_coord, end_coord) {
                    let delta_x = if game.home_playing {
                        end.x - start.x
                    } else {
                        start.x - end.x
                    };
                    pr.passing += delta_x;
                }
            }
        }

        // Java: fEndTurn || fEndPlayerAction || (thrower==actingPlayer && sufferingBloodLust && !hasFed)
        let thrower_is_acting_player = game.thrower_id.is_some()
            && game.thrower_id == game.acting_player.player_id;
        let blood_lust_forced_end = thrower_is_acting_player
            && game.acting_player.suffering_blood_lust
            && !game.acting_player.has_fed;

        if self.end_turn || self.end_player_action || blood_lust_forced_end {
            let other_team_has_catcher = catcher_id
                .as_deref()
                .map(|id| {
                    game.thrower_id
                        .as_deref()
                        .map(|thrower_id| UtilPlayer::find_other_team(game, thrower_id).has_player(id))
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            self.end_turn |= check_touchdown(game)
                || (catcher_id.is_none() && !game.acting_player.suffering_animosity)
                || other_team_has_catcher
                || self.pass_fumble;
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: self.end_player_action,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java else branch
        if let Some(ref interceptor_id) = self.interceptor_id.clone() {
            catcher_id = Some(interceptor_id.clone());
            let is_home = game.team_home.has_player(interceptor_id);
            let team_result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
            let cr = team_result.player_results.entry(interceptor_id.clone()).or_default();
            cr.interceptions += 1;
            if let Some(coord) = game.field_model.player_coordinate(interceptor_id) {
                game.field_model.ball_coordinate = Some(coord);
            }
            game.field_model.ball_moving = false;
        } else {
            catcher_id = game
                .field_model
                .ball_coordinate
                .and_then(|coord| game.field_model.player_at(coord))
                .cloned();
        }

        if thrower_is_acting_player {
            let other_team_has_catcher = catcher_id
                .as_deref()
                .map(|id| {
                    game.thrower_id
                        .as_deref()
                        .map(|thrower_id| UtilPlayer::find_other_team(game, thrower_id).has_player(id))
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            self.end_turn |= check_touchdown(game)
                || catcher_id.is_none()
                || other_team_has_catcher
                || (self.pass_fumble && !self.dont_drop_fumble);
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            return StepOutcome::next().push_seq(seq);
        }

        game.defender_action = None;
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

    fn push_home_player(game: &mut Game, id: &str) {
        game.team_home.players.push(ffb_model::model::player::Player {
            id: id.into(), nr: 1, name: id.into(),
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
    }

    fn make_game_with_thrower(thrower_id: &str) -> Game {
        let mut game = make_game();
        push_home_player(&mut game, thrower_id);
        game.thrower_id = Some(thrower_id.into());
        game.acting_player.player_id = Some(thrower_id.into());
        game
    }

    /// Sets up thrower + catcher (both on home team), with the ball on the catcher
    /// at `catch_coord`, which is also the pass coordinate — matching Java's
    /// `UtilPlayer.hasBall(...) && thrower.getTeam().hasPlayer(catcher) &&
    /// catcherCoordinate.equals(passCoordinate)` gate for the completions/passing block.
    fn make_game_with_thrower_and_catcher(
        thrower_id: &str,
        catcher_id: &str,
        thrower_coord: FieldCoordinate,
        catch_coord: FieldCoordinate,
    ) -> Game {
        let mut game = make_game_with_thrower(thrower_id);
        push_home_player(&mut game, catcher_id);
        game.field_model.set_player_coordinate(thrower_id, thrower_coord);
        game.field_model.set_player_coordinate(catcher_id, catch_coord);
        game.field_model.ball_coordinate = Some(catch_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;
        game.pass_coordinate = Some(catch_coord);
        game
    }

    #[test]
    fn accurate_pass_awards_completion_when_catcher_has_ball_at_pass_coordinate() {
        let mut game = make_game_with_thrower_and_catcher(
            "thrower1", "catcher1", FieldCoordinate::new(3, 7), FieldCoordinate::new(10, 7),
        );
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.pass_accurate = true;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("thrower1").unwrap();
        assert_eq!(pr.completions, 1);
    }

    #[test]
    fn inaccurate_pass_does_not_award_completion() {
        let mut game = make_game_with_thrower_and_catcher(
            "thrower1", "catcher1", FieldCoordinate::new(3, 7), FieldCoordinate::new(10, 7),
        );
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.pass_accurate = false;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(
            game.game_result.home.player_results.get("thrower1").map(|pr| pr.completions).unwrap_or(0),
            0
        );
    }

    #[test]
    fn no_completion_when_catcher_not_at_pass_coordinate() {
        let mut game = make_game_with_thrower("thrower1");
        push_home_player(&mut game, "catcher1");
        game.field_model.set_player_coordinate("catcher1", FieldCoordinate::new(5, 5));
        game.pass_coordinate = Some(FieldCoordinate::new(10, 7)); // does not match catcher's coord
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.pass_accurate = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(
            game.game_result.home.player_results.get("thrower1").map(|pr| pr.completions).unwrap_or(0),
            0
        );
    }

    /// Java's completions/passing-yards block only gates `completions` on
    /// `fPassAccurate` — passing yards are added whenever the catcher has the ball
    /// at the pass coordinate, regardless of accuracy. A prior Rust translation
    /// gated passing yards on `pass_accurate` too; this pins the Java behavior.
    #[test]
    fn passing_yards_awarded_even_when_pass_not_accurate() {
        let mut game = make_game_with_thrower_and_catcher(
            "thrower1", "catcher1", FieldCoordinate::new(3, 7), FieldCoordinate::new(10, 7),
        );
        game.home_playing = true;
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.pass_accurate = false;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("thrower1").unwrap();
        assert_eq!(pr.passing, 7); // 10 - 3 = 7 yards
        assert_eq!(pr.completions, 0);
    }

    #[test]
    fn passing_yards_calculated_on_accurate_pass() {
        let mut game = make_game_with_thrower_and_catcher(
            "thrower1", "catcher1", FieldCoordinate::new(3, 7), FieldCoordinate::new(10, 7),
        );
        game.home_playing = true;
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.pass_accurate = true;
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.home.player_results.get("thrower1").unwrap();
        assert_eq!(pr.passing, 7); // 10 - 3 = 7 yards
    }

    /// Java: `catcherResult.setInterceptions(catcherResult.getInterceptions() + 1)`
    /// in the interceptor branch — the prior Rust translation set the ball
    /// coordinate but never recorded the interception statistic.
    #[test]
    fn interceptor_increments_interception_count() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("int1", FieldCoordinate::new(10, 7));
        let mut step = StepEndPassing::new();
        step.interceptor_id = Some("int1".into());
        step.start(&mut game, &mut GameRng::new(0));
        let pr = game.game_result.away.player_results.get("int1").unwrap();
        assert_eq!(pr.interceptions, 1);
    }

    /// Java's bomb-turn branch prefers `fInterceptorId` over `fCatcherId` when
    /// building the Bomb sequence (`StringTool.isProvided(fInterceptorId) ? ... :
    /// ...`). The prior Rust translation always used `catcher_id`, ignoring any
    /// interceptor.
    #[test]
    fn bomb_turn_prefers_interceptor_id_over_catcher_id() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndPassing::new();
        step.catcher_id = Some("catcher1".into());
        step.interceptor_id = Some("interceptor1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let seq = &out.pushes[0];
        let init_bomb = seq.iter().find(|s| s.step_id == StepId::InitBomb).unwrap();
        assert!(init_bomb.params.iter().any(
            |p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "interceptor1")
        ));
        assert!(!init_bomb.params.iter().any(
            |p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "catcher1")
        ));
    }

    /// Java hardcodes `dontDropFumble=false` in both `Bomb.SequenceParams` calls
    /// in this step; the prior Rust translation passed through `self.dont_drop_fumble`.
    #[test]
    fn bomb_turn_dont_drop_fumble_is_always_false() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepEndPassing::new();
        step.dont_drop_fumble = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        let seq = &out.pushes[0];
        let init_bomb = seq.iter().find(|s| s.step_id == StepId::InitBomb).unwrap();
        assert!(init_bomb.params.iter().any(|p| matches!(p, StepParameter::DontDropFumble(false))));
    }

    /// Java: `if (fEndPlayerAction && (actingPlayer.getPlayerAction() == THROW_BOMB
    /// || actingPlayer.getPlayerAction() == HAIL_MARY_BOMB))` pushes an
    /// EndPlayerAction sequence directly (feedingAllowed=true) before any other
    /// check. This whole branch was missing from the prior Rust translation.
    #[test]
    fn end_player_action_for_bomb_thrower_pushes_end_player_action_directly() {
        let mut game = make_game();
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::ThrowBomb);
        let mut step = StepEndPassing::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let seq = &out.pushes[0];
        assert_eq!(seq[0].step_id, StepId::InitFeeding);
        assert!(seq[0].params.iter().any(|p| matches!(p, StepParameter::FeedingAllowed(true))));
    }
}
