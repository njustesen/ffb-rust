use ffb_model::enums::{Direction, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_event::ReportEvent;
use crate::action::Action;
use crate::mechanic::mixed::state_mechanic::StateMechanic;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepKickoffScatterRoll` (BB2020).
///
/// Differs from BB2025 variant:
/// - Kick skill halves `fScatterDistance` (integer division) rather than re-rolling as d3.
/// - `findKickingPlayer` prefers players in CENTER_FIELD bounds, falls back to LOS.
///   Within each group, prefers a player with `canReduceKickDistance` (Kick skill).
///
/// Expects stepParameter KICKOFF_START_COORDINATE to be set by a preceding step.
/// Sets stepParameter KICKING_PLAYER_COORDINATE for all steps on the stack.
/// Sets stepParameter KICKOFF_BOUNDS for all steps on the stack.
/// Sets stepParameter TOUCHBACK for all steps on the stack.
pub struct StepKickoffScatterRoll {
    /// Java: fKickoffStartCoordinate
    pub kickoff_start_coordinate: Option<FieldCoordinate>,
    /// Java: fUseKickChoice — None = not yet answered (dialog pending)
    pub use_kick_choice: Option<bool>,
    /// Java: fScatterDirection
    pub scatter_direction: Option<Direction>,
    /// Java: fScatterDistance
    pub scatter_distance: i32,
    /// Java: fKickingPlayerCoordinate
    pub kicking_player_coordinate: Option<FieldCoordinate>,
    /// Java: fKickoffBounds
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fTouchback
    pub touchback: bool,
    /// Raw D8 roll for direction (for event reporting)
    pub scatter_direction_roll: i32,
    /// Kicking player id (stored from phase 1 for SkillUse event in phase 2)
    pub kicking_player_id: Option<String>,
}

impl StepKickoffScatterRoll {
    pub fn new() -> Self {
        Self {
            kickoff_start_coordinate: None,
            use_kick_choice: None,
            scatter_direction: None,
            scatter_distance: 0,
            kicking_player_coordinate: None,
            kickoff_bounds: None,
            touchback: false,
            scatter_direction_roll: 0,
            kicking_player_id: None,
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
        // Java: CLIENT_USE_SKILL with canReduceKickDistance → sets fUseKickChoice
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

impl StepKickoffScatterRoll {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let start = match self.kickoff_start_coordinate {
            Some(c) => c,
            None => return StepOutcome::cont(),
        };

        // Phase 1: roll direction + distance, find kicking player, show dialog or auto-resolve
        if self.scatter_direction.is_none() {
            let dir_roll = rng.d8();
            let direction = Direction::for_roll(dir_roll).unwrap_or(Direction::North);
            let distance = rng.d6();

            self.scatter_direction = Some(direction);
            self.scatter_distance = distance;
            self.scatter_direction_roll = dir_roll;

            // Find kicking player (prefers CENTER_FIELD, falls back to LOS)
            let kicking_player_id = Self::find_kicking_player_id(game);
            self.kicking_player_id = kicking_player_id.clone();

            self.kicking_player_coordinate = match kicking_player_id.as_deref() {
                Some(id) => game.field_model.player_coordinate(id),
                None => None,
            };

            // Default coordinate when no kicking player found
            if self.kicking_player_coordinate.is_none() {
                self.kicking_player_coordinate = Some(if game.home_playing {
                    FieldCoordinate::new(0, 7)
                } else {
                    FieldCoordinate::new(25, 7)
                });
            }

            // Check if the kicking player has the Kick skill (canReduceKickDistance)
            let has_kick_skill = kicking_player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.all_skill_ids()
                    .any(|sk| sk == SkillId::Kick))
                .unwrap_or(false);

            if has_kick_skill {
                // Java: show DialogKickSkillParameter; wait for client answer.
                // client-only: DialogKickSkillParameter — headless auto-declines Kick skill
                self.use_kick_choice = Some(false);
            } else {
                self.use_kick_choice = Some(false);
            }
        }

        // Phase 2: apply scatter using (possibly halved) distance
        if let (Some(direction), Some(use_kick)) = (self.scatter_direction, self.use_kick_choice) {
            // BB2020: halve scatter distance (integer div) vs BB2025 which re-rolls d3
            let distance = if use_kick { self.scatter_distance / 2 } else { self.scatter_distance };

            let ball_end = start.step(direction, distance);

            // Walk back along the scatter path until we're on the field
            let mut d = distance;
            let mut last_valid = ball_end;
            while !FieldCoordinateBounds::FIELD.is_in_bounds(last_valid) {
                d -= 1;
                if d < 0 {
                    last_valid = start;
                    break;
                }
                last_valid = start.step(direction, d);
            }

            game.field_model.ball_in_play = false;
            game.field_model.ball_coordinate = Some(last_valid);
            game.field_model.ball_moving = true;

            // Determine kickoff bounds (the receiving half)
            let receiving_half = if game.home_playing {
                FieldCoordinateBounds::HALF_AWAY
            } else {
                FieldCoordinateBounds::HALF_HOME
            };

            self.kickoff_bounds = if receiving_half.is_in_bounds(ball_end) {
                Some(receiving_half)
            } else {
                None
            };
            self.touchback = self.kickoff_bounds.is_none();

            if self.touchback {
                game.field_model.out_of_bounds = true;
                game.report_list.add(ReportEvent::new(Some("The ball lands out of bounds -> TOUCHBACK!!".into())));
            }

            // Java: addReport(new ReportSkillUse(kick, true, SkillUse.HALVE_KICKOFF_SCATTER))
            let kick_event = if use_kick {
                self.kicking_player_id.as_ref().map(|pid| GameEvent::SkillUse {
                    player_id: pid.clone(),
                    skill_id: SkillId::Kick as u16,
                    used: true,
                })
            } else { None };

            // Java: if (game.getHalf() < 3 && turnDataHome.getTurnNr() == 0 && turnDataAway.getTurnNr() == 0)
            //   → UtilServerGame.handleChefRolls — fires exactly once, at the very first kickoff of the half
            //   (turnNr is reset to 0 for both teams at the start of each half and only increments on EndTurn).
            let first_kickoff_of_half = game.turn_data_home.turn_nr == 0 && game.turn_data_away.turn_nr == 0;
            if game.half < 3 && first_kickoff_of_half {
                let _chef_events = StateMechanic::new().handle_chef_rolls(game, rng);
                // Chef GameEvents are handled downstream; no report needed here.
            }

            let kicking_coord = self.kicking_player_coordinate.unwrap();
            let touchback = self.touchback;

            // Java: addReport(new ReportKickoffScatter(direction, distance))
            let mut outcome = StepOutcome::next()
                .with_event(GameEvent::KickoffScatter {
                    start: kicking_coord,
                    direction: self.scatter_direction_roll,
                    distance,
                })
                .publish(StepParameter::KickingPlayerCoordinate(kicking_coord))
                .publish(StepParameter::Touchback(touchback));
            if let Some(b) = self.kickoff_bounds {
                outcome = outcome.publish(StepParameter::KickoffBounds(b));
            }
            if let Some(ev) = kick_event {
                outcome = outcome.with_event(ev);
            }
            return outcome;
        }

        // Waiting for skill dialog
        StepOutcome::cont()
    }

    /// Java: `findKickingPlayer` — returns the id of the first player in the kicking team's
    /// CENTER_FIELD bounds (or LOS if none), preferring players with the Kick skill.
    fn find_kicking_player_id(game: &Game) -> Option<String> {
        let kicking_team = game.active_team();
        let center_bounds = if game.home_playing {
            FieldCoordinateBounds::CENTER_FIELD_HOME
        } else {
            FieldCoordinateBounds::CENTER_FIELD_AWAY
        };
        let los_bounds = if game.home_playing {
            FieldCoordinateBounds::LOS_HOME
        } else {
            FieldCoordinateBounds::LOS_AWAY
        };

        let center_ids: Vec<String> = kicking_team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id)
                .map(|c| center_bounds.is_in_bounds(c))
                .unwrap_or(false))
            .map(|p| p.id.clone())
            .collect();

        let candidates: Vec<String> = if center_ids.is_empty() {
            kicking_team.players.iter()
                .filter(|p| game.field_model.player_coordinate(&p.id)
                    .map(|c| los_bounds.is_in_bounds(c))
                    .unwrap_or(false))
                .map(|p| p.id.clone())
                .collect()
        } else {
            center_ids
        };

        if candidates.is_empty() {
            return None;
        }

        // Prefer player with Kick skill
        for id in &candidates {
            if let Some(p) = game.player(id) {
                if p.all_skill_ids().any(|sk| sk == SkillId::Kick) {
                    return Some(id.clone());
                }
            }
        }
        candidates.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
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
        if let Some(coord) = game.field_model.ball_coordinate {
            assert!(FieldCoordinateBounds::FIELD.is_in_bounds(coord),
                "Ball placed off-field at {:?}", coord);
        }
    }

    #[test]
    fn chef_rolls_fire_on_the_very_first_kickoff_of_the_game() {
        // Java: gate is `game.getHalf() < 3 && turnDataHome.getTurnNr() == 0 && turnDataAway.getTurnNr() == 0`.
        // On a freshly-started game (half 1, turn_nr 0 for both teams — the actual game state at the
        // very first kickoff), this must fire. The old Rust code gated on `first_turn_after_kickoff`,
        // which defaults to `false` on a fresh game and is only set `true` by StepEndTurn's KICKOFF
        // case *after* the first drive's EndTurn — so it would incorrectly skip chef rolls on the
        // game's actual first kickoff.
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;

        fn run(seed: u64) -> i32 {
            let mut game = make_game();
            game.turn_data_home.inducement_set.add_inducement(
                Inducement::new("masterChef", 2, vec![Usage::STEAL_REROLL]),
            );
            game.turn_data_away.rerolls = 3;
            let mut step = StepKickoffScatterRoll::new();
            step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
            step.start(&mut game, &mut GameRng::new(seed));
            game.turn_data_away.rerolls
        }

        {
            let game = make_game();
            assert_eq!(game.half, 1);
            assert_eq!(game.turn_data_home.turn_nr, 0);
            assert_eq!(game.turn_data_away.turn_nr, 0);
            assert!(!game.turn_data_home.first_turn_after_kickoff, "fresh game has this flag false");
        }

        // Find a seed where the 2 home chefs steal at least one away re-roll.
        let stolen_seed = (0u64..2000).find(|&s| run(s) < 3);
        let remaining = stolen_seed.map(run)
            .expect("expected at least one seed where master chefs steal a re-roll");
        assert!(remaining < 3,
            "master chef rolls must fire on the game's first kickoff and steal an away re-roll");
    }

    #[test]
    fn touchback_state_consistent_with_bounds() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffScatterRoll::new();
        step.kickoff_start_coordinate = Some(FieldCoordinate::new(13, 7));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.touchback, step.kickoff_bounds.is_none());
    }
}
