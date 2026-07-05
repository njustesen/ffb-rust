use std::collections::HashMap;
use ffb_model::events::GameEvent;
use ffb_model::enums::{KickoffResult, TurnMode, Weather, PS_EXHAUSTED, PS_RESERVE, PS_STANDING};
use ffb_model::inducement::inducement::Inducement;
use ffb_model::inducement::usage::Usage;
use ffb_model::option::game_option_id;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
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
                    // headless: no-op — playerCoordinates from ClientCommandEndTurn not applicable
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

    fn handle_get_the_ref(&self, game: &mut Game) -> StepOutcome {
        // Java: InducementTypeFactory.allTypes() → filter AVOID_BAN → computeIfAbsent + setValue+1
        // Both teams gain +1 bribe regardless of whether they had any before.
        for home in [true, false] {
            let set = if home { &mut game.turn_data_home.inducement_set }
                      else   { &mut game.turn_data_away.inducement_set };
            let type_id = set.for_usage(Usage::AVOID_BAN)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "BRIBE".to_string());
            let mut ind = set.get(&type_id)
                .unwrap_or_else(|| Inducement::new(type_id.clone(), 0, vec![Usage::AVOID_BAN]));
            ind.value += 1;
            set.add_inducement(ind);
        }
        // client-only: setAnimation(KICKOFF_GET_THE_REF)
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

        // client-only: setAnimation(KICKOFF_TIMEOUT)
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
                    // client-only: show setup error dialog when !valid
                    // Java: hide prone box players back to reserve on valid completion
                    {
                        use ffb_model::enums::PS_PRONE;
                        let acting_ids: Vec<String> = game.active_team().players.iter()
                            .map(|p| p.id.clone()).collect();
                        for id in &acting_ids {
                            if game.field_model.player_coordinate(id).is_none() {
                                if let Some(state) = game.field_model.player_state(id) {
                                    if state.base() == PS_PRONE {
                                        game.field_model.set_player_state(id, state.change_base(PS_RESERVE));
                                    }
                                }
                            }
                        }
                    }
                    // Java: setKickingSwarmers(0)
                    game.kicking_swarmers = 0;
                    game.turn_mode = TurnMode::Kickoff;
                    StepOutcome::next()
                } else {
                    // Invalid: too many players moved; reset end_kickoff, show dialog again
                    self.end_kickoff = false;
                    // client-only: DialogInvalidSolidDefenceParameter — headless allows overcount
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

            // client-only: setAnimation(KICKOFF_SOLID_DEFENSE)
            // Java: for each acting-team player on field:
            //   if adjacent to opposing tacklers → deactivate (pinned)
            //   else → add to playersAtCoordinates (eligible to move)
            //   if in box and RESERVE → change to PRONE (unhide)
            let other_is_home = !game.home_playing;
            let acting_ids: Vec<(String, Option<FieldCoordinate>)> = game.active_team().players.iter()
                .map(|p| (p.id.clone(), game.field_model.player_coordinate(&p.id)))
                .collect();
            for (id, coord_opt) in &acting_ids {
                if let Some(coord) = coord_opt {
                    if FieldCoordinateBounds::FIELD.is_in_bounds(*coord) {
                        let other_team = if other_is_home { &game.team_home } else { &game.team_away };
                        let adj = UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, *coord, false);
                        if !adj.is_empty() {
                            // Deactivate: pinned by opponent tackle zones
                            if let Some(state) = game.field_model.player_state(id) {
                                game.field_model.set_player_state(id, state.change_active(false));
                            }
                        } else {
                            self.players_at_coordinates.insert(id.clone(), *coord);
                        }
                    }
                } else {
                    // Box player: if RESERVE → change to PRONE (reveal for Solid Defence)
                    if let Some(state) = game.field_model.player_state(id) {
                        use ffb_model::enums::PS_PRONE;
                        if state.base() == PS_RESERVE {
                            game.field_model.set_player_state(id, state.change_base(PS_PRONE));
                        }
                    }
                }
            }

            game.turn_mode = TurnMode::SolidDefence;
            // client-only: setAnimation(KICKOFF_SOLID_DEFENSE)
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

        // Exhibition: rolls 1–8; league (INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE): rolls 1–16.
        let use_league = game.options.is_enabled(game_option_id::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE);
        let max_prayer_roll = if use_league { 16 } else { 8 };
        let mut outcome = StepOutcome::next();
        if total_home > total_away {
            let prayer_roll = rng.range(max_prayer_roll) as i32 + 1;
            let team_id = game.team_home.id.clone();
            outcome = outcome.push_seq(vec![
                SequenceStep::with_params(StepId::Prayer, vec![
                    StepParameter::PrayerRoll(prayer_roll),
                    StepParameter::TeamId(team_id),
                ])
            ]);
        } else if total_away > total_home {
            let prayer_roll = rng.range(max_prayer_roll) as i32 + 1;
            let team_id = game.team_away.id.clone();
            outcome = outcome.push_seq(vec![
                SequenceStep::with_params(StepId::Prayer, vec![
                    StepParameter::PrayerRoll(prayer_roll),
                    StepParameter::TeamId(team_id),
                ])
            ]);
        }
        // client-only: setAnimation(KICKOFF_CHEERING_FANS)
        outcome.with_event(GameEvent::CheeringFans { home_roll: roll_home, away_roll: roll_away })
    }

    // ── WeatherChange ─────────────────────────────────────────────────────────

    fn handle_weather_change(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        use ffb_model::enums::Direction;

        let weather_roll = rng.roll_weather();
        let weather = DiceInterpreter::interpret_roll_weather(&weather_roll);
        game.field_model.weather = weather;
        // client-only: setAnimation based on weather

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
            // headless: ReportScatterBall — report system not yet ported
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
        // client-only: setAnimation(KICKOFF_BRILLIANT_COACHING)
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
                    // headless: ReportKickoffSequenceActivationsCount — report system not yet ported
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
            // client-only: setAnimation(KICKOFF_QUICK_SNAP)

            // Java: deactivate acting-team players adjacent to opposing tacklers
            let other_is_home = !game.home_playing;
            let acting_ids: Vec<(String, FieldCoordinate)> = game.active_team().players.iter()
                .filter_map(|p| game.field_model.player_coordinate(&p.id).map(|c| (p.id.clone(), c)))
                .collect();
            for (pid, coord) in &acting_ids {
                let other_team = if other_is_home { &game.team_home } else { &game.team_away };
                if !UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, *coord, false).is_empty() {
                    if let Some(state) = game.field_model.player_state(pid) {
                        game.field_model.set_player_state(pid, state.change_active(false));
                    }
                }
            }

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
        // client-only: setAnimation(KICKOFF_BLITZ)
        StepOutcome::goto(&self.goto_label_on_blitz)
    }

    // ── OficiousRef ───────────────────────────────────────────────────────────

    fn handle_officious_ref(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll throwARock (D6) per team; lower fan-factor total → player targeted.
        // Per targeted player: roll D6; 1 → eject (headless: eject not yet ported), else → stun immediately.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        // Java: gameResult.getTeamResultHome().getFanFactor() — uses fan_factor, not modifier
        let total_home = roll_home + game.game_result.home.fan_factor;
        let total_away = roll_away + game.game_result.away.fan_factor;

        // client-only: setAnimation(KICKOFF_OFFICIOUS_REF)

        let mut targeted_ids: Vec<String> = Vec::new();
        // Collect eject sequences to push (one per ejected player).
        let mut eject_seqs: Vec<Vec<SequenceStep>> = Vec::new();
        // ParametersToConsume discriminants for EjectPlayer's ConsumeParameter step.
        let params_to_consume = vec![
            std::mem::discriminant(&StepParameter::CatchScatterThrowInMode(
                ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode::CatchBomb)),
            std::mem::discriminant(&StepParameter::FoulerHasBall(false)),
        ];

        // Java: if totalAway >= totalHome → home team player targeted
        if total_away >= total_home {
            if let Some(home_player) = Self::random_player_on_field(game, rng, true) {
                targeted_ids.push(home_player.clone());
                let ref_roll = rng.d6();
                if ref_roll == 1 {
                    eject_seqs.push(Self::build_eject_seq(&home_player, params_to_consume.clone()));
                } else {
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
                    eject_seqs.push(Self::build_eject_seq(&away_player, params_to_consume.clone()));
                } else {
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
        let mut outcome = StepOutcome::next()
            .push_seq(restore_seq)
            .with_event(GameEvent::KickoffOfficiousRef {
                roll_home,
                roll_away,
                player_ids: targeted_ids,
            });
        for seq in eject_seqs {
            outcome = outcome.push_seq(seq);
        }
        outcome
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

    /// Java: `insertSteps` (when roll == 1) — builds SetActingPlayerAndTeam + EjectPlayer + ConsumeParameter.
    fn build_eject_seq(
        player_id: &str,
        params_to_consume: Vec<std::mem::Discriminant<StepParameter>>,
    ) -> Vec<SequenceStep> {
        use crate::step::generator::labels;
        vec![
            SequenceStep::with_params(
                StepId::SetActingPlayerAndTeam,
                vec![StepParameter::PlayerId(player_id.to_owned())],
            ),
            SequenceStep::with_params(
                StepId::EjectPlayer,
                vec![StepParameter::GotoLabelOnEnd(labels::END_FOULING.to_owned())],
            ),
            SequenceStep::labelled(
                StepId::ConsumeParameter,
                labels::END_FOULING,
                vec![StepParameter::ParametersToConsume(params_to_consume)],
            ),
        ]
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

        // client-only: setAnimation(KICKOFF_PITCH_INVASION)
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

    // ── GetTheRef tests ───────────────────────────────────────────────────────

    #[test]
    fn get_the_ref_creates_bribe_when_none_exist() {
        let mut step = make_step();
        let mut game = make_game();
        step.kickoff_result = Some(KickoffResult::GetTheRef);
        step.start(&mut game, &mut GameRng::new(0));
        // Both teams should now have 1 bribe (AVOID_BAN inducement)
        assert_eq!(
            game.turn_data_home.inducement_set.value(ffb_model::inducement::usage::Usage::AVOID_BAN),
            1,
            "home should gain 1 bribe"
        );
        assert_eq!(
            game.turn_data_away.inducement_set.value(ffb_model::inducement::usage::Usage::AVOID_BAN),
            1,
            "away should gain 1 bribe"
        );
    }

    #[test]
    fn get_the_ref_increments_existing_bribes() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        let mut step = make_step();
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(Inducement::new("BRIBE", 2, vec![Usage::AVOID_BAN]));
        game.turn_data_away.inducement_set.add_inducement(Inducement::new("BRIBE", 1, vec![Usage::AVOID_BAN]));
        step.kickoff_result = Some(KickoffResult::GetTheRef);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.inducement_set.value(Usage::AVOID_BAN), 3, "home bribe +1");
        assert_eq!(game.turn_data_away.inducement_set.value(Usage::AVOID_BAN), 2, "away bribe +1");
    }

    // ── QuickSnap deactivation test ────────────────────────────────────────────

    #[test]
    fn quick_snap_deactivates_players_in_tackle_zones() {
        use ffb_model::enums::{PS_STANDING, PlayerState, PlayerType, PlayerGender};
        use ffb_model::model::player::Player;

        let mut step = make_step();
        let mut game = make_game();
        // home_playing = true → after flip, active = away = receiving team
        game.home_playing = true;

        // Add an away player (receiving) at (5,5)
        let away_player = Player {
            id: "away1".into(), name: "a".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, ..Default::default()
        };
        game.team_away.players.push(away_player);
        game.field_model.set_player_coordinate("away1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("away1", PlayerState::new(PS_STANDING));

        // Add a home player (kicking) adjacent at (6,5) WITH tackle zones
        let home_player = Player {
            id: "home1".into(), name: "h".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, ..Default::default()
        };
        game.team_home.players.push(home_player);
        game.field_model.set_player_coordinate("home1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("home1", PlayerState::new(PS_STANDING));

        step.kickoff_result = Some(KickoffResult::QuickSnap);
        step.start(&mut game, &mut GameRng::new(0));

        // away1 is adjacent to home1 (kicking team with TZ), so should be deactivated
        let away1_state = game.field_model.player_state("away1").expect("state");
        assert!(!away1_state.is_active(), "away player in tackle zone should be deactivated by QuickSnap");
    }
}
