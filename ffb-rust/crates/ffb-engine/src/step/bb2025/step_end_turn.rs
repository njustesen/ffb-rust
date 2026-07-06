/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepEndTurn (BB2025).
///
/// Ends the current turn and drives the turn/half/game state machine:
/// - TurnMode skip guard (BLITZ/KICKOFF_RETURN/etc.) → publish END_TURN + NEXT_STEP
/// - Touchdown detection → score update, TurnMode → SETUP
/// - End-of-half detection (both teams at turn_nr >= 8)
/// - KICKOFF/REGULAR TurnMode transitions (home_playing flip, turn_nr++)
/// - start_turn: resets both TurnData for the next turn
///
/// Stubs (untranslated server-side systems):
/// - ArgueTheCall, Bribes, StarOfTheShow dialogs → skip (choices set to Some(false) immediately)
/// - Secret weapon ban/bribe handling → skip
/// - new_half H2 push wired (h2_kickoff_sequence); end-game sequence not yet ported.
/// - EndGame sequence push → TODO (end-game generator not yet ported)
/// - Prayer/inducement deactivation → skip
/// - Per-drive reroll removal → skip
/// - Fainting (Sweltering Heat) → skip
/// - Reports/sounds/timers → skip
/// - FumbblGame update → skip
use std::collections::HashSet;
use ffb_model::enums::{TurnMode, Weather, PS_KNOCKED_OUT, PS_EXHAUSTED, PS_RESERVE};
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_turn_end::{ReportTurnEnd, KnockoutRecovery, HeatExhaustion};
use ffb_model::types::FIELD_WIDTH;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_game::UtilServerGame;

pub struct StepEndTurn {
    /// Java: fTouchdown (Boolean tristate — None = unchecked)
    pub touchdown: Option<bool>,
    /// Java: fBribesChoiceHome
    pub bribes_choice_home: Option<bool>,
    /// Java: fBribesChoiceAway
    pub bribes_choice_away: Option<bool>,
    /// Java: fArgueTheCallChoiceHome
    pub argue_the_call_choice_home: Option<bool>,
    /// Java: fArgueTheCallChoiceAway
    pub argue_the_call_choice_away: Option<bool>,
    /// Java: useStarOfTheShow
    pub use_star_of_the_show: Option<bool>,
    /// Java: fNextSequencePushed
    pub next_sequence_pushed: bool,
    /// Java: fRemoveUsedSecretWeapons
    pub remove_used_secret_weapons: bool,
    /// Java: fNewHalf
    pub new_half: bool,
    /// Java: fEndGame
    pub end_game: bool,
    /// Java: fWithinSecretWeaponHandling
    pub within_secret_weapon_handling: bool,
    /// Java: turnNr (captured at step start for send-to-box reason)
    pub turn_nr: i32,
    /// Java: half (captured at step start)
    pub half: i32,
    /// Java: playerIdsNaturalOnes
    pub player_ids_natural_ones: Vec<String>,
    /// Java: playerIdsFailedBribes
    pub player_ids_failed_bribes: HashSet<String>,
    /// Java: playerIdsArgued
    pub player_ids_argued: HashSet<String>,
    /// Java: touchdownPlayerId
    pub touchdown_player_id: Option<String>,
    /// Java: isHomeTurnEnding = game.isHomePlaying() — captured before home_playing is flipped.
    pub is_home_turn_ending: Option<bool>,
}

impl StepEndTurn {
    pub fn new() -> Self {
        Self {
            touchdown: None,
            bribes_choice_home: None,
            bribes_choice_away: None,
            argue_the_call_choice_home: None,
            argue_the_call_choice_away: None,
            use_star_of_the_show: None,
            next_sequence_pushed: false,
            remove_used_secret_weapons: false,
            new_half: false,
            end_game: false,
            within_secret_weapon_handling: false,
            turn_nr: 0,
            half: 0,
            player_ids_natural_ones: Vec::new(),
            player_ids_failed_bribes: HashSet::new(),
            player_ids_argued: HashSet::new(),
            touchdown_player_id: None,
            is_home_turn_ending: None,
        }
    }

    /// Java: UtilServerSteps.checkTouchdown — ball in play, not moving, carrier not
    /// prone/stunned, ball is in the correct end zone for the carrier's team.
    fn check_touchdown(game: &Game) -> bool {
        if !game.field_model.ball_in_play || game.field_model.ball_moving {
            return false;
        }
        let ball_coord = match game.field_model.ball_coordinate {
            Some(c) => c,
            None => return false,
        };
        let carrier_id = match game.field_model.player_at(ball_coord) {
            Some(id) => id.clone(),
            None => return false,
        };
        let carrier_state = match game.field_model.player_state(&carrier_id) {
            Some(s) => s,
            None => return false,
        };
        if carrier_state.is_prone_or_stunned() {
            return false;
        }
        let home_has_carrier = game.team_home.player(&carrier_id).is_some();
        if home_has_carrier {
            ball_coord.x == FIELD_WIDTH - 1  // FieldCoordinateBounds.ENDZONE_AWAY
        } else {
            ball_coord.x == 0               // FieldCoordinateBounds.ENDZONE_HOME
        }
    }

    /// Java: UtilServerSteps.checkEndOfHalf — both teams have used all 8 turns.
    fn check_end_of_half(game: &Game) -> bool {
        game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (turnNr == 0) capture turnNr + half for later send-to-box use
        if self.turn_nr == 0 {
            self.turn_nr = game.turn_data().turn_nr;
            self.half = game.half;
            // Java: isHomeTurnEnding = game.isHomePlaying() — captured before state flip.
            self.is_home_turn_ending = Some(game.home_playing);
        }

        // Skip guard: non-player-turn modes just propagate END_TURN and exit
        let skip_mode = matches!(
            game.turn_mode,
            TurnMode::Blitz | TurnMode::KickoffReturn | TurnMode::PassBlock
                | TurnMode::IllegalSubstitution | TurnMode::Swarming
        );
        if skip_mode {
            return StepOutcome::next().publish(StepParameter::EndTurn(true));
        }

        // Stub: within_secret_weapon_handling path (uses server dialogs) → treat as false
        // i.e. always enter the main block below

        // Touchdown check (cached tristate)
        if self.touchdown.is_none() {
            self.touchdown = Some(Self::check_touchdown(game));
        }
        let touchdown = self.touchdown.unwrap_or(false);

        // StarOfTheShow: dialog not translated → always false
        if self.use_star_of_the_show.is_none() {
            self.use_star_of_the_show = Some(false);
        }

        UtilServerGame::mark_played_and_secret_weapons(game);

        self.new_half = Self::check_end_of_half(game);

        if !self.next_sequence_pushed {
            self.next_sequence_pushed = true;

            if touchdown {
                // Identify ball carrier and update score
                if let Some(ball_coord) = game.field_model.ball_coordinate {
                    if let Some(carrier_id) = game.field_model.player_at(ball_coord).cloned() {
                        self.touchdown_player_id = Some(carrier_id.clone());
                        let home_has_carrier = game.team_home.player(&carrier_id).is_some();
                        let off_turn_touchdown;
                        if home_has_carrier {
                            game.game_result.home.score += 1;
                            off_turn_touchdown = !game.home_playing;
                        } else {
                            game.game_result.away.score += 1;
                            off_turn_touchdown = game.home_playing;
                        }
                        game.home_playing = home_has_carrier;
                        if off_turn_touchdown {
                            game.turn_data_mut().turn_nr += 1;
                            self.new_half = Self::check_end_of_half(game);
                        }
                    }
                }
                game.turn_mode = TurnMode::Setup;
                game.setup_offense = false;
                // Stub: kickoffGenerator / endGenerator pushes → skip
                // Stub: ball clear, sound, resetSpecialSkillAtEndOfDrive → skip
            } else {
                match game.turn_mode {
                    TurnMode::Kickoff => {
                        game.home_playing = !game.home_playing;
                        game.turn_data_mut().turn_nr += 1;
                        game.turn_data_mut().turn_started = false;
                        game.turn_data_mut().first_turn_after_kickoff = true;
                        game.turn_mode = TurnMode::Regular;
                    }
                    TurnMode::Regular => {
                        if self.new_half {
                            game.turn_mode = TurnMode::Setup;
                            game.setup_offense = false;
                        } else {
                            game.home_playing = !game.home_playing;
                            game.turn_data_mut().turn_nr += 1;
                        }
                        game.turn_data_mut().turn_started = false;
                        game.turn_data_mut().first_turn_after_kickoff = false;
                    }
                    _ => {}
                }
                // Stub: sequence generator pushes (Kickoff, EndGame, Inducement) → skip
            }

            // Java: fieldModel.clearMoveSquares / clearTrackNumbers / clearDiceDecorations → stub
            // Java: reportSecretWeaponsUsed → stub
        }

        // ArgueTheCall / Bribes dialogs — not translated, resolve immediately to false
        if self.argue_the_call_choice_away.is_none() {
            self.argue_the_call_choice_away = Some(false);
        }
        if self.argue_the_call_choice_home.is_none() && self.argue_the_call_choice_away.is_some() {
            self.argue_the_call_choice_home = Some(false);
        }
        if self.bribes_choice_away.is_none()
            && self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
        {
            self.bribes_choice_away = Some(false);
        }
        if self.bribes_choice_home.is_none()
            && self.bribes_choice_away.is_some()
            && self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
        {
            self.bribes_choice_home = Some(false);
        }

        let all_choices_done = self.argue_the_call_choice_home.is_some()
            && self.argue_the_call_choice_away.is_some()
            && self.bribes_choice_home.is_some()
            && self.bribes_choice_away.is_some();

        if self.end_game || all_choices_done {
            // Java: deactivateEffectsAndPrayers / deactivateCards
            {
                use crate::util::util_server_cards::UtilServerCards;
                use ffb_model::enums::InducementDuration;
                let is_home = self.is_home_turn_ending.unwrap_or(game.home_playing);
                UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfTurn, is_home);
                UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfOpponentsTurn, is_home);
                if self.new_half || touchdown {
                    UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfDrive, is_home);
                    UtilServerCards::deactivate_cards(game, InducementDuration::UntilEndOfHalf, is_home);
                }
            }
            // Java: getFaintingCount / heatExhaustions / KO recovery — only on new half or touchdown
            if self.new_half || touchdown {
                let all_player_ids: Vec<String> = game.team_home.players.iter()
                    .chain(game.team_away.players.iter())
                    .map(|p| p.id.clone())
                    .collect();
                let mut ko_recoveries: Vec<KnockoutRecovery> = Vec::new();
                let mut heat_exhaustions: Vec<HeatExhaustion> = Vec::new();
                for player_id in &all_player_ids {
                    let player_state = match game.field_model.player_state(player_id) {
                        Some(s) => s,
                        None => continue,
                    };
                    let base = player_state.base();
                    if base == PS_KNOCKED_OUT {
                        let is_home = game.team_home.has_player(player_id);
                        let bloodweiser_keg = if is_home {
                            game.turn_data_home.inducement_set.value(Usage::KNOCKOUT_RECOVERY)
                        } else {
                            game.turn_data_away.inducement_set.value(Usage::KNOCKOUT_RECOVERY)
                        };
                        let roll = rng.d6();
                        let recovered = DiceInterpreter::is_recovering_from_knockout(roll, bloodweiser_keg);
                        if recovered {
                            game.field_model.set_player_state(player_id, player_state.change_base(PS_RESERVE));
                        }
                        ko_recoveries.push(KnockoutRecovery::new(player_id.clone(), recovered));
                    }
                    if base == PS_EXHAUSTED {
                        game.field_model.set_player_state(player_id, player_state.change_base(PS_RESERVE));
                    }
                    if let Some(coord) = game.field_model.player_coordinate(player_id) {
                        if game.field_model.weather == Weather::SwelteringHeat && !coord.is_box_coordinate() {
                            let roll = rng.d6();
                            if DiceInterpreter::is_exhausted(roll) {
                                let cur = game.field_model.player_state(player_id).unwrap_or_default();
                                game.field_model.set_player_state(player_id, cur.change_base(PS_EXHAUSTED));
                                heat_exhaustions.push(HeatExhaustion::new(player_id.clone(), roll));
                            }
                        }
                    }
                }
                let td_player_id = if touchdown { game.acting_player.player_id.clone() } else { None };
                game.report_list.add(ReportTurnEnd::new(
                    td_player_id,
                    ko_recoveries,
                    heat_exhaustions,
                    vec![],
                    0,
                ));
                UtilBox::put_all_players_into_box(game);
            }

            // Java: game.startTurn() — reset per-turn flags for both teams
            game.turn_data_home.reset_for_turn();
            game.turn_data_away.reset_for_turn();

            // Java: endGenerator.pushSequence / kickoffGenerator
            use ffb_model::enums::InducementPhase;
            use crate::step::sequences::{inducement_sequence, h2_kickoff_sequence, end_game_sequence};
            let mut outcome = StepOutcome::next();
            if self.new_half {
                // Java: half > 2 → end_game; half > 1 → overtime check or end_game; else → H2 kickoff
                if game.half > 1 {
                    // Half 2+ ended → end game (overtime check not yet ported)
                    outcome = outcome.push_seq(end_game_sequence(game.admin_mode));
                } else {
                    // End of first half → push H2 kickoff sequence
                    outcome = outcome.push_seq(h2_kickoff_sequence());
                }
            } else if touchdown {
                // Java: touchdownEndsGame check → end_game if last turn
                let td_ends_game = game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8;
                if td_ends_game {
                    outcome = outcome.push_seq(end_game_sequence(game.admin_mode));
                } else {
                    outcome = outcome.push_seq(h2_kickoff_sequence());
                }
            } else if game.turn_mode != TurnMode::Regular {
                // Non-regular turn end (blitz turn etc.) → kickoff
                outcome = outcome.push_seq(h2_kickoff_sequence());
            } else {
                outcome = outcome.push_seq(inducement_sequence(InducementPhase::StartOfOwnTurn, game.home_playing));
            }
            // Java: UtilServerTimer.startTurnTimer → stub
            // Java: updateFumbblGame → stub
            return outcome;
        }

        // In the stub, all_choices_done is always true so we never reach here.
        // Kept for correctness if dialogs are later wired in.
        StepOutcome::cont()
    }
}

impl Default for StepEndTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndTurn {
    fn id(&self) -> StepId { StepId::EndTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    // Java StepEndTurn does not override setParameter.
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.turn_nr = 1;
        game.turn_data_away.turn_nr = 1;
        game
    }

    #[test]
    fn blitz_mode_publishes_end_turn_and_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn regular_turn_flips_home_playing_and_increments_turn_nr() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 3;
        game.turn_data_away.turn_nr = 3;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // After home's turn ends, home_playing flips to false (away's turn)
        assert!(!game.home_playing);
        // Away's turn_nr was 3, now 4
        assert_eq!(game.turn_data_away.turn_nr, 4);
        assert_eq!(game.turn_data_home.turn_nr, 3);
    }

    #[test]
    fn end_of_half_transitions_to_setup() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Setup);
    }

    #[test]
    fn touchdown_increments_home_score_and_transitions_to_setup() {
        let mut game = make_game();
        game.team_home.players.push(make_player("scorer"));
        let ball_coord = FieldCoordinate::new(FIELD_WIDTH - 1, 7);
        game.field_model.set_player_coordinate("scorer", ball_coord);
        game.field_model.set_player_state("scorer", PlayerState::new(PS_STANDING));
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;

        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.score, 1);
        assert_eq!(game.turn_mode, TurnMode::Setup);
        assert_eq!(step.touchdown_player_id.as_deref(), Some("scorer"));
    }

    #[test]
    fn touchdown_increments_away_score() {
        let mut game = make_game();
        game.home_playing = false;
        game.team_away.players.push(make_player("scorer2"));
        let ball_coord = FieldCoordinate::new(0, 7);
        game.field_model.set_player_coordinate("scorer2", ball_coord);
        game.field_model.set_player_state("scorer2", PlayerState::new(PS_STANDING));
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = false;

        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.away.score, 1);
        assert_eq!(game.game_result.home.score, 0);
        assert_eq!(game.turn_mode, TurnMode::Setup);
    }

    #[test]
    fn kickoff_mode_transitions_to_regular_and_flips_team() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        game.turn_data_away.turn_nr = 2;
        let mut step = StepEndTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
        // Home was playing, so now away is playing
        assert!(!game.home_playing);
        // Away's turn_nr incremented
        assert_eq!(game.turn_data_away.turn_nr, 3);
        assert!(game.turn_data_away.first_turn_after_kickoff);
    }

    #[test]
    fn start_turn_resets_both_turn_data() {
        let mut game = make_game();
        game.turn_data_home.blitz_used = true;
        game.turn_data_away.foul_used = true;
        let mut step = StepEndTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.turn_data_home.blitz_used);
        assert!(!game.turn_data_away.foul_used);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepEndTurn::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(!step.set_parameter(&StepParameter::EndGame(true)));
    }

    #[test]
    fn check_end_of_half_requires_both_teams_at_8() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 7;
        assert!(!StepEndTurn::check_end_of_half(&game));
        game.turn_data_away.turn_nr = 8;
        assert!(StepEndTurn::check_end_of_half(&game));
    }
}
