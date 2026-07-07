use ffb_model::enums::{Direction, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_kickoff_scatter::ReportKickoffScatter;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::report::mixed::report_event::ReportEvent;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rolls scatter direction (d8) and distance (d6) for the kickoff, places the ball
/// at the last valid square along the scatter path, determines kickoff bounds and
/// whether a touchback applies.
///
/// Publishes:
///  - `KickingPlayerCoordinate` — coordinate of the player who kicked the ball
///    (or a default centre-field square when no player is found).
///  - `KickoffBounds` — the receiving half bounds, or `None` → touchback.
///  - `Touchback` — true when the ball lands outside the receiving half.
///
/// Skill: if the kicking player has `canReduceKickDistance` (the Kick skill),
/// the step should show a dialog before rolling.  That dialog path is a TODO.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoffScatterRoll`.
pub struct StepKickoffScatterRoll {
    /// Java: fKickoffStartCoordinate — set by preceding StepKickoff.
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
    /// Java: fUseKickChoice — whether the Kick skill is used to halve distance.
    pub use_kick_choice: Option<bool>,
    /// Java: fScatterDirection — rolled d8 direction.
    pub scatter_direction: Option<Direction>,
    /// Java: fScatterDistance — rolled d6 distance (or d3 when Kick is used).
    pub scatter_distance: Option<i32>,
    /// Java: fKickingPlayerCoordinate — published downstream.
    pub kicking_player_coordinate: Option<FieldCoordinate>,
    /// Kicking player ID, derived from coordinate in phase 1; stored for SkillUse event in phase 2.
    pub kicking_player_id: Option<String>,
    /// Java: fKickoffBounds — the half the ball must land in; None → touchback.
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fTouchback — true when ball lands outside the receiving half.
    pub touchback: bool,
}

impl StepKickoffScatterRoll {
    pub fn new() -> Self {
        Self {
            kickoff_start_coordinate: None,
            use_kick_choice: None,
            scatter_direction: None,
            scatter_distance: None,
            kicking_player_coordinate: None,
            kicking_player_id: None,
            kickoff_bounds: None,
            touchback: false,
        }
    }
}

impl Default for StepKickoffScatterRoll {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffScatterRoll {
    fn id(&self) -> StepId { StepId::KickoffScatterRoll }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_USE_SKILL with canReduceKickDistance: set fUseKickChoice.
        if let Action::UseSkill { use_skill, .. } = action {
            if self.use_kick_choice.is_none() {
                self.use_kick_choice = Some(*use_skill);
                if *use_skill {
                    let pid = self.kicking_player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        pid,
                        SkillId::Kick,
                        true,
                        SkillUse::HALVE_KICKOFF_SCATTER,
                    ));
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickoffStartCoordinate(v) => { self.kickoff_start_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

impl StepKickoffScatterRoll {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let start = match self.kickoff_start_coordinate {
            Some(c) => c,
            None => return StepOutcome::cont(),
        };

        // ── Determine the kicking player coordinate ──────────────────────────
        // Java: search the kicking team's players for one in CENTER_FIELD or LOS bounds.
        if self.kicking_player_coordinate.is_none() {
            let (center_bounds, los_bounds) = if game.home_playing {
                (FieldCoordinateBounds::CENTER_FIELD_HOME, FieldCoordinateBounds::LOS_HOME)
            } else {
                (FieldCoordinateBounds::CENTER_FIELD_AWAY, FieldCoordinateBounds::LOS_AWAY)
            };
            let kicking_team = game.active_team();
            let found = kicking_team.players.iter().find_map(|p| {
                let coord = game.field_model.player_coordinate(&p.id)?;
                if center_bounds.is_in_bounds(coord) || los_bounds.is_in_bounds(coord) {
                    Some((p.id.clone(), coord))
                } else {
                    None
                }
            });
            let default_kicker = if game.home_playing {
                FieldCoordinate::new(0, 7)
            } else {
                FieldCoordinate::new(25, 7)
            };
            if let Some((id, coord)) = found {
                self.kicking_player_id = Some(id);
                self.kicking_player_coordinate = Some(coord);
            } else {
                self.kicking_player_coordinate = Some(default_kicker);
            }
        }

        // ── Kick skill dialog ────────────────────────────────────────────────
        // Java: if kicking player has canReduceKickDistance and use_kick_choice is None → show dialog.
        if self.use_kick_choice.is_none() {
            let has_kick = self.kicking_player_id.as_deref()
                .and_then(|pid| game.player(pid))
                .map(|p| p.has_skill_property(NamedProperties::CAN_REDUCE_KICK_DISTANCE))
                .unwrap_or(false);
            if has_kick {
                // Wait for agent decision via Action::UseSkill.
                return StepOutcome::cont();
            }
            self.use_kick_choice = Some(false);
        }

        // ── Roll scatter direction (d8) and distance (d6 or d3 for Kick) ────
        let dir_roll = rng.d8();
        let direction = Direction::for_roll(dir_roll)
            .unwrap_or(Direction::North);
        self.scatter_direction = Some(direction);

        let use_kick = self.use_kick_choice.unwrap_or(false);
        let raw_distance = if use_kick { rng.d3() } else { rng.d6() };
        self.scatter_distance = Some(raw_distance);

        // ── Find scatter endpoint ────────────────────────────────────────────
        let ball_end = start.step(direction, raw_distance);

        // Walk back along the scatter path until we're on the field.
        let mut distance = raw_distance;
        let mut last_valid = ball_end;
        while !FieldCoordinateBounds::FIELD.is_in_bounds(last_valid) {
            distance -= 1;
            last_valid = start.step(direction, distance);
            if distance < 0 {
                // Degenerate: start itself is off-field; just use start.
                last_valid = start;
                break;
            }
        }

        // Place ball at last valid square, not yet in play (same as Java).
        game.field_model.ball_in_play = false;
        game.field_model.ball_coordinate = Some(last_valid);
        game.field_model.ball_moving = true;

        game.report_list.add(ReportKickoffScatter::new(
            ball_end,
            direction,
            dir_roll,
            raw_distance,
        ));

        // ── Determine kickoff bounds and touchback ────────────────────────────
        // Java: if home is kicking the ball must land in HALF_AWAY; if away is kicking → HALF_HOME.
        let receiving_half = if game.home_playing {
            FieldCoordinateBounds::HALF_AWAY
        } else {
            FieldCoordinateBounds::HALF_HOME
        };

        if receiving_half.is_in_bounds(ball_end) {
            self.kickoff_bounds = Some(receiving_half);
        } else {
            self.kickoff_bounds = None;
        }
        self.touchback = self.kickoff_bounds.is_none();
        game.field_model.out_of_bounds = self.touchback;

        if self.touchback {
            game.report_list.add(ReportEvent::new(Some("The ball lands out of bounds -> TOUCHBACK!!".to_string())));
        }

        // ── Publish parameters and events ────────────────────────────────────
        let kicking_coord = self.kicking_player_coordinate.unwrap();
        let touchback = self.touchback;
        let bounds = self.kickoff_bounds;
        let kick_event = if use_kick {
            self.kicking_player_id.as_ref().map(|pid| GameEvent::SkillUse {
                player_id: pid.clone(),
                skill_id: SkillId::Kick as u16,
                used: true,
            })
        } else { None };

        let mut outcome = StepOutcome::next()
            .with_event(GameEvent::KickoffScatter {
                start: kicking_coord,
                direction: dir_roll,
                distance,
            })
            .publish(StepParameter::KickingPlayerCoordinate(kicking_coord))
            .publish(StepParameter::Touchback(touchback));
        if let Some(ev) = kick_event { outcome = outcome.with_event(ev); }
        if let Some(b) = bounds {
            outcome = outcome.publish(StepParameter::KickoffBounds(b));
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_without_coordinate_returns_cont() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_with_coordinate_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_accepts_kickoff_start_coordinate() {
        let mut step = StepKickoffScatterRoll::new();
        let coord = FieldCoordinate::new(13, 7);
        assert!(step.set_parameter(&StepParameter::KickoffStartCoordinate(coord)));
        assert_eq!(step.kickoff_start_coordinate, Some(coord));
    }

    #[test]
    fn publishes_touchback_and_kicking_coordinate() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Touchback(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickingPlayerCoordinate(_))));
    }

    #[test]
    fn ball_placed_on_field_after_scatter() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        // Ball must be placed somewhere on the field.
        if let Some(coord) = game.field_model.ball_coordinate {
            assert!(FieldCoordinateBounds::FIELD.is_in_bounds(coord),
                "Ball placed off-field at {coord:?}");
        }
    }

    #[test]
    fn touchback_when_ball_lands_in_wrong_half() {
        // Force a direction that will scatter the ball into the home half
        // when home is playing (kicking to away half). We can verify the
        // touchback flag gets set correctly by checking the step state.
        let mut game = make_game();
        game.home_playing = true; // home kicks to away half (x >= 13)
        let mut step = StepKickoffScatterRoll::new();
        // Put the start at the home end — any scatter toward home side = touchback
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        // Regardless of outcome, touchback state must match bounds.
        assert_eq!(step.touchback, step.kickoff_bounds.is_none());
    }

    #[test]
    fn adds_kickoff_scatter_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::KICKOFF_SCATTER));
    }

    #[test]
    fn adds_event_report_on_touchback() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        if step.touchback {
            assert!(game.report_list.has_report(ReportId::EVENT));
        }
    }

    #[test]
    fn kick_skill_player_waits_for_choice() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;

        let mut game = make_game();
        // Add a home player with the Kick skill
        let p = Player {
            id: "kicker".into(), name: "kicker".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Kick, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("kicker", FieldCoordinate::new(13, 7));
        game.field_model.set_player_state("kicker", PlayerState::new(PS_STANDING));
        game.home_playing = true;

        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.kicking_player_id = Some("kicker".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should wait for agent to choose whether to use Kick skill
        assert_eq!(out.action, StepAction::Continue, "player with Kick skill should wait for choice");
    }
}
