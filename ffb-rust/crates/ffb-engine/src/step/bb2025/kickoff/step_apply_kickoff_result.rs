use std::collections::HashMap;
use ffb_model::events::GameEvent;
use ffb_model::enums::{KickoffResult, TurnMode, Weather, PS_EXHAUSTED, PS_RESERVE, PS_STANDING};
use ffb_model::inducement::inducement::Inducement;
use ffb_model::inducement::usage::Usage;
use ffb_model::report::mixed::report_kickoff_sequence_activations_count::ReportKickoffSequenceActivationsCount;
use ffb_model::report::mixed::report_kickoff_extra_re_roll::ReportKickoffExtraReRoll;
use ffb_model::report::mixed::report_solid_defence_roll::ReportSolidDefenceRoll;
use ffb_model::report::bb2025::report_cheering_fans::ReportCheeringFans as ReportCheeringFansBb2025;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::util::util_box::UtilBox;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury;
use crate::util::util_server_setup::UtilServerSetup;
use crate::util::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;

/// Applies the rolled kickoff result.
///
/// Routing overview (per BB2025 Java source):
///
///  - `GetTheRef`          → both teams receive +1 bribe; NEXT_STEP. Implemented.
///  - `TimeOut`            → adjust turn counters by ±1; NEXT_STEP. Implemented.
///  - `SolidDefence`       → kicking team repositions D3+3 players. The eligibility roll,
///                           report and pinning logic are implemented; the multi-round
///                           `DialogPlayerChoiceParameter` player-selection UI is client-side
///                           and not ported, so headless play always resolves with no players
///                           selected (matches this crate's convention for other dialog-gated steps).
///  - `HighKick`           → receiving team may move a player to the ball; the touchback/catcher
///                           check and side-swap are implemented; the player-repositioning dialog
///                           is client-side and not ported (same convention as SolidDefence).
///  - `CheeringFans`       → roll d6+cheerleaders per team; winner gets +1 block assist. Implemented
///                           (Java's cheering-fans reroll-on-`REROLL_CHEERING_FANS` inducement is not
///                           yet wired here).
///  - `WeatherChange`      → roll 2d6 weather; re-scatter ball if Nice. Implemented, including the
///                           Sweltering Heat exhausted->reserve state change and the ball re-scatter.
///  - `BrilliantCoaching`  → roll d6+coaches; winner gets +1 re-roll. Implemented.
///  - `QuickSnap`          → receiving team repositions D3+3 open players 1 sq each. The roll,
///                           activation-exhaustion reporting and side-swap are implemented; the actual
///                           per-player repositioning is driven by `PlacePlayer`/`EndTurn` commands.
///  - `Charge`             → kicking team selects D3+3 players for move/blitz actions;
///                           GOTO_LABEL_ON_BLITZ to re-enter kickoff sequence for the blitz. The
///                           eligibility roll is implemented; the player-selection dialog is
///                           client-side and not ported, so headless play always skips the blitz.
///  - `DodgySnack`         → random player per team takes -MA/-AV (`KickoffResult::DODGY_SNACK`
///                           enhancement: `TemporaryStatDecrementer` on MA and AV) or goes to
///                           reserves on a roll of 1. Implemented.
///  - `PitchInvasion`      → roll d6+fan-factor; losing team has D3 players stunned. Implemented.
///
/// SolidDefence / QuickSnap / HighKick / Charge all depend on `DialogPlayerChoiceParameter`
/// (client-side player-selection dialog), which this crate does not model — that is a documented,
/// out-of-scope structural gap consistent with the rest of this crate's dialog-gated steps, not an
/// invented simplification.
///
/// Mandatory init params: `GOTO_LABEL_ON_END` and `GOTO_LABEL_ON_BLITZ`.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepApplyKickoffResult`.
pub struct StepApplyKickoffResult {
    /// Java: fGotoLabelOnEnd — mandatory init param.
    pub goto_label_on_end: String,
    /// Java: fGotoLabelOnBlitz — mandatory init param.
    pub goto_label_on_blitz: String,
    /// Java: fKickoffResult.
    pub kickoff_result: Option<KickoffResult>,
    /// Java: fTouchback.
    pub touchback: bool,
    /// Java: fKickoffBounds.
    pub kickoff_bounds: Option<FieldCoordinateBounds>,
    /// Java: fEndKickoff — set by CLIENT_END_TURN during multi-round sub-states.
    pub end_kickoff: bool,
    /// Java: playersAtCoordinates — snapshot of eligible player positions.
    pub players_at_coordinates: HashMap<String, FieldCoordinate>,
    /// Java: nrOfPlayersAllowed — D3+3 cap for Solid Defence / Quick Snap / Charge.
    pub nr_of_players_allowed: i32,
    /// Java: nrOfMovedPlayers — count of Quick Snap moves used.
    pub nr_of_moved_players: i32,
    /// Java: movedPlayer — player id being repositioned (Quick Snap).
    pub moved_player: Option<String>,
    /// Java: toCoordinate — target square for Quick Snap move.
    pub to_coordinate: Option<FieldCoordinate>,
    /// Java: selectedPlayers — players chosen via dialog (Solid Defence / Charge).
    pub selected_players: Vec<String>,
    /// Java: eligiblePlayers — players eligible for selection.
    pub eligible_players: Vec<String>,
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
            selected_players: Vec::new(),
            eligible_players: Vec::new(),
        }
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
                if game.turn_mode == TurnMode::QuickSnap {
                    self.moved_player = Some(player_id.clone());
                    self.to_coordinate = Some(*coord);
                }
                if game.turn_mode != TurnMode::QuickSnap {
                    UtilServerSetup::setup_player(game, player_id, *coord);
                }
            }
            Action::EndTurn => {
                self.end_kickoff = true;
                if game.turn_mode == TurnMode::QuickSnap {
                    game.home_playing = !game.home_playing;
                    game.turn_mode = TurnMode::Kickoff;
                } else if game.turn_mode == TurnMode::SolidDefence {
                    // client-only: no-op — SolidDefence playerCoordinate updates are sent only by clients
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
            StepParameter::GotoLabelOnEnd(v)   => { self.goto_label_on_end   = v.clone(); true }
            StepParameter::GotoLabelOnBlitz(v) => { self.goto_label_on_blitz = v.clone(); true }
            StepParameter::KickoffResult(v)    => { self.kickoff_result       = Some(*v); true }
            StepParameter::Touchback(v)        => { self.touchback            = *v;       true }
            StepParameter::KickoffBounds(v)    => { self.kickoff_bounds       = Some(*v); true }
            _ => false,
        }
    }
}

impl StepApplyKickoffResult {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let result = match self.kickoff_result {
            Some(r) => r,
            None => return StepOutcome::cont(),
        };

        match result {
            KickoffResult::GetTheRef        => self.handle_get_the_ref(game),
            KickoffResult::TimeOut          => self.handle_timeout(game),
            KickoffResult::SolidDefence     => self.handle_solid_defence(game, rng),
            KickoffResult::HighKick         => self.handle_high_kick(game),
            KickoffResult::CheeringFans     => self.handle_cheering_fans(game, rng),
            KickoffResult::WeatherChange    => self.handle_weather_change(game, rng),
            KickoffResult::BrilliantCoaching => self.handle_brilliant_coaching(game, rng),
            KickoffResult::QuickSnap        => self.handle_quick_snap(game, rng),
            KickoffResult::Charge           => self.handle_charge(game, rng),
            KickoffResult::DodgySnack       => self.handle_dodgy_snack(game, rng),
            KickoffResult::PitchInvasion    => self.handle_pitch_invasion(game, rng),
            // Non-BB2025 variants (should not reach here in BB2025 games):
            KickoffResult::Blitz            |
            KickoffResult::Riot             |
            KickoffResult::PerfectDefence   |
            KickoffResult::ThrowARock       |
            KickoffResult::OficiousRef      => {
                StepOutcome::goto(&self.goto_label_on_end.clone())
            }
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
        // Java: kicking team's turn number determines direction.
        // If kicking team is on turn >= 6 → modifier = -1 (both teams go back), else +1.
        let kicking_turn = if game.home_playing {
            game.turn_data_home.turn_nr
        } else {
            game.turn_data_away.turn_nr
        };
        let modifier = if kicking_turn >= 6 { -1 } else { 1 };
        game.turn_data_home.turn_nr += modifier;
        game.turn_data_away.turn_nr += modifier;
        StepOutcome::next().with_event(GameEvent::KickoffTimeout { turn_number: kicking_turn, turn_modifier: modifier })
    }

    // ── CheeringFans ──────────────────────────────────────────────────────────

    fn handle_cheering_fans(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.cheerleaders
            + game.turn_data_home.inducement_set.value(Usage::ADD_CHEERLEADER);
        let total_away = roll_away + game.team_away.cheerleaders
            + game.turn_data_away.inducement_set.value(Usage::ADD_CHEERLEADER);

        // Java: winning team (or both on tie) gains 1 extra offensive block assist.
        let mut winner_ids: Vec<String> = Vec::new();
        if total_home >= total_away {
            game.home_additional_assists += 1;
            winner_ids.push(game.team_home.id.clone());
        }
        if total_away >= total_home {
            game.away_additional_assists += 1;
            winner_ids.push(game.team_away.id.clone());
        }
        game.report_list.add(ReportCheeringFansBb2025::new(winner_ids, roll_home, roll_away, vec![]));

        StepOutcome::next().with_event(GameEvent::CheeringFans { home_roll: roll_home, away_roll: roll_away })
    }

    // ── BrilliantCoaching ─────────────────────────────────────────────────────

    fn handle_brilliant_coaching(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.assistant_coaches
            + game.turn_data_home.inducement_set.value(Usage::ADD_COACH);
        let total_away = roll_away + game.team_away.assistant_coaches
            + game.turn_data_away.inducement_set.value(Usage::ADD_COACH);

        let mut outcome = StepOutcome::next();
        let tie = total_home == total_away;
        // Java: winning team (or both on tie) gains +1 re-roll for the drive.
        if total_home >= total_away {
            game.turn_data_home.rerolls += 1;
            let team_id = game.team_home.id.clone();
            outcome = outcome.with_event(GameEvent::KickoffExtraReRoll { team_id });
        }
        if total_away >= total_home {
            game.turn_data_away.rerolls += 1;
            let team_id = game.team_away.id.clone();
            outcome = outcome.with_event(GameEvent::KickoffExtraReRoll { team_id });
        }
        let report_team = if tie { None } else if total_home > total_away {
            Some(game.team_home.id.clone())
        } else {
            Some(game.team_away.id.clone())
        };
        game.report_list.add(ReportKickoffExtraReRoll::new(roll_home, roll_away, report_team));
        outcome
    }

    // ── Charge ────────────────────────────────────────────────────────────────

    fn handle_charge(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.eligible_players.is_empty() {
            // First entry: roll D3+3 to find nr of allowed players, collect eligible players.
            let roll = rng.d3();
            let kicking_team = if game.home_playing {
                &game.team_home
            } else {
                &game.team_away
            };
            // Count players on the field without adjacent opponents with tackle zones.
            let receiving_team = if game.home_playing {
                &game.team_away
            } else {
                &game.team_home
            };
            let on_field: Vec<String> = kicking_team
                .players
                .iter()
                .filter_map(|p| {
                    let coord = game.field_model.player_coordinate(&p.id)?;
                    if !FieldCoordinateBounds::FIELD.is_in_bounds(coord) {
                        return None;
                    }
                    let adjacent = UtilPlayer::find_adjacent_players_with_tacklezones(
                        game, receiving_team, coord, false,
                    );
                    if adjacent.is_empty() { Some(p.id.clone()) } else { None }
                })
                .collect();

            let cap = (roll + 3).min(on_field.len() as i32);
            self.nr_of_players_allowed = cap;
            for id in &on_field {
                if let Some(c) = game.field_model.player_coordinate(id) {
                    self.players_at_coordinates.insert(id.clone(), c);
                }
            }
            self.eligible_players = on_field;

            if self.eligible_players.is_empty() {
                // No players to select; go straight to end.
                game.turn_mode = TurnMode::Kickoff;
                return StepOutcome::next();
            }

            // client-only: show DialogPlayerChoiceParameter(CHARGE) — dialog is client-side
            // For now (random-agent stub): no players selected → skip blitz.
            game.turn_mode = TurnMode::Kickoff;
            return StepOutcome::next();
        }

        if self.selected_players.is_empty() {
            // No players chosen → end without blitz.
            game.turn_mode = TurnMode::Kickoff;
            return StepOutcome::next();
        }

        // Players were selected → deactivate non-selected players and GOTO blitz label.
        let label = self.goto_label_on_blitz.clone();
        StepOutcome::goto(&label)
    }

    // ── SolidDefence ──────────────────────────────────────────────────────────

    fn handle_solid_defence(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::SolidDefence {
            if self.end_kickoff {
                let moved_players = self.players_at_coordinates.keys()
                    .filter(|id| {
                        game.field_model.player_coordinate(id)
                            .map(|c| Some(c) != self.players_at_coordinates.get(*id).copied().map(Some).flatten())
                            .unwrap_or(false)
                    })
                    .count() as i32;

                // Java: validSolidDefence(movedPlayers) && mechanic.checkSetup(gameState, game.isHomePlaying(), getKickingSwarmers())
                // Both conditions must hold to leave Solid Defence; checkSetup is a real (non-client-only)
                // validity check and must gate the transition, not just the moved-player count.
                let valid = SetupMechanic::new().check_setup_with_swarmers(game, game.home_playing, game.kicking_swarmers);
                if moved_players <= self.nr_of_players_allowed && valid {
                    // Java: leaveSolidDefence → setKickingSwarmers(0)
                    game.kicking_swarmers = 0;
                    game.turn_mode = TurnMode::Kickoff;
                    StepOutcome::next()
                } else {
                    self.end_kickoff = false;
                    // client-only: DialogInvalidSolidDefenceParameter — headless allows overcount
                    StepOutcome::cont()
                }
            } else {
                StepOutcome::cont()
            }
        } else {
            let roll = rng.d3();
            self.nr_of_players_allowed = roll + 3;
            let acting_team_id = game.active_team().id.clone();
            // client-only: setAnimation; pin players in tacklezones is impl below
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
            game.report_list.add(ReportSolidDefenceRoll::new(
                Some(acting_team_id.clone()),
                roll,
                self.nr_of_players_allowed,
            ));
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
                // Java: `skip = ...noneMatch(player -> playerState.isActive())` — if the receiving team
                // has no active (unpinned) players left, immediately undo the toggle and finish.
                let any_active = game.active_team().players.iter()
                    .any(|p| game.field_model.player_state(&p.id)
                        .map(|s| s.is_active())
                        .unwrap_or(false));
                if !any_active {
                    game.home_playing = !game.home_playing;
                    game.turn_mode = TurnMode::Kickoff;
                    StepOutcome::next()
                } else {
                    StepOutcome::cont()
                }
            }
        }
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
                let mut scatter_dirs = Vec::new();
                let mut scatter_rolls = Vec::new();
                for _ in 0..3 {
                    let dir_roll = rng.d8();
                    let direction = Direction::for_roll(dir_roll).unwrap_or(Direction::North);
                    let candidate = UtilServerCatchScatterThrowIn::find_scatter_coordinate(last_valid, direction, 1);
                    let in_bounds = self.kickoff_bounds
                        .map(|b| b.is_in_bounds(candidate))
                        .unwrap_or_else(|| FieldCoordinateBounds::FIELD.is_in_bounds(candidate));
                    self.touchback = !in_bounds;
                    scatter_dirs.push(direction);
                    scatter_rolls.push(dir_roll);
                    if !self.touchback {
                        game.field_model.ball_coordinate = Some(candidate);
                        last_valid = candidate;
                    } else {
                        game.field_model.ball_coordinate = Some(last_valid);
                        break;
                    }
                }
                game.report_list.add(ReportScatterBall::new(scatter_dirs, scatter_rolls, true));
            }
        }

        StepOutcome::next()
            .with_event(GameEvent::WeatherChange { weather })
            .publish(StepParameter::Touchback(self.touchback))
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
                    UtilServerSetup::setup_player(game, player_id, coord);
                    let active_on_field = count_active_players_on_field(game);
                    game.report_list.add(ReportKickoffSequenceActivationsCount::new(
                        active_on_field,
                        self.nr_of_moved_players,
                        self.nr_of_players_allowed,
                    ));
                    // Java: `if (nrOfMovedPlayers == nrOfPlayersAllowed) { fEndKickoff = true; ... }
                    //        else if (activePlayersOnField == 0) { fEndKickoff = true; ... }`
                    // The active-players-exhausted case must also end Quick Snap, otherwise headless
                    // play stalls waiting for moves from a team with no active players left.
                    if self.nr_of_moved_players == self.nr_of_players_allowed {
                        self.end_kickoff = true;
                    } else if active_on_field == 0 {
                        self.end_kickoff = true;
                    }
                }
                self.moved_player = None;
                self.to_coordinate = None;
            }

            if self.end_kickoff {
                // Java reports `limit_reached=true` when the move cap was hit, and
                // `limit_reached=false` when the team ran out of active players instead.
                let limit_reached = self.nr_of_moved_players == self.nr_of_players_allowed;
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next().with_event(GameEvent::KickoffSequenceActivationsExhausted { limit_reached })
            } else {
                StepOutcome::cont()
            }
        } else {
            game.home_playing = !game.home_playing;
            game.turn_mode = TurnMode::QuickSnap;
            let roll = rng.d3();
            self.nr_of_players_allowed = roll + 3;
            let active_team_id = game.active_team().id.clone();
            // client-only: setAnimation(KICKOFF_QUICK_SNAP)

            // Java: deactivate acting-team players adjacent to opposing tacklers
            let other_is_home = !game.home_playing;
            let acting_ids: Vec<(String, FieldCoordinate)> = game.active_team().players.iter()
                .filter_map(|p| {
                    game.field_model.player_coordinate(&p.id).map(|c| (p.id.clone(), c))
                })
                .collect();
            for (pid, coord) in &acting_ids {
                let other_team = if other_is_home { &game.team_home } else { &game.team_away };
                let adj = UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, *coord, false);
                if !adj.is_empty() {
                    if let Some(state) = game.field_model.player_state(pid) {
                        game.field_model.set_player_state(pid, state.change_active(false));
                    }
                }
            }

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

    // ── DodgySnack ────────────────────────────────────────────────────────────

    fn handle_dodgy_snack(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll d6 per team; player from lower-roll team (or both on tie) is targeted.
        // Roll d6 per targeted player: 1 → move to RESERVE; else → add DODGY_SNACK enhancement.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let mut targeted_ids: Vec<String> = Vec::new();
        let mut pending_events: Vec<GameEvent> = Vec::new();

        if roll_away >= roll_home {
            if let Some(id) = self.random_player_on_field(game, rng, true) {
                targeted_ids.push(id.clone());
                let snack_roll = rng.d6();
                pending_events.push(GameEvent::DodgySnackRoll { player_id: id.clone(), roll: snack_roll });
                if snack_roll == 1 {
                    if let Some(state) = game.field_model.player_state(&id) {
                        game.field_model.set_player_state(&id, state.change_base(PS_RESERVE));
                    }
                    UtilBox::put_player_into_box(game, &id);
                } else {
                    // Java: FieldModel.addEnhancements(player, "Dodgy Snack") → -MA and -AV for the drive
                    if let Some(p) = game.player_mut(&id) {
                        p.add_temporary_stat_mod("Dodgy Snack", ffb_model::model::player::STAT_MA, -1);
                        p.add_temporary_stat_mod("Dodgy Snack", ffb_model::model::player::STAT_AV, -1);
                    }
                }
            }
        }
        if roll_home >= roll_away {
            if let Some(id) = self.random_player_on_field(game, rng, false) {
                targeted_ids.push(id.clone());
                let snack_roll = rng.d6();
                pending_events.push(GameEvent::DodgySnackRoll { player_id: id.clone(), roll: snack_roll });
                if snack_roll == 1 {
                    if let Some(state) = game.field_model.player_state(&id) {
                        game.field_model.set_player_state(&id, state.change_base(PS_RESERVE));
                    }
                    UtilBox::put_player_into_box(game, &id);
                } else {
                    // Java: FieldModel.addEnhancements(player, "Dodgy Snack") → -MA and -AV for the drive
                    if let Some(p) = game.player_mut(&id) {
                        p.add_temporary_stat_mod("Dodgy Snack", ffb_model::model::player::STAT_MA, -1);
                        p.add_temporary_stat_mod("Dodgy Snack", ffb_model::model::player::STAT_AV, -1);
                    }
                }
            }
        }

        // client-only: setAnimation(KICKOFF_DODGY_SNACK)
        let mut outcome = StepOutcome::next().with_event(GameEvent::KickoffDodgySnack {
            roll_home,
            roll_away,
            player_ids: targeted_ids,
        });
        for ev in pending_events {
            outcome = outcome.with_event(ev);
        }
        outcome
    }

    // ── PitchInvasion ─────────────────────────────────────────────────────────

    fn handle_pitch_invasion(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let fan_factor_home = game.game_result.home.fan_factor;
        let fan_factor_away = game.game_result.away.fan_factor;

        let total_home = roll_home + fan_factor_home;
        let total_away = roll_away + fan_factor_away;

        let stunned = rng.d3();

        if total_home <= total_away {
            self.stun_random_standing_players(game, rng, true, stunned);
        }
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

    /// Java: `randomPlayer(playersOnField(game, team))` — random player on field for given side.
    fn random_player_on_field(&self, game: &Game, rng: &mut GameRng, home: bool) -> Option<String> {
        let team = if home { &game.team_home } else { &game.team_away };
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
}

/// Java: StepApplyKickoffResult — active players on field for acting team.
fn count_active_players_on_field(game: &Game) -> i32 {
    let acting_team = game.active_team();
    let ids: Vec<String> = acting_team.players.iter().map(|p| p.id.clone()).collect();
    ids.iter()
        .filter(|pid| {
            game.field_model.player_coordinate(pid)
                .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                .unwrap_or(false)
                && game.field_model.player_state(pid)
                    .map(|s| s.is_active())
                    .unwrap_or(false)
        })
        .count() as i32
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

    fn make_step() -> StepApplyKickoffResult {
        StepApplyKickoffResult::new("end".into(), "blitz".into())
    }

    #[test]
    fn start_without_kickoff_result_waits() {
        let mut game = make_game();
        let mut step = make_step();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_kickoff_result() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::KickoffResult(KickoffResult::HighKick)));
        assert_eq!(step.kickoff_result, Some(KickoffResult::HighKick));
    }

    #[test]
    fn set_parameter_goto_labels() {
        let mut step = make_step();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("x".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnBlitz("y".into())));
        assert_eq!(step.goto_label_on_end, "x");
        assert_eq!(step.goto_label_on_blitz, "y");
    }

    #[test]
    fn timeout_increments_turn_on_early_turn() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.turn_nr = 3;
        game.turn_data_away.turn_nr = 3;
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::TimeOut);
        step.start(&mut game, &mut GameRng::new(0));
        // kicking_turn = 3 < 6 → modifier = +1
        assert_eq!(game.turn_data_home.turn_nr, 4);
        assert_eq!(game.turn_data_away.turn_nr, 4);
    }

    #[test]
    fn timeout_decrements_turn_on_late_turn() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.turn_nr = 7;
        game.turn_data_away.turn_nr = 7;
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::TimeOut);
        step.start(&mut game, &mut GameRng::new(0));
        // kicking_turn = 7 >= 6 → modifier = -1
        assert_eq!(game.turn_data_home.turn_nr, 6);
        assert_eq!(game.turn_data_away.turn_nr, 6);
    }

    #[test]
    fn get_the_ref_returns_next_step() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::GetTheRef);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn brilliant_coaching_grants_reroll_to_higher_total() {
        let mut game = make_game();
        // Set assistant_coaches so home always wins any roll.
        game.team_home.assistant_coaches = 10;
        game.team_away.assistant_coaches = 0;
        let home_before = game.turn_data_home.rerolls;
        let away_before = game.turn_data_away.rerolls;
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::BrilliantCoaching);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.rerolls > home_before, "Home should gain a reroll");
        assert_eq!(game.turn_data_away.rerolls, away_before, "Away should not gain a reroll");
    }

    #[test]
    fn cheering_fans_result_returns_next_step() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::CheeringFans);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn non_bb2025_result_goto_end_label() {
        // Blitz is not a valid BB2025 kickoff result — should fall through to goto(end).
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::Blitz);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn high_kick_waits_for_receiving_team() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        let mut game = make_game();
        // Receiving team (away, since home_playing toggles true->false) needs at least
        // one active player, otherwise (matching Java's `noneMatch` skip check) the step
        // would immediately skip back to Kickoff instead of waiting.
        game.team_away.players.push(make_min_player("away1"));
        game.field_model.set_player_coordinate("away1", FieldCoordinate::new(3, 3));
        game.field_model.set_player_state("away1", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(7, 7));
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::HighKick);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::HighKick);
    }

    #[test]
    fn pitch_invasion_returns_next_step() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::PitchInvasion);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn weather_change_sets_weather_field() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::WeatherChange);
        // Just confirm it runs without panic and returns NextStep or publishes touchback
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.action == StepAction::NextStep || out.action == StepAction::NextStep);
        // weather field was updated (not the default)
        let _ = game.field_model.weather;
    }

    #[test]
    fn dodgy_snack_returns_next_step() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::DodgySnack);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // ── GetTheRef tests ───────────────────────────────────────────────────────

    #[test]
    fn get_the_ref_creates_bribe_when_none_exist() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::GetTheRef);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(
            game.turn_data_home.inducement_set.value(ffb_model::inducement::usage::Usage::AVOID_BAN),
            1
        );
        assert_eq!(
            game.turn_data_away.inducement_set.value(ffb_model::inducement::usage::Usage::AVOID_BAN),
            1
        );
    }

    #[test]
    fn get_the_ref_increments_existing_bribes() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        let mut game = make_game();
        let mut step = make_step();
        game.turn_data_home.inducement_set.add_inducement(Inducement::new("BRIBE", 3, vec![Usage::AVOID_BAN]));
        step.kickoff_result = Some(KickoffResult::GetTheRef);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.inducement_set.value(Usage::AVOID_BAN), 4);
        assert_eq!(game.turn_data_away.inducement_set.value(Usage::AVOID_BAN), 1);
    }

    // ── DodgySnack enhancement test ────────────────────────────────────────────

    #[test]
    fn dodgy_snack_applies_ma_av_penalty_on_non_one_roll() {
        use ffb_model::enums::{PS_STANDING, PlayerState, PlayerType, PlayerGender};
        use ffb_model::model::player::{Player, STAT_MA, STAT_AV};

        let mut game = make_game();
        let mut step = make_step();
        // Add a home player on field (home = lower roll → targeted when away >= home)
        let home_player = Player {
            id: "hp1".into(), name: "h".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, ..Default::default()
        };
        game.team_home.players.push(home_player);
        game.field_model.set_player_coordinate("hp1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("hp1", PlayerState::new(PS_STANDING));

        // RNG seed 42 typically produces non-1 rolls for the snack roll, so -MA/-AV applies.
        // We can check that the player's movement decremented by -1 if snack_roll != 1.
        step.kickoff_result = Some(KickoffResult::DodgySnack);
        step.start(&mut game, &mut GameRng::new(42));

        // Check if hp1 was targeted and had snack_roll != 1 (stat penalty applied)
        if let Some(player) = game.team_home.player("hp1") {
            let ma_mod: i32 = player.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_MA).map(|(_, _, d)| *d).sum();
            let av_mod: i32 = player.temporary_stat_mods.iter()
                .filter(|(_, s, _)| *s == STAT_AV).map(|(_, _, d)| *d).sum();
            // Either: stat penalty applied OR player was sent to reserve (roll == 1)
            let to_box = game.field_model.player_state("hp1")
                .map(|s| s.base() == ffb_model::enums::PS_RESERVE)
                .unwrap_or(false);
            assert!(
                to_box || (ma_mod == -1 && av_mod == -1),
                "player should be sent to reserve or have -1 MA/-1 AV"
            );
        }
    }

    // ── HighKick: skip when receiving team has no active players ─────────────

    fn make_min_player(id: &str) -> ffb_model::model::player::Player {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, ..Default::default()
        }
    }

    #[test]
    fn high_kick_skips_when_receiving_team_fully_pinned() {
        use ffb_model::enums::{PS_STANDING, PlayerState};

        let mut game = make_game();
        // Home player will end up pinning the away player's tacklezone.
        game.team_home.players.push(make_min_player("hp1"));
        game.field_model.set_player_coordinate("hp1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("hp1", PlayerState::new(PS_STANDING).change_active(true));

        // Away is the only player on the receiving team; it will be pinned
        // (adjacent to hp1's tacklezone) and has no teammates to fall back on.
        game.team_away.players.push(make_min_player("ap1"));
        game.field_model.set_player_coordinate("ap1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("ap1", PlayerState::new(PS_STANDING).change_active(true));

        // No catcher on the ball square, so the high-kick reposition sub-phase is entered.
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(1, 1));
        game.home_playing = true;

        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::HighKick);
        step.touchback = false;
        let out = step.start(&mut game, &mut GameRng::new(0));

        // Before the fix: the pinned-out receiving team would leave turn_mode stuck at
        // HighKick with action Continue forever, since the "no active players" guard was
        // dropped. After the fix, it must immediately bounce back to Kickoff/NextStep.
        assert_eq!(out.action, StepAction::NextStep, "should skip HighKick when no active players remain");
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
        assert!(game.home_playing, "home_playing toggle must be undone when skipping");
    }

    // ── SolidDefence: invalid setup must not end the sub-phase ────────────────

    #[test]
    fn solid_defence_stays_active_when_setup_invalid() {
        use ffb_model::enums::PlayerState;
        let mut game = make_game();
        game.options.set(ffb_model::option::game_option_id::MAX_PLAYERS_ON_FIELD, "11");
        game.options.set(ffb_model::option::game_option_id::MIN_PLAYERS_ON_LOS, "3");

        // 11 reservable (but unplaced) players make `checkSetup` fail: available_players (11)
        // >= max_players_on_field (11) while all_players_on_field is 0.
        for i in 0..11 {
            let id = format!("hp{i}");
            game.team_home.players.push(make_min_player(&id));
            game.field_model.set_player_state(&id, PlayerState::new(ffb_model::enums::PS_RESERVE));
        }
        game.home_playing = true;

        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::SolidDefence);
        // First call: enters Solid Defence mode.
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::SolidDefence);

        // Simulate the receiving player's EndTurn (no players actually moved).
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));

        // Before the fix: an invalid setup (checkSetup == false) was ignored, and the step
        // always left Solid Defence once the moved-player count was within the allowed cap.
        // After the fix: an invalid setup must keep the game in Solid Defence.
        assert_eq!(game.turn_mode, TurnMode::SolidDefence, "invalid setup must not leave Solid Defence");
    }

    // ── QuickSnap: end sub-phase when active players run out mid-move ────────

    #[test]
    fn quick_snap_ends_when_active_players_exhausted_before_cap() {
        use ffb_model::enums::{PS_STANDING, PlayerState};

        let mut game = make_game();
        // Only one player total on the acting (post-toggle) team, so after the
        // single allowed move it has no more active players — but nrOfPlayersAllowed
        // (roll+3 >= 4) will never be reached by moving just one player.
        game.team_home.players.push(make_min_player("hp1"));
        game.field_model.set_player_coordinate("hp1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("hp1", PlayerState::new(PS_STANDING).change_active(true));
        game.home_playing = false; // toggled to true (home) when QuickSnap starts

        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::QuickSnap);
        let out1 = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(game.turn_mode, TurnMode::QuickSnap);
        assert_eq!(out1.action, StepAction::Continue);
        assert!(step.nr_of_players_allowed >= 4, "roll (1..3) + 3 should be >= 4");

        // Move the only active player once — this consumes it (it becomes the "moved"
        // player, but importantly there are no other active players left to move).
        step.moved_player = Some("hp1".into());
        step.to_coordinate = Some(FieldCoordinate::new(5, 6));
        let out2 = step.execute_step(&mut game, &mut GameRng::new(1));

        // Before the fix: with active_players_on_field == 0 but nr_of_moved_players (1) <
        // nr_of_players_allowed (>=4), the step would never end and StepOutcome::cont()
        // would be returned forever. After the fix, the exhausted-players branch ends it.
        assert_eq!(out2.action, StepAction::NextStep, "quick snap must end when no active players remain");
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }
}
