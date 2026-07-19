use ffb_model::enums::Direction;
use ffb_model::enums::SkillId;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_event::ReportEvent;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_kickoff_scatter::ReportKickoffScatter;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Variant of `StepKickoffScatterRoll` that rolls direction + distance **first**,
/// then shows a dialog asking the kicking coach whether to use the Kick skill to
/// halve the scatter distance.  If no Kick skill is present the dialog is skipped
/// and execution proceeds identically to `StepKickoffScatterRoll`.
///
/// Publishes (same contract as `StepKickoffScatterRoll`):
///  - `KickingPlayerCoordinate`
///  - `KickoffBounds`
///  - `Touchback`
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepKickoffScatterRollAskAfter`.
pub struct StepKickoffScatterRollAskAfter {
    /// Java: fKickoffStartCoordinate — set by preceding StepKickoff.
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
    /// Java: fUseKickChoice — None until the coach responds (or skill is absent).
    pub use_kick_choice: Option<bool>,
    /// Java: fScatterDirection — rolled d8.
    pub scatter_direction: Option<Direction>,
    /// Java: fScatterDistance — rolled d6 (raw, before Kick halving).
    pub scatter_distance: i32,
    /// Java: fKickingPlayerCoordinate — published downstream.
    pub kicking_player_coordinate: Option<FieldCoordinate>,
    /// Kicking player id, derived from findKickingPlayer(); used to check for the Kick
    /// skill (canReduceKickDistance) and to report skill use.
    pub kicking_player_id: Option<String>,
    /// Java: fKickoffBounds — receiving half or None → touchback.
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fTouchback.
    pub touchback: bool,
}

impl StepKickoffScatterRollAskAfter {
    pub fn new() -> Self {
        Self {
            kickoff_start_coordinate: None,
            use_kick_choice: None,
            scatter_direction: None,
            scatter_distance: 0,
            kicking_player_coordinate: None,
            kicking_player_id: None,
            kickoff_bounds: None,
            touchback: false,
        }
    }
}

impl Default for StepKickoffScatterRollAskAfter {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffScatterRollAskAfter {
    fn id(&self) -> StepId { StepId::KickoffScatterRollAskAfter }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_USE_SKILL with canReduceKickDistance: capture the choice.
        if let Action::UseSkill { use_skill, .. } = action {
            if self.use_kick_choice.is_none() {
                self.use_kick_choice = Some(*use_skill);
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

impl StepKickoffScatterRollAskAfter {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let start = match self.kickoff_start_coordinate {
            Some(c) => c,
            None => return StepOutcome::cont(),
        };

        // ── Phase 1: roll direction + raw distance (once) ────────────────────
        if self.scatter_direction.is_none() {
            let dir_roll = rng.d8();
            self.scatter_direction = Direction::for_roll(dir_roll);
            self.scatter_distance = rng.d6();

            // Determine kicking player: Java findKickingPlayer() — first filter the
            // kicking team's players in CENTER_FIELD bounds; if none are there, fall
            // back to LOS bounds. Among the resulting candidates, prefer one with the
            // canReduceKickDistance (Kick) skill, else the first candidate.
            let (center_bounds, los_bounds) = if game.home_playing {
                (FieldCoordinateBounds::CENTER_FIELD_HOME, FieldCoordinateBounds::LOS_HOME)
            } else {
                (FieldCoordinateBounds::CENTER_FIELD_AWAY, FieldCoordinateBounds::LOS_AWAY)
            };
            let kicking_team = game.active_team();
            let center_players: Vec<(String, FieldCoordinate)> = kicking_team.players.iter()
                .filter_map(|p| {
                    let coord = game.field_model.player_coordinate(&p.id)?;
                    center_bounds.is_in_bounds(coord).then_some((p.id.clone(), coord))
                })
                .collect();
            let players_on_field = if center_players.is_empty() {
                kicking_team.players.iter()
                    .filter_map(|p| {
                        let coord = game.field_model.player_coordinate(&p.id)?;
                        los_bounds.is_in_bounds(coord).then_some((p.id.clone(), coord))
                    })
                    .collect::<Vec<_>>()
            } else {
                center_players
            };
            let found = players_on_field.iter()
                .find(|(id, _)| {
                    game.player(id)
                        .map(|p| p.has_skill_property(NamedProperties::CAN_REDUCE_KICK_DISTANCE))
                        .unwrap_or(false)
                })
                .or_else(|| players_on_field.first())
                .cloned();

            let default_kicker = if game.home_playing {
                FieldCoordinate::new(0, 7)
            } else {
                FieldCoordinate::new(25, 7)
            };
            match found {
                Some((id, coord)) => {
                    self.kicking_player_id = Some(id);
                    self.kicking_player_coordinate = Some(coord);
                }
                None => {
                    self.kicking_player_id = None;
                    self.kicking_player_coordinate = Some(default_kicker);
                }
            }

            // Java: getResult().addReport(new ReportKickoffScatter(ballCoordinateEnd, fScatterDirection, rollScatterDirection, fScatterDistance))
            let ball_end_report = start.step(self.scatter_direction.unwrap_or(Direction::North), self.scatter_distance);
            game.report_list.add(ReportKickoffScatter::new(
                ball_end_report,
                self.scatter_direction.unwrap_or(Direction::North),
                dir_roll,
                self.scatter_distance,
            ));

            // Java: if kicking player has canReduceKickDistance, show DialogKickSkillParameter
            // and wait for the coach's choice; otherwise fUseKickChoice = false immediately.
            let has_kick = self.kicking_player_id.as_deref()
                .and_then(|pid| game.player(pid))
                .map(|p| p.has_skill_property(NamedProperties::CAN_REDUCE_KICK_DISTANCE))
                .unwrap_or(false);
            if has_kick {
                return StepOutcome::cont();
            }
            self.use_kick_choice = Some(false);
        }

        // ── Phase 2: resolve final distance and place ball ───────────────────
        let direction = match self.scatter_direction {
            Some(d) => d,
            None => Direction::North,
        };
        let use_kick = self.use_kick_choice.unwrap_or(false);
        // Java: Math.ceil(fScatterDistance / 2.0) when Kick is used.
        let distance = if use_kick {
            (self.scatter_distance as f64 / 2.0).ceil() as i32
        } else {
            self.scatter_distance
        };

        let ball_end = start.step(direction, distance);

        // Java: if (fUseKickChoice && skillReduceKickDistance != null) { addReport(ReportSkillUse(...)) }
        if use_kick {
            if let Some(pid) = self.kicking_player_id.clone() {
                let has_kick = game.player(&pid)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_REDUCE_KICK_DISTANCE))
                    .unwrap_or(false);
                if has_kick {
                    game.report_list.add(ReportSkillUse::new(
                        Some(pid),
                        SkillId::Kick,
                        true,
                        SkillUse::HALVE_KICKOFF_SCATTER,
                    ));
                }
            }
        }

        // Walk back until the ball is on the field.
        let mut dist_walk = distance;
        let mut last_valid = ball_end;
        while !FieldCoordinateBounds::FIELD.is_in_bounds(last_valid) {
            dist_walk -= 1;
            last_valid = start.step(direction, dist_walk);
            if dist_walk < 0 {
                last_valid = start;
                break;
            }
        }

        game.field_model.ball_in_play = false;
        game.field_model.ball_coordinate = Some(last_valid);
        game.field_model.ball_moving = true;

        // ── Determine kickoff bounds and touchback ────────────────────────────
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
        // Java: if (fTouchback) getResult().addReport(new ReportEvent("The ball lands out of bounds -> TOUCHBACK!!"))
        if self.touchback {
            game.report_list.add(ReportEvent::new(Some("The ball lands out of bounds -> TOUCHBACK!!".into())));
        }

        // ── Publish ────────────────────────────────────────────────────────────
        let kicking_coord = self.kicking_player_coordinate.unwrap();
        let touchback = self.touchback;
        let bounds = self.kickoff_bounds;

        let mut outcome = StepOutcome::next()
            .publish(StepParameter::KickingPlayerCoordinate(kicking_coord))
            .publish(StepParameter::Touchback(touchback));
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
    fn start_without_start_coordinate_returns_cont() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_with_coordinate_returns_next_step() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_kicking_coord_and_touchback() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickingPlayerCoordinate(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Touchback(_))));
    }

    #[test]
    fn ball_on_field_after_scatter() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(5));
        if let Some(c) = game.field_model.ball_coordinate {
            assert!(FieldCoordinateBounds::FIELD.is_in_bounds(c), "Ball off-field at {c:?}");
        }
    }

    /// Java `executeStep()`: when the kicking player has `canReduceKickDistance` (the
    /// Kick skill) and `fUseKickChoice` is still null, a `DialogKickSkillParameter` is
    /// shown and the step waits for the coach's answer instead of proceeding straight
    /// through with `fUseKickChoice = false`. Before the fix, this Rust step always
    /// hardcoded `use_kick_choice = Some(false)` on the first pass and never waited,
    /// so this test would have failed (asserting NextStep instead of Continue).
    #[test]
    fn kick_skill_player_waits_for_choice() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;

        let mut game = make_game();
        let p = Player {
            id: "kicker".into(), name: "kicker".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Kick, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p);
        // LOS_HOME is x=12, y=4..=10 — within bounds for the home kicking team's search.
        game.field_model.set_player_coordinate("kicker", FieldCoordinate::new(12, 7));
        game.field_model.set_player_state("kicker", PlayerState::new(PS_STANDING));
        game.home_playing = true;

        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "player with Kick skill should wait for choice");
        assert_eq!(step.kicking_player_id.as_deref(), Some("kicker"));
    }

    #[test]
    fn use_skill_action_sets_kick_choice() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        // Roll first (sets scatter_direction) but not use_kick_choice yet.
        // We can simulate by setting direction manually.
        step.scatter_direction = Some(Direction::East);
        step.scatter_distance = 4;
        step.kicking_player_coordinate = Some(FieldCoordinate::new(0, 7));
        let action = Action::UseSkill { skill_id: ffb_mechanics::skills::SkillId::Kick, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // With use_kick=true and distance=4, effective distance = ceil(4/2) = 2.
        // Verify the step finished (returns NextStep).
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn report_kickoff_scatter_added_on_start() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::KICKOFF_SCATTER));
    }

    #[test]
    fn no_kickoff_scatter_report_without_start_coordinate() {
        let mut game = make_game();
        let mut step = StepKickoffScatterRollAskAfter::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::KICKOFF_SCATTER));
    }
}
