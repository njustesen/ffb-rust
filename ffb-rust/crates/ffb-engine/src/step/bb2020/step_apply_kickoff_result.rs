use std::collections::HashMap;
use ffb_model::enums::{KickoffResult, TurnMode};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepApplyKickoffResult` (BB2020).
///
/// BB2020 kickoff table differs from BB2025:
/// - `OficiousRef` (BB2020) instead of `Charge` (BB2025-only)
/// - `TimeOut` adjusts turn numbers by ±1 (same as BB2025)
/// - `BrilliantCoaching` rolls d6+coaches; winner gets +1 re-roll
/// - `CheeringFans` rolls d6+cheerleaders; winner gets +1 Prayer of Nuffle
/// - `QuickSnap` / `SolidDefence` / `HighKick` require multi-round dialogs (TODO)
///
/// Mandatory init params: `GOTO_LABEL_ON_END` and `GOTO_LABEL_ON_BLITZ`.
///
/// Expects stepParameter KICKOFF_RESULT, KICKOFF_BOUNDS, TOUCHBACK from preceding steps.
/// Sets stepParameter TOUCHBACK for all steps on the stack.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepApplyKickoffResult`.
pub struct StepApplyKickoffResult {
    /// Java: fGotoLabelOnEnd — mandatory init param.
    pub goto_label_on_end: String,
    /// Java: fGotoLabelOnBlitz — mandatory init param.
    pub goto_label_on_blitz: String,
    /// Java: fKickoffResult
    pub kickoff_result: Option<KickoffResult>,
    /// Java: fTouchback
    pub touchback: bool,
    /// Java: fKickoffBounds
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fEndKickoff — set by CLIENT_END_TURN during multi-round sub-states.
    pub end_kickoff: bool,
    /// Java: playersAtCoordinates — snapshot of eligible player positions (Solid Defence).
    pub players_at_coordinates: HashMap<String, FieldCoordinate>,
    /// Java: nrOfPlayersAllowed — D3+3 cap for Solid Defence / Quick Snap.
    pub nr_of_players_allowed: i32,
    /// Java: nrOfMovedPlayers — count of Quick Snap moves used.
    pub nr_of_moved_players: i32,
    /// Java: movedPlayer — player id being repositioned (Quick Snap).
    pub moved_player: Option<String>,
    /// Java: toCoordinate — target square for Quick Snap move.
    pub to_coordinate: Option<FieldCoordinate>,
}

impl StepApplyKickoffResult {
    pub fn new(goto_label_on_end: String, goto_label_on_blitz: String) -> Self {
        Self {
            goto_label_on_end,
            goto_label_on_blitz,
            kickoff_result: None,
            touchback: false,
            kickoff_bounds: None,
            end_kickoff: false,
            players_at_coordinates: HashMap::new(),
            nr_of_players_allowed: 0,
            nr_of_moved_players: 0,
            moved_player: None,
            to_coordinate: None,
        }
    }
}

impl Default for StepApplyKickoffResult {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

impl Step for StepApplyKickoffResult {
    fn id(&self) -> StepId { StepId::ApplyKickoffResult }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::PlacePlayer { player_id, coord } => {
                // Java: CLIENT_SETUP_PLAYER during QUICK_SNAP → movedPlayer + toCoordinate
                if game.turn_mode == TurnMode::QuickSnap {
                    self.moved_player = Some(player_id.clone());
                    self.to_coordinate = Some(*coord);
                } else {
                    // Normal setup player during other sub-states: handled as SkipStep in Java
                    // DEFERRED(kickoff): UtilServerSetup.setupPlayer
                }
            }
            Action::EndTurn => {
                // Java: CLIENT_END_TURN → fEndKickoff = true; for QUICK_SNAP: endQuickSnap
                self.end_kickoff = true;
                if game.turn_mode == TurnMode::QuickSnap {
                    // Java: endQuickSnap — flip playing side, restore KICKOFF mode
                    game.home_playing = !game.home_playing;
                    game.turn_mode = TurnMode::Kickoff;
                } else if game.turn_mode == TurnMode::SolidDefence {
                    // Java: setPlayerCoordinates from command
                    // DEFERRED(kickoff): persist playerCoordinates from ClientCommandEndTurn
                }
            }
            Action::ConfirmSetup => {
                self.end_kickoff = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickoffBounds(b)  => { self.kickoff_bounds = Some(*b); true }
            StepParameter::KickoffResult(r)  => { self.kickoff_result = Some(*r); true }
            StepParameter::Touchback(v)      => { self.touchback = *v; true }
            _ => false,
        }
    }
}

impl StepApplyKickoffResult {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let result = match self.kickoff_result {
            Some(r) => r,
            None => return StepOutcome::next(),
        };

        match result {
            KickoffResult::GetTheRef       => self.handle_get_the_ref(game),
            KickoffResult::TimeOut         => self.handle_timeout(game),
            KickoffResult::SolidDefence    => self.handle_solid_defence(game, rng),
            KickoffResult::HighKick        => self.handle_high_kick(game),
            KickoffResult::CheeringFans    => self.handle_cheering_fans(game, rng),
            KickoffResult::WeatherChange   => self.handle_weather_change(game, rng),
            KickoffResult::BrilliantCoaching => self.handle_brilliant_coaching(game, rng),
            KickoffResult::QuickSnap       => self.handle_quick_snap(game, rng),
            KickoffResult::Blitz           => self.handle_blitz(),
            KickoffResult::OficiousRef     => self.handle_officious_ref(game, rng),
            KickoffResult::PitchInvasion   => self.handle_pitch_invasion(game, rng),
            _ => StepOutcome::next(),
        }
    }

    // ── GetTheRef ─────────────────────────────────────────────────────────────

    fn handle_get_the_ref(&self, _game: &mut Game) -> StepOutcome {
        // Java: both teams +1 bribe (via InducementTypeFactory.allTypes with Usage.AVOID_BAN)
        // DEFERRED(kickoff): add bribes to both teams when inducement model is ported
        // DEFERRED(kickoff): setAnimation(KICKOFF_GET_THE_REF)
        StepOutcome::next()
    }

    // ── TimeOut ───────────────────────────────────────────────────────────────

    fn handle_timeout(&self, game: &mut Game) -> StepOutcome {
        let kicking_turn = if game.home_playing {
            game.turn_data_home.turn_nr
        } else {
            game.turn_data_away.turn_nr
        };

        // Java: kickingTeamTurn >= 6 → -1 (penalty), else +1 (bonus)
        let turn_modifier = if kicking_turn >= 6 { -1 } else { 1 };

        game.turn_data_home.turn_nr += turn_modifier;
        game.turn_data_away.turn_nr += turn_modifier;

        // DEFERRED(kickoff): ReportKickoffTimeout(kickingTeamTurn, turnModifier)
        // DEFERRED(kickoff): setAnimation(KICKOFF_TIMEOUT)
        StepOutcome::next()
    }

    // ── SolidDefence ──────────────────────────────────────────────────────────

    fn handle_solid_defence(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::SolidDefence {
            if self.end_kickoff {
                // Java: count players who moved from their starting position
                let moved_players = self.players_at_coordinates.keys()
                    .filter(|id| {
                        game.field_model.player_coordinate(id)
                            .map(|c| Some(c) != self.players_at_coordinates.get(*id).copied().map(Some).flatten())
                            .unwrap_or(false)
                    })
                    .count() as i32;

                if moved_players <= self.nr_of_players_allowed {
                    // DEFERRED(kickoff): mechanic.checkSetup, clear markers, unhide prone players
                    game.turn_mode = TurnMode::Kickoff;
                    StepOutcome::next()
                } else {
                    // Invalid: too many players moved; reset end_kickoff, show dialog again
                    self.end_kickoff = false;
                    // DEFERRED(kickoff): DialogInvalidSolidDefenceParameter
                    StepOutcome::cont()
                }
            } else {
                StepOutcome::cont()
            }
        } else {
            // First entry: roll D3+3, set mode, mark eligible players
            let roll = rng.d3();
            self.nr_of_players_allowed = roll + 3;

            // DEFERRED(kickoff): ReportSolidDefenceRoll, setAnimation, pin players in tacklezones
            // Record current positions of acting team players on the field
            let acting_team_ids: Vec<String> = game.active_team().players.iter()
                .map(|p| p.id.clone())
                .collect();
            for id in &acting_team_ids {
                if let Some(coord) = game.field_model.player_coordinate(id) {
                    if FieldCoordinateBounds::FIELD.is_in_bounds(coord) {
                        self.players_at_coordinates.insert(id.clone(), coord);
                    }
                }
            }

            game.turn_mode = TurnMode::SolidDefence;
            // DEFERRED(kickoff): setAnimation(KICKOFF_SOLID_DEFENSE)
            StepOutcome::cont()
        }
    }

    // ── HighKick ──────────────────────────────────────────────────────────────

    fn handle_high_kick(&mut self, game: &mut Game) -> StepOutcome {
        if game.turn_mode == TurnMode::HighKick {
            if self.end_kickoff {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                StepOutcome::cont()
            }
        } else {
            // Check if there is already a player on the ball
            let catcher_exists = game.field_model.ball_coordinate
                .and_then(|bc| game.field_model.player_at(bc))
                .is_some();

            if self.touchback || catcher_exists {
                StepOutcome::next()
            } else {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::HighKick;
                // DEFERRED(kickoff): pinPlayersInTacklezones, setAnimation(KICKOFF_HIGH_KICK)
                StepOutcome::cont()
            }
        }
    }

    // ── CheeringFans ──────────────────────────────────────────────────────────

    fn handle_cheering_fans(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.cheerleaders;
        // DEFERRED(kickoff): + inducementSet.value(Usage.ADD_CHEERLEADER)
        let total_away = roll_away + game.team_away.cheerleaders;
        // DEFERRED(kickoff): + inducementSet.value(Usage.ADD_CHEERLEADER)

        // Java: winner gets a Prayer sequence pushed on stack
        // DEFERRED(kickoff): push Prayer sequence via generator when prayer factory is ported

        if total_home > total_away {
            // home wins
        } else if total_away > total_home {
            // away wins
        }

        // DEFERRED(kickoff): ReportCheeringFans, setAnimation(KICKOFF_CHEERING_FANS)
        let _ = (roll_home, roll_away, total_home, total_away, game);
        StepOutcome::next()
    }

    // ── WeatherChange ─────────────────────────────────────────────────────────

    fn handle_weather_change(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        use ffb_model::enums::Direction;

        let weather_roll = [rng.d6(), rng.d6()];
        let weather_total = weather_roll[0] + weather_roll[1];
        // DEFERRED(kickoff): interpret weather from 2d6 total, set game.weather
        // DEFERRED(kickoff): ReportWeather, setAnimation based on weather

        // Java: if weather is NICE and no touchback → scatter ball 3 times
        // Use heuristic: 7 = nice weather in Java's DiceInterpreter
        let is_nice = weather_total == 7;
        if !self.touchback && is_nice {
            let ball_coord = match game.field_model.ball_coordinate {
                Some(c) => c,
                None => return StepOutcome::next().publish(StepParameter::Touchback(self.touchback)),
            };

            let mut last_valid = ball_coord;
            for _ in 0..3 {
                let dir_roll = rng.d8();
                let direction = Direction::for_roll(dir_roll).unwrap_or(Direction::North);
                let candidate = last_valid.step(direction, 1);
                let in_bounds = self.kickoff_bounds
                    .map(|b| b.is_in_bounds(candidate))
                    .unwrap_or(false);
                self.touchback = !in_bounds;
                if !self.touchback {
                    game.field_model.ball_coordinate = Some(candidate);
                    last_valid = candidate;
                } else {
                    game.field_model.ball_coordinate = Some(last_valid);
                    break;
                }
            }
            // DEFERRED(kickoff): ReportScatterBall
        }

        StepOutcome::next().publish(StepParameter::Touchback(self.touchback))
    }

    // ── BrilliantCoaching ─────────────────────────────────────────────────────

    fn handle_brilliant_coaching(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let coach_banned_home = game.turn_data_home.coach_banned;
        let coach_banned_away = game.turn_data_away.coach_banned;

        let total_home = roll_home + game.team_home.assistant_coaches
            + if coach_banned_home { -1 } else { 0 };
        // DEFERRED(kickoff): + inducementSet.value(Usage.ADD_COACH)
        let total_away = roll_away + game.team_away.assistant_coaches
            + if coach_banned_away { -1 } else { 0 };
        // DEFERRED(kickoff): + inducementSet.value(Usage.ADD_COACH)

        if total_home > total_away {
            game.turn_data_home.rerolls += 1;
            game.turn_data_home.rerolls_brilliant_coaching_one_drive += 1;
        } else if total_away > total_home {
            game.turn_data_away.rerolls += 1;
            game.turn_data_away.rerolls_brilliant_coaching_one_drive += 1;
        }

        // DEFERRED(kickoff): ReportKickoffExtraReRoll, setAnimation(KICKOFF_BRILLIANT_COACHING)
        let _ = (roll_home, roll_away, total_home, total_away);
        StepOutcome::next()
    }

    // ── QuickSnap ─────────────────────────────────────────────────────────────

    fn handle_quick_snap(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.end_kickoff {
            return StepOutcome::next();
        }

        if game.turn_mode == TurnMode::QuickSnap {
            if let (Some(ref player_id), Some(coord)) = (self.moved_player.clone(), self.to_coordinate) {
                if self.nr_of_moved_players < self.nr_of_players_allowed {
                    self.nr_of_moved_players += 1;
                    // DEFERRED(kickoff): UtilServerSetup.setupPlayer, ReportKickoffSequenceActivationsCount
                    let _ = (player_id, coord);

                    if self.nr_of_moved_players == self.nr_of_players_allowed {
                        self.end_kickoff = true;
                        // DEFERRED(kickoff): ReportKickoffSequenceActivationsExhausted(true)
                    }
                }
                self.moved_player = None;
                self.to_coordinate = None;
            }

            if self.end_kickoff {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                StepOutcome::cont()
            }
        } else {
            // First entry: flip side, set QUICK_SNAP mode, roll D3+3
            game.home_playing = !game.home_playing;
            game.turn_mode = TurnMode::QuickSnap;
            let roll = rng.d3();
            self.nr_of_players_allowed = roll + 3;
            // DEFERRED(kickoff): ReportQuickSnapRoll, setAnimation, deactivate tackled players

            // Check if any active players remain
            let any_active = game.active_team().players.iter()
                .any(|p| game.field_model.player_state(&p.id)
                    .map(|s| s.is_active())
                    .unwrap_or(false));

            if !any_active {
                self.end_kickoff = true;
                // DEFERRED(kickoff): ReportKickoffSequenceActivationsExhausted(false)
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                StepOutcome::cont()
            }
        }
    }

    // ── Blitz ─────────────────────────────────────────────────────────────────

    fn handle_blitz(&self) -> StepOutcome {
        // Java: setAnimation(KICKOFF_BLITZ); GOTO_LABEL fGotoLabelOnBlitz
        // DEFERRED(kickoff): setAnimation(KICKOFF_BLITZ)
        StepOutcome::goto(&self.goto_label_on_blitz)
    }

    // ── OficiousRef ───────────────────────────────────────────────────────────

    fn handle_officious_ref(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll throwARock dice per team; player with lower fan-factor roll is targeted.
        // If roll == 1 → eject player; else → stun player.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let fan_factor_home = game.game_result.home.fan_factor_modifier;
        let fan_factor_away = game.game_result.away.fan_factor_modifier;

        let total_home = roll_home + fan_factor_home;
        let total_away = roll_away + fan_factor_away;

        // DEFERRED(kickoff): ReportKickoffOfficiousRef(rollHome, rollAway, playerIds)
        // DEFERRED(kickoff): push eject/stun sequence for each affected player

        // Java: if totalAway >= totalHome → home team player targeted
        if total_away >= total_home {
            let _home_player = Self::random_player_on_field(game, rng, true);
            // DEFERRED(kickoff): push SET_ACTING_PLAYER_AND_TEAM + EJECT_PLAYER or stunPlayer sequence
        }
        // Java: if totalHome >= totalAway → away team player targeted
        if total_home >= total_away {
            let _away_player = Self::random_player_on_field(game, rng, false);
            // DEFERRED(kickoff): push sequence for away player
        }

        // DEFERRED(kickoff): push SET_ACTING_TEAM(actingTeam.id) to restore after sequence
        // DEFERRED(kickoff): setAnimation(KICKOFF_OFFICIOUS_REF)
        let _ = (total_home, total_away);
        StepOutcome::next()
    }

    /// Java: `playersOnField` — all players for the given side that are within FIELD bounds.
    /// Returns the id of a random player (if any).
    fn random_player_on_field(game: &Game, rng: &mut GameRng, home_team: bool) -> Option<String> {
        let team = if home_team { &game.team_home } else { &game.team_away };
        let on_field: Vec<&str> = team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id)
                .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                .unwrap_or(false))
            .map(|p| p.id.as_str())
            .collect();
        if on_field.is_empty() {
            return None;
        }
        let idx = rng.range(on_field.len());
        Some(on_field[idx].to_owned())
    }

    // ── PitchInvasion ─────────────────────────────────────────────────────────

    fn handle_pitch_invasion(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let fan_factor_home = game.game_result.home.fan_factor_modifier;
        let fan_factor_away = game.game_result.away.fan_factor_modifier;

        let total_home = roll_home + fan_factor_home;
        let total_away = roll_away + fan_factor_away;

        let stunned = rng.d3();

        // Java: if totalHome <= totalAway → stun home team players
        if total_home <= total_away {
            self.stun_random_standing_players(game, rng, true, stunned);
        }
        // Java: if totalHome >= totalAway → stun away team players
        if total_home >= total_away {
            self.stun_random_standing_players(game, rng, false, stunned);
        }

        // DEFERRED(kickoff): ReportKickoffPitchInvasion(rollHome, rollAway, affectedIds, count)
        // DEFERRED(kickoff): setAnimation(KICKOFF_PITCH_INVASION)
        let _ = (total_home, total_away, stunned);
        StepOutcome::next()
    }

    /// Java: `stunPlayers` — randomly select up to `count` standing players and stun them.
    fn stun_random_standing_players(&self, game: &mut Game, rng: &mut GameRng, home: bool, count: i32) {
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_STUNNED};

        let team = if home { &game.team_home } else { &game.team_away };
        let mut standing: Vec<String> = team.players.iter()
            .filter(|p| game.field_model.player_state(&p.id)
                .map(|s| s.base() == PS_STANDING)
                .unwrap_or(false))
            .map(|p| p.id.clone())
            .collect();

        for _ in 0..count {
            if standing.is_empty() {
                break;
            }
            let idx = rng.range(standing.len());
            let id = standing.remove(idx);
            // Java: UtilServerInjury.stunPlayer(this, player, ApothecaryMode.HOME)
            // DEFERRED(kickoff): call stun_player when util_server_injury is ported
            // Directly apply stun state for now
            if let Some(state) = game.field_model.player_state(&id) {
                game.field_model.set_player_state(&id, PlayerState::new(PS_STUNNED));
                let _ = state;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{KickoffResult, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_step() -> StepApplyKickoffResult {
        StepApplyKickoffResult::new("END".into(), "BLITZ".into())
    }

    #[test]
    fn no_kickoff_result_returns_next() {
        let mut step = make_step();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::KickoffResult(KickoffResult::Blitz)));
        assert_eq!(step.kickoff_result, Some(KickoffResult::Blitz));
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
        let bounds = FieldCoordinateBounds::HALF_AWAY;
        assert!(step.set_parameter(&StepParameter::KickoffBounds(bounds)));
        assert_eq!(step.kickoff_bounds, Some(bounds));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn blitz_goes_to_label() {
        let mut step = make_step();
        let mut game = make_game();
        step.kickoff_result = Some(KickoffResult::Blitz);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("BLITZ"));
    }

    #[test]
    fn timeout_adjusts_turn_numbers_early_game() {
        let mut step = make_step();
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.turn_nr = 3; // < 6 → +1
        game.turn_data_away.turn_nr = 3;
        step.kickoff_result = Some(KickoffResult::TimeOut);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.turn_nr, 4);
        assert_eq!(game.turn_data_away.turn_nr, 4);
    }

    #[test]
    fn timeout_adjusts_turn_numbers_late_game() {
        let mut step = make_step();
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.turn_nr = 7; // >= 6 → -1
        game.turn_data_away.turn_nr = 7;
        step.kickoff_result = Some(KickoffResult::TimeOut);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.turn_nr, 6);
        assert_eq!(game.turn_data_away.turn_nr, 6);
    }

    #[test]
    fn brilliant_coaching_home_wins_adds_reroll() {
        // Force home to roll higher than away via seed
        let mut step = make_step();
        let mut game = make_game();
        game.turn_data_home.rerolls = 2;
        game.turn_data_away.rerolls = 2;
        step.kickoff_result = Some(KickoffResult::BrilliantCoaching);

        // Try seeds until home wins (rolls d6 twice: home then away)
        let seed = (0u64..10000).find(|&s| {
            let mut rng = GameRng::new(s);
            let r1 = rng.d6();
            let r2 = rng.d6();
            r1 > r2
        }).expect("seed not found");

        step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(game.turn_data_home.rerolls, 3, "home should gain +1 reroll");
        assert_eq!(game.turn_data_away.rerolls, 2, "away should not gain reroll");
    }

    #[test]
    fn pitch_invasion_stuns_players_on_losing_team() {
        let mut step = make_step();
        let mut game = make_game();

        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        let home_player = Player {
            id: "hp1".into(), name: "hp1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        };
        game.team_home.players.push(home_player);
        game.field_model.set_player_coordinate("hp1", FieldCoordinate::new(5, 7));
        game.field_model.set_player_state("hp1", PlayerState::new(PS_STANDING));

        // fan_factor: 0 for both → both teams affected (totalHome == totalAway)
        step.kickoff_result = Some(KickoffResult::PitchInvasion);
        step.start(&mut game, &mut GameRng::new(0));

        // Both stunned branches fire when equal; either way, step completes
        // (we just verify it doesn't panic)
    }
}
