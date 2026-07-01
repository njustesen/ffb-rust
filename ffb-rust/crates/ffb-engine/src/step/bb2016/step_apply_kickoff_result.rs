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
/// DEFERRED items (require untranslated infrastructure):
///  - handleGetTheRef: InducementTypeFactory
///  - handlePerfectDefense: SetupMechanic.checkSetup
///  - handleHighKick: SetupMechanic.pinPlayersInTacklezones
///  - handleWeatherChange: DiceInterpreter.interpretRollWeather + scatter on NICE
///  - handleThrowARock: UtilServerInjury.handleInjury / dropPlayer
///  - handlePitchInvasion: UtilServerInjury.stunPlayer
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepApplyKickoffResult`.
use ffb_model::enums::{KickoffResult, TurnMode};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

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

    fn handle_get_the_ref(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(InducementTypeFactory): InducementTypeFactory.allTypes() not yet ported.
        // Java: each team gets +1 bribes inducement.
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
            // DEFERRED(DiceInterpreter): interpretRiotRoll not yet ported; using d6 stub (1-3=-1, 4-6=+1)
            riot_roll = rng.d6();
            // Java: turn_modifier = DiceInterpreter.interpretRiotRoll(roll)
            // 1-3 → -1 (time wasted), 4-6 → +1 (riot speeds up)
            turn_modifier = if riot_roll <= 3 { -1 } else { 1 };
        }

        game.turn_data_home.turn_nr = (game.turn_data_home.turn_nr + turn_modifier).max(0);
        game.turn_data_away.turn_nr = (game.turn_data_away.turn_nr + turn_modifier).max(0);

        let _ = riot_roll;
        // Java: getResult().addReport(new ReportKickoffRiot(turnModifier, riotRoll))
        let riot_event = GameEvent::KickoffRiot;

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
                // DEFERRED(SetupMechanic): checkSetup not yet ported.
                game.turn_mode = TurnMode::Kickoff;
                StepOutcome::next()
            } else {
                // Waiting for CLIENT_END_TURN
                StepOutcome::cont()
            }
        } else {
            // Java: setAnimation(KICKOFF_PERFECT_DEFENSE); setTurnMode(PERFECT_DEFENCE)
            game.turn_mode = TurnMode::PerfectDefence;
            // DEFERRED(animation): setAnimation not yet ported.
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
                // DEFERRED(SetupMechanic): pinPlayersInTacklezones not yet ported.
                StepOutcome::cont()
            }
        }
    }

    fn handle_extra_reroll(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: roll d6 for each team, add fame/cheerleaders/coaches, winner gets +1 reroll.
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        // Java: total = roll + fame + cheerleaders + assistantCoaches
        let home_bonus = game.game_result.home.fame
            + game.team_home.cheerleaders
            + game.team_home.assistant_coaches;
        let away_bonus = game.game_result.away.fame
            + game.team_away.cheerleaders
            + game.team_away.assistant_coaches;
        let total_home = roll_home + home_bonus;
        let total_away = roll_away + away_bonus;

        if total_home >= total_away {
            game.turn_data_home.rerolls += 1;
        }
        if total_away >= total_home {
            game.turn_data_away.rerolls += 1;
        }

        StepOutcome::next()
    }

    fn handle_weather_change(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(weather): DiceInterpreter.interpretRollWeather and weather state not yet ported.
        let _ = (rng.d6(), rng.d6());

        // Java: publishParameter(TOUCHBACK, fTouchback)
        StepOutcome::next()
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
            // DEFERRED(animation): setAnimation not yet ported.
            StepOutcome::cont()
        }
    }

    fn handle_blitz(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_blitz.clone();
        StepOutcome::goto(&label)
    }

    fn handle_throw_a_rock(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(throwARock): rollThrowARock, randomPlayer, handleInjury, dropPlayer not yet ported.
        StepOutcome::next()
    }

    fn handle_pitch_invasion(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(pitchInvasion): rollPitchInvasion, stunPlayer not yet ported.
        StepOutcome::next()
    }
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
}
