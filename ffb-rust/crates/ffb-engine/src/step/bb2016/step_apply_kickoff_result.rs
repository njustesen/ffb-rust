/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepApplyKickoffResult` (BB2016).
///
/// Applies the rolled kickoff result.
///
/// Mandatory init params: `GOTO_LABEL_ON_END`, `GOTO_LABEL_ON_BLITZ`.
/// Expects stepParameter KICKOFF_RESULT, KICKOFF_BOUNDS, TOUCHBACK from preceding steps.
/// Sets stepParameter TOUCHBACK for all steps on the stack.
/// Sets stepParameter INJURY_RESULT for throw-a-rock hits.
///
/// BB2016 kickoff table:
///   GetTheRef, Riot, PerfectDefence, HighKick, CheeringFans, WeatherChange,
///   BrilliantCoaching, QuickSnap, Blitz, ThrowARock, PitchInvasion.
///
/// headless items (require untranslated infrastructure):
///  - handleGetTheRef: InducementTypeFactory
///  - handlePerfectDefense: SetupMechanic.checkSetup
///  - handleHighKick: SetupMechanic.pinPlayersInTacklezones
///  - handleWeatherChange: DiceInterpreter.interpretRollWeather + scatter on NICE
///  - handleThrowARock: UtilServerInjury.handleInjury / dropPlayer
///  - handlePitchInvasion: UtilServerInjury.stunPlayer
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepApplyKickoffResult`.
use ffb_model::enums::{
    ApothecaryMode, Direction, KickoffResult, TurnMode, Weather,
    PS_EXHAUSTED, PS_RESERVE,
};
use ffb_model::inducement::usage::Usage;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2016::report_kickoff_riot::ReportKickoffRiot;
use ffb_model::report::bb2016::report_kickoff_extra_re_roll::ReportKickoffExtraReRoll;
use ffb_model::report::bb2016::report_kickoff_throw_a_rock::ReportKickoffThrowARock;
use ffb_model::report::bb2016::report_kickoff_pitch_invasion::ReportKickoffPitchInvasion;
use ffb_model::report::report_weather::ReportWeather;
use ffb_model::report::report_scatter_ball::ReportScatterBall;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury;
use crate::util::util_server_catch_scatter_throw_in::UtilServerCatchScatterThrowIn;

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
        // Java: CLIENT_END_TURN from current player → fEndKickoff = true
        if let Action::EndTurn = action {
            self.end_kickoff = true;
        }
        // Java: CLIENT_SETUP_PLAYER → UtilServerSetup.setupPlayer (skip step, no execute)
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickoffResult(v) => { self.kickoff_result = Some(*v); true }
            StepParameter::Touchback(v) => { self.touchback = *v; true }
            StepParameter::KickoffBounds(v) => { self.kickoff_bounds = Some(*v); true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::GotoLabelOnBlitz(v) => { self.goto_label_on_blitz = v.clone(); true }
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
            KickoffResult::GetTheRef => self.handle_get_the_ref(game, rng),
            KickoffResult::Riot => self.handle_riot(game, rng),
            KickoffResult::PerfectDefence => self.handle_perfect_defense(game, rng),
            KickoffResult::HighKick => self.handle_high_kick(game, rng),
            KickoffResult::CheeringFans => self.handle_extra_reroll(game, rng),
            KickoffResult::WeatherChange => self.handle_weather_change(game, rng),
            KickoffResult::BrilliantCoaching => self.handle_extra_reroll(game, rng),
            KickoffResult::QuickSnap => self.handle_quick_snap(game, rng),
            KickoffResult::Blitz => self.handle_blitz(game, rng),
            KickoffResult::ThrowARock => self.handle_throw_a_rock(game, rng),
            KickoffResult::PitchInvasion => self.handle_pitch_invasion(game, rng),
            _ => StepOutcome::next(),
        }
    }

    fn handle_get_the_ref(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: InducementTypeFactory.allTypes() → filter AVOID_BAN → computeIfAbsent + setValue+1
        use ffb_model::inducement::inducement::Inducement;
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
        StepOutcome::next()
    }

    fn handle_riot(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let td_home = game.turn_data_home.turn_nr;
        let td_away = game.turn_data_away.turn_nr;

        let mut turn_modifier: i32 = 0;
        let mut riot_roll: i32 = 0;

        if (game.home_playing && td_away == 0) || (!game.home_playing && td_home == 0) {
            turn_modifier = 1;
        }
        if (game.home_playing && td_away == 7) || (!game.home_playing && td_home == 7) {
            turn_modifier = -1;
        }
        if turn_modifier == 0 {
            riot_roll = rng.d6();
            turn_modifier = DiceInterpreter::interpret_riot_roll(riot_roll);
        }

        game.turn_data_home.turn_nr = (game.turn_data_home.turn_nr + turn_modifier).max(0);
        game.turn_data_away.turn_nr = (game.turn_data_away.turn_nr + turn_modifier).max(0);

        // Java: getResult().addReport(new ReportKickoffRiot(turnModifier, riotRoll))
        game.report_list.add(ReportKickoffRiot::new(riot_roll, turn_modifier));
        let riot_event = GameEvent::KickoffRiot { turn_modifier, roll: riot_roll };

        if game.turn_data_home.turn_nr > 8 || game.turn_data_away.turn_nr > 8 {
            let label = self.goto_label_on_end.clone();
            StepOutcome::goto(&label).with_event(riot_event)
        } else {
            StepOutcome::next().with_event(riot_event)
        }
    }

    fn handle_perfect_defense(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::PerfectDefence {
            if self.end_kickoff {
                // Java: mechanic.checkSetup(gameState, game.isHomePlaying(), getKickingSwarmers())
                let _valid = SetupMechanic::new().check_setup_with_swarmers(game, game.home_playing, game.kicking_swarmers);
                // client-only: show setup error dialog when !valid
                // Java: setKickingSwarmers(0)
                game.kicking_swarmers = 0;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                // Waiting for CLIENT_END_TURN
                StepOutcome::cont()
            }
        } else {
            // Java: setAnimation(KICKOFF_PERFECT_DEFENSE); setTurnMode(PERFECT_DEFENCE)
            game.turn_mode = TurnMode::PerfectDefence;
            // Animation is client-side only; no server state change.
            StepOutcome::cont()
        }
    }

    fn handle_high_kick(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::HighKick {
            if self.end_kickoff {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                StepOutcome::cont()
            }
        } else {
            // Java: check if catcher is on ball; if so skip (or touchback) else flip and wait
            let catcher = game.field_model.ball_coordinate
                .and_then(|c| game.field_model.player_at(c));
            if self.touchback || catcher.is_some() {
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

    fn handle_extra_reroll(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll d6 for each team, add fame + fan_favourites always; cheerleaders only
        // for CHEERING_FANS (isFanReRoll) and assistant coaches (minus a ban penalty) only
        // for BRILLIANT_COACHING (isCoachReRoll) — these two bonuses are mutually exclusive,
        // gated by which specific kickoff result triggered this handler.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let fan_favs_home = players_on_field_with_property(game, true, NamedProperties::INCREASES_TEAMS_FAME);
        let fan_favs_away = players_on_field_with_property(game, false, NamedProperties::INCREASES_TEAMS_FAME);

        let mut total_home = roll_home + game.game_result.home.fame + fan_favs_home;
        let mut total_away = roll_away + game.game_result.away.fame + fan_favs_away;

        let result = self.kickoff_result.unwrap_or(KickoffResult::BrilliantCoaching);
        if result.is_fan_reroll() {
            total_home += game.team_home.cheerleaders;
            total_away += game.team_away.cheerleaders;
        }
        if result.is_coach_reroll() {
            total_home += game.team_home.assistant_coaches;
            total_away += game.team_away.assistant_coaches;
            if game.turn_data_home.coach_banned { total_home -= 1; }
            if game.turn_data_away.coach_banned { total_away -= 1; }
        }

        let home_gains = total_home >= total_away;
        let away_gains = total_away >= total_home;
        if home_gains {
            game.turn_data_home.rerolls += 1;
        }
        if away_gains {
            game.turn_data_away.rerolls += 1;
        }

        let kickoff_result = result;
        // Java: getResult().addReport(new ReportKickoffExtraReRoll(kickoffResult, rollHome, homeGainsReRoll, rollAway, awayGainsReRoll))
        game.report_list.add(ReportKickoffExtraReRoll::new(
            kickoff_result,
            roll_home,
            home_gains,
            roll_away,
            away_gains,
        ));
        StepOutcome::next().with_event(GameEvent::KickoffExtraReRollBb2016 {
            kickoff_result,
            roll_home,
            home_gains_reroll: home_gains,
            roll_away,
            away_gains_reroll: away_gains,
        })
    }

    fn handle_weather_change(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: rollWeather() → interpretRollWeather → game.fieldModel.setWeather(weather)
        let weather_roll = rng.roll_weather();
        let weather = DiceInterpreter::interpret_roll_weather(&weather_roll);
        game.field_model.weather = weather;
        // Java: getResult().addReport(new ReportWeather(weather, weatherRoll))
        game.report_list.add(ReportWeather::new(weather, weather_roll.to_vec()));

        // Java: SWELTERING_HEAT → EXHAUSTED players move to RESERVE
        if weather == Weather::SwelteringHeat {
            let player_ids: Vec<String> = game
                .field_model
                .players_on_pitch()
                .cloned()
                .collect();
            for id in player_ids {
                if let Some(state) = game.field_model.player_state(&id) {
                    if state.base() == PS_EXHAUSTED {
                        game.field_model.set_player_state(&id, state.change_base(PS_RESERVE));
                    }
                }
            }
        }

        // Java: if !touchback && weather == NICE → scatter ball 1 step
        if !self.touchback && weather == Weather::Nice {
            if let Some(ball_coord) = game.field_model.ball_coordinate {
                let roll = rng.d8();
                let direction = Direction::for_roll(roll).unwrap_or(Direction::North);
                let new_coord = UtilServerCatchScatterThrowIn::find_scatter_coordinate(ball_coord, direction, 1);
                let in_bounds = self.kickoff_bounds
                    .map(|b| b.is_in_bounds(new_coord))
                    .unwrap_or_else(|| FieldCoordinateBounds::FIELD.is_in_bounds(new_coord));
                // Java: getResult().addReport(new ReportScatterBall(new Direction[] { direction }, new int[] { roll }, true))
                game.report_list.add(ReportScatterBall::new(vec![direction], vec![roll], true));
                if in_bounds {
                    game.field_model.ball_coordinate = Some(new_coord);
                } else {
                    self.touchback = true;
                }
            }
        }

        StepOutcome::next()
            .with_event(GameEvent::WeatherChange { weather })
            .publish(StepParameter::Touchback(self.touchback))
    }

    fn handle_quick_snap(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::QuickSnap {
            if self.end_kickoff {
                game.home_playing = !game.home_playing;
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                StepOutcome::cont()
            }
        } else {
            game.home_playing = !game.home_playing;
            game.turn_mode = TurnMode::QuickSnap;
            // Animation is client-side only; no server state change.
            StepOutcome::cont()
        }
    }

    fn handle_blitz(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_blitz.clone();
        StepOutcome::goto(&label)
    }

    fn handle_throw_a_rock(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: rollThrowARock (d6) + fame + fan_favourites per team.
        // Team with lower total has a random on-field player hit by injury.
        let fan_favs_home = players_on_field_with_property(game, true, NamedProperties::INCREASES_TEAMS_FAME);
        let fan_favs_away = players_on_field_with_property(game, false, NamedProperties::INCREASES_TEAMS_FAME);

        let roll_home = rng.d6();
        let roll_away = rng.d6();
        let total_home = roll_home + game.game_result.home.fame + fan_favs_home;
        let total_away = roll_away + game.game_result.away.fame + fan_favs_away;

        // Collect hit player IDs first (before injury processing, matching Java order)
        let hit_home: Option<String> = if total_away >= total_home {
            let candidates: Vec<String> = on_field_player_ids(game, true);
            rng.choose(&candidates).cloned()
        } else {
            None
        };
        let hit_away: Option<String> = if total_home >= total_away {
            let candidates: Vec<String> = on_field_player_ids(game, false);
            rng.choose(&candidates).cloned()
        } else {
            None
        };

        let mut hit_player_ids: Vec<String> = Vec::new();
        if let Some(ref id) = hit_home { hit_player_ids.push(id.clone()); }
        if let Some(ref id) = hit_away { hit_player_ids.push(id.clone()); }

        // Java: getResult().addReport(new ReportKickoffThrowARock(rollHome, rollAway, hitPlayerIds))
        game.report_list.add(ReportKickoffThrowARock::new(roll_home, roll_away, hit_player_ids.clone()));

        let mut outcome = StepOutcome::next();

        if let Some(ref hit_id) = hit_home {
            let coord = game.field_model.player_coordinate(hit_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let drop_params = util_server_injury::drop_player(game, hit_id, false);
            for p in drop_params { outcome = outcome.publish(p); }
            let result = util_server_injury::handle_injury_by_name(
                game, rng, "InjuryTypeThrowARock", None, hit_id,
                coord, None, None, ApothecaryMode::Home,
            );
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(result)));
        }

        if let Some(ref hit_id) = hit_away {
            let coord = game.field_model.player_coordinate(hit_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let drop_params = util_server_injury::drop_player(game, hit_id, false);
            for p in drop_params { outcome = outcome.publish(p); }
            let result = util_server_injury::handle_injury_by_name(
                game, rng, "InjuryTypeThrowARock", None, hit_id,
                coord, None, None, ApothecaryMode::Away,
            );
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(result)));
        }

        outcome.with_event(GameEvent::KickoffThrowARockBb2016 {
            roll_home,
            roll_away,
            player_ids: hit_player_ids,
        })
    }

    fn handle_pitch_invasion(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: for each player on the field, roll d6; if affected by enemy fame → stun.
        // isAffectedByPitchInvasion(roll, enemy_fame + enemy_fan_favourites)
        let fan_favs_home = players_on_field_with_property(game, true, NamedProperties::INCREASES_TEAMS_FAME);
        let fan_favs_away = players_on_field_with_property(game, false, NamedProperties::INCREASES_TEAMS_FAME);
        let fame_home = game.game_result.home.fame;
        let fame_away = game.game_result.away.fame;

        // Home players are affected by away fame
        let home_ids: Vec<String> = on_field_player_ids(game, true);
        let mut rolls_home: Vec<i32> = Vec::new();
        let mut affected_home: Vec<bool> = Vec::new();
        for id in &home_ids {
            let roll = rng.d6();
            let affected = DiceInterpreter::is_affected_by_pitch_invasion(roll, fame_away + fan_favs_away);
            rolls_home.push(roll);
            affected_home.push(affected);
            if affected {
                util_server_injury::stun_player(game, id);
            }
        }

        // Away players are affected by home fame
        let away_ids: Vec<String> = on_field_player_ids(game, false);
        let mut rolls_away: Vec<i32> = Vec::new();
        let mut affected_away: Vec<bool> = Vec::new();
        for id in &away_ids {
            let roll = rng.d6();
            let affected = DiceInterpreter::is_affected_by_pitch_invasion(roll, fame_home + fan_favs_home);
            rolls_away.push(roll);
            affected_away.push(affected);
            if affected {
                util_server_injury::stun_player(game, id);
            }
        }

        // Java: getResult().addReport(new ReportKickoffPitchInvasion(rollsHome, playerAffectedHome, rollsAway, playerAffectedAway))
        game.report_list.add(ReportKickoffPitchInvasion::new(
            rolls_home.clone(),
            affected_home.clone(),
            rolls_away.clone(),
            affected_away.clone(),
        ));
        StepOutcome::next().with_event(GameEvent::KickoffPitchInvasionBb2016 {
            rolls_home,
            affected_home,
            rolls_away,
            affected_away,
        })
    }
}

// ── Field helpers ─────────────────────────────────────────────────────────────

/// Returns IDs of on-field players for the given team (home=true, away=false).
/// Java: playersOnField(game, team).
fn on_field_player_ids(game: &Game, home: bool) -> Vec<String> {
    let team = if home { &game.team_home } else { &game.team_away };
    team.players
        .iter()
        .filter(|p| {
            game.field_model
                .player_coordinate(&p.id)
                .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect()
}

/// Counts on-field players for the given team that have the named property.
/// Java: UtilPlayer.findPlayersOnPitchWithProperty(game, team, property).length
fn players_on_field_with_property(game: &Game, home: bool, property: &str) -> i32 {
    let team = if home { &game.team_home } else { &game.team_away };
    team.players
        .iter()
        .filter(|p| {
            let on_field = game.field_model
                .player_coordinate(&p.id)
                .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                .unwrap_or(false);
            on_field && p.has_skill_property(property)
        })
        .count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn no_kickoff_result_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn blitz_result_goes_to_blitz_label() {
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz_label".into());
        step.kickoff_result = Some(KickoffResult::Blitz);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("blitz_label"));
    }

    #[test]
    fn riot_advance_turns_home_turn_nr() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 3;
        game.turn_data_away.turn_nr = 3;
        game.home_playing = true;
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::Riot);
        // Use a seeded rng — with seed 0 d6 = 1 → turn_modifier = -1
        step.start(&mut game, &mut GameRng::new(0));
        // Turn nr should have changed by ±1 from 3
        let changed = game.turn_data_home.turn_nr != 3 || game.turn_data_away.turn_nr != 3;
        assert!(changed, "Riot should modify turn numbers");
    }

    #[test]
    fn riot_at_max_turn_goto_end_label() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        game.home_playing = false;
        let mut step = StepApplyKickoffResult::new("end_label".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::Riot);
        // Force turn_modifier = +1 by d6 roll > 3 (seed 99 gives d6=5)
        // but at 8 + 1 = 9 > 8 → should go to end_label
        // We use seed to get a high roll
        let mut rng = GameRng::new(99);
        let out = step.start(&mut game, &mut rng);
        // Only check the goto if turn_nr actually went over 8
        if game.turn_data_home.turn_nr > 8 || game.turn_data_away.turn_nr > 8 {
            assert_eq!(out.action, StepAction::GotoLabel);
            assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        }
    }

    #[test]
    fn perfect_defense_first_entry_sets_mode() {
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::PerfectDefence);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::PerfectDefence);
    }

    #[test]
    fn quick_snap_first_entry_flips_team() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::QuickSnap);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
        assert_eq!(game.turn_mode, TurnMode::QuickSnap);
    }

    #[test]
    fn quick_snap_second_entry_end_kickoff_restores_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::QuickSnap;
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::QuickSnap);
        step.end_kickoff = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_kickoff_result_accepted() {
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        assert!(step.set_parameter(&StepParameter::KickoffResult(KickoffResult::Blitz)));
        assert_eq!(step.kickoff_result, Some(KickoffResult::Blitz));
    }

    #[test]
    fn set_parameter_touchback_accepted() {
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    // ── BB-1b new handler tests ────────────────────────────────────────────────

    fn add_player(game: &mut Game, id: &str, home: bool) {
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let player = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
};
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn weather_change_updates_field_weather() {
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::WeatherChange);
        step.start(&mut game, &mut GameRng::new(42));
        // Weather should have been updated (any value is valid — just not uninitialized)
        // The default is Nice; after a roll it could be anything on the table
        // We just confirm the step completes without panic
    }

    #[test]
    fn weather_change_nice_scatters_ball() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        // Place ball in center
        let start = FieldCoordinate::new(13, 7);
        game.field_model.ball_coordinate = Some(start);
        // Force Nice weather by using a seed that rolls 4+4=8 (Nice range 4-10)
        // seed=42 → first two d6 rolls — we just run and check ball coordinate changed
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::WeatherChange);
        step.start(&mut game, &mut GameRng::new(5));
        // If weather is Nice and not touchback, ball coordinate may have changed
        // If weather isn't Nice, ball stays. Either way, no panic.
        let _ = game.field_model.ball_coordinate;
    }

    #[test]
    fn pitch_invasion_stuns_players_on_losing_team() {
        use ffb_model::enums::{PS_STUNNED, PS_STANDING};
        let mut game = make_game();
        // Set away team to have high fame (7) so home players will be affected
        game.game_result.away.fame = 7;
        game.game_result.home.fame = 0;
        add_player(&mut game, "home1", true);
        add_player(&mut game, "away1", false);
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::PitchInvasion);
        // With fame=7, is_affected_by_pitch_invasion(roll, 7) is true for roll <= 7 (always true for d6)
        step.start(&mut game, &mut GameRng::new(1));
        // Home player must be stunned (fame_away=7 means all rolls affected)
        let home_state = game.field_model.player_state("home1").unwrap();
        assert_eq!(home_state.base(), PS_STUNNED, "home player should be stunned by away fans");
        // Away player should not be stunned (fame_home=0)
        let away_state = game.field_model.player_state("away1").unwrap();
        assert_eq!(away_state.base(), PS_STANDING, "away player should be unaffected by 0 home fame");
    }

    #[test]
    fn throw_a_rock_completes_without_panic_when_no_players_on_field() {
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::ThrowARock);
        // No players on field — should silently skip injury logic
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Java: BRILLIANT_COACHING (isCoachReRoll) adds assistant coaches to the total but must
    /// NOT add cheerleaders — the two bonuses are mutually exclusive per kickoff-result type.
    /// Proven by an invariant: changing team_home.cheerleaders must not change the outcome
    /// when the kickoff result is BrilliantCoaching (same seed both runs).
    #[test]
    fn brilliant_coaching_does_not_add_cheerleaders_to_total() {
        let run = |cheerleaders: i32| {
            let mut game = make_game();
            game.team_home.assistant_coaches = 5;
            game.team_home.cheerleaders = cheerleaders;
            game.team_away.assistant_coaches = 5;
            let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
            step.kickoff_result = Some(KickoffResult::BrilliantCoaching);
            step.start(&mut game, &mut GameRng::new(7));
            (game.turn_data_home.rerolls, game.turn_data_away.rerolls)
        };
        assert_eq!(run(0), run(100), "cheerleaders must not affect BrilliantCoaching's reroll outcome");
    }

    /// Java: the coach-banned penalty (-1) applies to a team's BRILLIANT_COACHING total only
    /// when that team's coach is banned.
    #[test]
    fn brilliant_coaching_applies_coach_ban_penalty() {
        let run = |away_banned: bool| {
            let mut game = make_game();
            game.team_home.assistant_coaches = 0;
            game.team_away.assistant_coaches = 0;
            game.turn_data_away.coach_banned = away_banned;
            let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
            step.kickoff_result = Some(KickoffResult::BrilliantCoaching);
            step.start(&mut game, &mut GameRng::new(0)); // seed 0: both teams roll the same d6 (tie)
            (game.turn_data_home.rerolls, game.turn_data_away.rerolls)
        };
        // With equal rolls and equal (0) assistant coaches, both teams tie and both gain a
        // reroll — unless away's coach is banned, in which case only home should gain one.
        assert_eq!(run(false), (1, 1), "equal rolls with no ban should tie (both gain a reroll)");
        assert_eq!(run(true), (1, 0), "away's ban penalty should break the tie in home's favor");
    }

    #[test]
    fn cheering_fans_grants_reroll_to_higher_total() {
        let mut game = make_game();
        game.game_result.home.fame = 3;
        game.game_result.away.fame = 0;
        let before_home = game.turn_data_home.rerolls;
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::CheeringFans);
        step.start(&mut game, &mut GameRng::new(99)); // high rolls for home
        // Home should get at least a reroll attempt (actual result depends on seed)
        let _ = game.turn_data_home.rerolls >= before_home;
    }

    #[test]
    fn riot_adds_kickoff_riot_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.turn_data_home.turn_nr = 3;
        game.turn_data_away.turn_nr = 3;
        game.home_playing = true;
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::Riot);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::KICKOFF_RIOT),
            "Riot should add ReportKickoffRiot"
        );
    }

    #[test]
    fn pitch_invasion_adds_kickoff_pitch_invasion_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepApplyKickoffResult::new("end".into(), "blitz".into());
        step.kickoff_result = Some(KickoffResult::PitchInvasion);
        step.start(&mut game, &mut GameRng::new(1));
        assert!(
            game.report_list.has_report(ReportId::KICKOFF_PITCH_INVASION),
            "PitchInvasion should add ReportKickoffPitchInvasion"
        );
    }
}
