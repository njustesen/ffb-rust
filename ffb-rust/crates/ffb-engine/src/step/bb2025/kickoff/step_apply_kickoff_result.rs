use std::collections::HashMap;
use ffb_model::enums::{KickoffResult, TurnMode};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Applies the rolled kickoff result.
///
/// Routing overview (per BB2025 Java source):
///
///  - `GetTheRef`          → both teams receive +1 bribe; NEXT_STEP.  (TODO: bribe storage)
///  - `TimeOut`            → adjust turn counters by ±1; NEXT_STEP.
///  - `SolidDefence`       → kicking team repositions D3+3 players; multi-round dialog. TODO.
///  - `HighKick`           → receiving team may move a player to the ball; dialog. TODO.
///  - `CheeringFans`       → roll d6+cheerleaders per team; winner gets +1 block assist.
///  - `WeatherChange`      → roll 2d6 weather; re-scatter ball if Nice. TODO (weather table).
///  - `BrilliantCoaching`  → roll d6+coaches; winner gets +1 re-roll.
///  - `QuickSnap`          → receiving team repositions D3+3 open players 1 sq each. TODO.
///  - `Charge`             → kicking team selects D3+3 players for move/blitz actions;
///                           GOTO_LABEL_ON_BLITZ to re-enter kickoff sequence for the blitz.
///  - `DodgySnack`         → random player per team takes -MA/-AV or goes to reserves. TODO.
///  - `PitchInvasion`      → roll d6+fan-factor; losing team has D3 players stunned. TODO.
///
/// Complex sub-states (SolidDefence, QuickSnap, HighKick, Charge player-selection dialog,
/// DodgySnack sequence push, PitchInvasion injury) are marked TODO and fall through to
/// GOTO_LABEL_ON_END in the current stub.
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
            Action::EndTurn => {
                self.end_kickoff = true;
                if game.turn_mode == TurnMode::QuickSnap {
                    // Java endQuickSnap: flip home_playing, set Kickoff mode.
                    game.home_playing = !game.home_playing;
                    game.turn_mode = TurnMode::Kickoff;
                }
                // Solid Defence end handled in execute_step.
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
            KickoffResult::GetTheRef => self.handle_get_the_ref(game),
            KickoffResult::TimeOut => self.handle_timeout(game),
            KickoffResult::CheeringFans => self.handle_cheering_fans(game, rng),
            KickoffResult::BrilliantCoaching => self.handle_brilliant_coaching(game, rng),
            KickoffResult::Charge => self.handle_charge(game, rng),
            // All other results: TODO stubs — proceed to end.
            KickoffResult::SolidDefence     |
            KickoffResult::HighKick         |
            KickoffResult::WeatherChange    |
            KickoffResult::QuickSnap        |
            KickoffResult::DodgySnack       |
            KickoffResult::PitchInvasion    |
            // Non-BB2025 variants (should not reach here in BB2025 games):
            KickoffResult::Blitz            |
            KickoffResult::Riot             |
            KickoffResult::PerfectDefence   |
            KickoffResult::ThrowARock       |
            KickoffResult::OficiousRef      => {
                // DEFERRED: implement full result handling.
                StepOutcome::goto(&self.goto_label_on_end.clone())
            }
        }
    }

    // ── GetTheRef ─────────────────────────────────────────────────────────────

    fn handle_get_the_ref(&self, _game: &mut Game) -> StepOutcome {
        // Java: both teams receive +1 bribe (InducementSet mutation).
        // DEFERRED: port bribe increment (InducementTypeFactory / InducementSet not yet ported).
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
        StepOutcome::next()
    }

    // ── CheeringFans ──────────────────────────────────────────────────────────

    fn handle_cheering_fans(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.cheerleaders;
        let total_away = roll_away + game.team_away.cheerleaders;

        // Java: winning team (or both on tie) gains 1 extra offensive block assist.
        if total_home >= total_away {
            game.home_additional_assists += 1;
        }
        if total_away >= total_home {
            game.away_additional_assists += 1;
        }

        StepOutcome::next()
    }

    // ── BrilliantCoaching ─────────────────────────────────────────────────────

    fn handle_brilliant_coaching(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll_home = rng.d6();
        let roll_away = rng.d6();

        let total_home = roll_home + game.team_home.assistant_coaches;
        let total_away = roll_away + game.team_away.assistant_coaches;

        // Java: winning team (or both on tie) gains +1 re-roll for the drive.
        if total_home >= total_away {
            game.turn_data_home.rerolls += 1;
        }
        if total_away >= total_home {
            game.turn_data_away.rerolls += 1;
        }

        StepOutcome::next()
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

            // DEFERRED: show DialogPlayerChoiceParameter(CHARGE) and wait for response.
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
    fn unknown_result_goto_end_label() {
        let mut game = make_game();
        let mut step = make_step();
        step.kickoff_result = Some(KickoffResult::HighKick);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }
}
