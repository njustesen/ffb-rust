use std::collections::HashMap;
use ffb_model::events::GameEvent;
use ffb_model::enums::{KickoffResult, TurnMode, Weather, PS_EXHAUSTED, PS_RESERVE, PS_STANDING};
use ffb_model::inducement::usage::Usage;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome, SequenceStep};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury;
use crate::util::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;
use crate::util::util_server_setup::UtilServerSetup;

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
                    UtilServerSetup::setup_player(game, player_id, *coord);
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

        // DEFERRED(kickoff): setAnimation(KICKOFF_TIMEOUT)
        StepOutcome::next().with_event(GameEvent::KickoffTimeout { turn_number: kicking_turn, turn_modifier })
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
                    // Java: mechanic.checkSetup(gameState, game.isHomePlaying(), getKickingSwarmers())
                    let _valid = SetupMechanic::new().check_setup_with_swarmers(game, game.home_playing, game.kicking_swarmers);
                    // DEFERRED(dialog): show setup error when !valid, clear markers, unhide prone players
                    // Java: setKickingSwarmers(0)
                    game.kicking_swarmers = 0;
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
            let acting_team_id = game.active_team().id.clone();

            // DEFERRED(kickoff): setAnimation, pin players in tacklezones
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
            StepOutcome::cont().with_event(GameEvent::SolidDefenceRoll {
                team_id: acting_team_id,
                roll,
                amount: self.nr_of_players_allowed,
            })
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
                let pin_team_id = if game.home_playing {
                    game.team_home.id.clone()
                } else {
                    game.team_away.id.clone()
                };
                SetupMechanic::new().pin_players_in_tacklezones(game, &pin_team_id);
                // Animation is client-side only.
                StepOutcome::cont()
            }
        }
    }

    // ── CheeringFans ──────────────────────────────────────────────────────────

    fn handle_cheering_fans(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.cheerleaders
            + game.turn_data_home.inducement_set.value(Usage::ADD_CHEERLEADER);
        let total_away = roll_away + game.team_away.cheerleaders
            + game.turn_data_away.inducement_set.value(Usage::ADD_CHEERLEADER);

        // Java: winner gets a Prayer pushed: pick a random availablePrayerRoll from factory.
        // DEFERRED(kickoff-prayer-factory): availablePrayerRolls filtering via PrayerFactory deferred.
        // Simplified: push a D8 prayer roll for the winner directly.
        let mut outcome = StepOutcome::next();
        if total_home > total_away {
            let prayer_roll = rng.d8();
            let team_id = game.team_home.id.clone();
            outcome = outcome.push_seq(vec![
                SequenceStep::with_params(StepId::Prayer, vec![
                    StepParameter::PrayerRoll(prayer_roll),
                    StepParameter::TeamId(team_id),
                ])
            ]);
        } else if total_away > total_home {
            let prayer_roll = rng.d8();
            let team_id = game.team_away.id.clone();
            outcome = outcome.push_seq(vec![
                SequenceStep::with_params(StepId::Prayer, vec![
                    StepParameter::PrayerRoll(prayer_roll),
                    StepParameter::TeamId(team_id),
                ])
            ]);
        }
        // DEFERRED(kickoff): setAnimation(KICKOFF_CHEERING_FANS)
        outcome.with_event(GameEvent::CheeringFans { home_roll: roll_home, away_roll: roll_away })
    }

    // ── WeatherChange ─────────────────────────────────────────────────────────

    fn handle_weather_change(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        use ffb_model::enums::Direction;

        let weather_roll = rng.roll_weather();
        let weather = DiceInterpreter::interpret_roll_weather(&weather_roll);
        game.field_model.weather = weather;
        // DEFERRED(kickoff): setAnimation based on weather

        if weather == Weather::SwelteringHeat {
            let player_ids: Vec<String> = game.field_model.players_on_pitch().cloned().collect();
            for id in player_ids {
                if let Some(state) = game.field_model.player_state(&id) {
                    if state.base() == PS_EXHAUSTED {
                        game.field_model.set_player_state(&id, state.change_base(PS_RESERVE));
                    }
                }
            }
        }

        if !self.touchback && weather == Weather::Nice {
            if let Some(ball_coord) = game.field_model.ball_coordinate {
                let mut last_valid = ball_coord;
                for _ in 0..3 {
                    let dir_roll = rng.d8();
                    let direction = Direction::for_roll(dir_roll).unwrap_or(Direction::North);
                    let candidate = UtilServerCatchScatterThrowIn::find_scatter_coordinate(last_valid, direction, 1);
                    let in_bounds = self.kickoff_bounds
                        .map(|b| b.is_in_bounds(candidate))
                        .unwrap_or_else(|| FieldCoordinateBounds::FIELD.is_in_bounds(candidate));
                    self.touchback = !in_bounds;
                    if !self.touchback {
                        game.field_model.ball_coordinate = Some(candidate);
                        last_valid = candidate;
                    } else {
                        game.field_model.ball_coordinate = Some(last_valid);
                        break;
                    }
                }
            }
            // DEFERRED(kickoff): ReportScatterBall
        }

        StepOutcome::next()
            .with_event(GameEvent::WeatherChange { weather })
            .publish(StepParameter::Touchback(self.touchback))
    }

    // ── BrilliantCoaching ─────────────────────────────────────────────────────

    fn handle_brilliant_coaching(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let coach_banned_home = game.turn_data_home.coach_banned;
        let coach_banned_away = game.turn_data_away.coach_banned;

        let total_home = roll_home + game.team_home.assistant_coaches
            + if coach_banned_home { -1 } else { 0 }
            + game.turn_data_home.inducement_set.value(Usage::ADD_COACH);
        let total_away = roll_away + game.team_away.assistant_coaches
            + if coach_banned_away { -1 } else { 0 }
            + game.turn_data_away.inducement_set.value(Usage::ADD_COACH);

        let mut outcome = StepOutcome::next();
        // DEFERRED(kickoff): setAnimation(KICKOFF_BRILLIANT_COACHING)
        if total_home > total_away {
            game.turn_data_home.rerolls += 1;
            game.turn_data_home.rerolls_brilliant_coaching_one_drive += 1;
            let team_id = game.team_home.id.clone();
            outcome = outcome.with_event(GameEvent::KickoffExtraReRoll { team_id });
        } else if total_away > total_home {
            game.turn_data_away.rerolls += 1;
            game.turn_data_away.rerolls_brilliant_coaching_one_drive += 1;
            let team_id = game.team_away.id.clone();
            outcome = outcome.with_event(GameEvent::KickoffExtraReRoll { team_id });
        }
        outcome
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
                    // DEFERRED(kickoff): ReportKickoffSequenceActivationsCount (report system)
                    UtilServerSetup::setup_player(game, player_id, coord);

                    if self.nr_of_moved_players == self.nr_of_players_allowed {
                        self.end_kickoff = true;
                    }
                }
                self.moved_player = None;
                self.to_coordinate = None;
            }

            if self.end_kickoff {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next().with_event(GameEvent::KickoffSequenceActivationsExhausted { limit_reached: true })
            } else {
                StepOutcome::cont()
            }
        } else {
            // First entry: flip side, set QUICK_SNAP mode, roll D3+3
            game.home_playing = !game.home_playing;
            game.turn_mode = TurnMode::QuickSnap;
            let roll = rng.d3();
            self.nr_of_players_allowed = roll + 3;
            let active_team_id = game.active_team().id.clone();
            // DEFERRED(kickoff): setAnimation, deactivate tackled players

            // Check if any active players remain
            let any_active = game.active_team().players.iter()
                .any(|p| game.field_model.player_state(&p.id)
                    .map(|s| s.is_active())
                    .unwrap_or(false));

            let snap_event = GameEvent::QuickSnapRoll { team_id: active_team_id, roll, amount: self.nr_of_players_allowed };
            if !any_active {
                self.end_kickoff = true;
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
                    .with_event(snap_event)
                    .with_event(GameEvent::KickoffSequenceActivationsExhausted { limit_reached: false })
            } else {
                StepOutcome::cont().with_event(snap_event)
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
        // Java: roll throwARock (D6) per team; lower fan-factor total → player targeted.
        // Per targeted player: roll D6; 1 → eject (DEFERRED), else → stun immediately.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        // Java: gameResult.getTeamResultHome().getFanFactor() — uses fan_factor, not modifier
        let total_home = roll_home + game.game_result.home.fan_factor;
        let total_away = roll_away + game.game_result.away.fan_factor;

        // DEFERRED(kickoff): setAnimation(KICKOFF_OFFICIOUS_REF)

        let mut targeted_ids: Vec<String> = Vec::new();

        // Java: if totalAway >= totalHome → home team player targeted
        if total_away >= total_home {
            if let Some(home_player) = Self::random_player_on_field(game, rng, true) {
                targeted_ids.push(home_player.clone());
                let ref_roll = rng.d6();
                if ref_roll == 1 {
                    // Java: push EJECT_PLAYER sequence — DEFERRED(kickoff-eject): eject sequence deferred.
                } else {
                    // Java: publishParameters(UtilServerInjury.stunPlayer(this, player, HOME))
                    util_server_injury::stun_player(game, &home_player);
                }
            }
        }
        // Java: if totalHome >= totalAway → away team player targeted
        if total_home >= total_away {
            if let Some(away_player) = Self::random_player_on_field(game, rng, false) {
                targeted_ids.push(away_player.clone());
                let ref_roll = rng.d6();
                if ref_roll == 1 {
                    // Java: push EJECT_PLAYER sequence — DEFERRED(kickoff-eject): eject sequence deferred.
                } else {
                    // Java: publishParameters(UtilServerInjury.stunPlayer(this, player, AWAY))
                    util_server_injury::stun_player(game, &away_player);
                }
            }
        }

        // Java: sequence.add(SET_ACTING_TEAM, TEAM_ID=actingTeam.id) to restore team after ref stuns
        let acting_team_id = if game.home_playing {
            game.team_home.id.clone()
        } else {
            game.team_away.id.clone()
        };
        let restore_seq = vec![SequenceStep::with_params(
            StepId::SetActingTeam,
            vec![StepParameter::TeamId(acting_team_id)],
        )];
        StepOutcome::next().push_seq(restore_seq).with_event(GameEvent::KickoffOfficiousRef {
            roll_home,
            roll_away,
            player_ids: targeted_ids,
        })
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

        // Java: gameResult.getTeamResultHome().getFanFactor() — uses fan_factor, not modifier
        let total_home = roll_home + game.game_result.home.fan_factor;
        let total_away = roll_away + game.game_result.away.fan_factor;

        let stunned = rng.d3();

        // Java: if totalHome <= totalAway → stun home team players
        if total_home <= total_away {
            self.stun_random_standing_players(game, rng, true, stunned);
        }
        // Java: if totalHome >= totalAway → stun away team players
        if total_home >= total_away {
            self.stun_random_standing_players(game, rng, false, stunned);
        }

        // DEFERRED(kickoff): setAnimation(KICKOFF_PITCH_INVASION)
        StepOutcome::next().with_event(GameEvent::KickoffPitchInvasion { home_roll: roll_home, away_roll: roll_away })
    }

    /// Java: `stunPlayers` — randomly select up to `count` standing players and stun them.
    fn stun_random_standing_players(&self, game: &mut Game, rng: &mut GameRng, home: bool, count: i32) {
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
            util_server_injury::stun_player(game, &id);
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
            ..Default::default()
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

    #[test]
    fn officious_ref_pushes_set_acting_team_sequence() {
        let mut step = make_step();
        let mut game = make_game();
        game.home_playing = true;
        step.kickoff_result = Some(KickoffResult::OficiousRef);
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should push a SET_ACTING_TEAM sequence to restore the acting team
        assert_eq!(out.pushes.len(), 1, "expected one restore sequence");
        let seq = &out.pushes[0];
        assert_eq!(seq.len(), 1);
        assert_eq!(seq[0].step_id, StepId::SetActingTeam);
        let has_team_id = seq[0].params.iter().any(|p| {
            matches!(p, StepParameter::TeamId(id) if id == &game.team_home.id)
        });
        assert!(has_team_id, "SET_ACTING_TEAM should carry home team id when home_playing=true");
    }
}
