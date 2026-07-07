use ffb_model::enums::{TurnMode, InducementPhase};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_no_players_to_field::ReportNoPlayersToField;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::common::inducement::{Inducement, InducementParams};
use crate::util::util_server_setup::UtilServerSetup;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepSetup.
///
/// Handles team setup before kickoff. Manages loading/saving/deleting presets
/// (SKIP_STEP), individual player placement (SKIP_STEP), and the final
/// CLIENT_END_TURN that commits the setup.
///
/// On end-setup:
///   - checkNoPlayersInBoxOrField → if triggered, goto GOTO_LABEL_ON_END (auto-TD)
///   - checkSetup() validates formation rules
///   - toggles isHomePlaying, pushes Inducement sequences for BEFORE_SETUP phase
///   - or switches to KICKOFF turn mode
///
/// Init: mandatory GOTO_LABEL_ON_END.
pub struct StepSetup {
    /// Java: fGotoLabelOnEnd (mandatory)
    pub goto_label_on_end: Option<String>,
    /// Java: fEndSetup
    pub end_setup: bool,
}

impl StepSetup {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            end_setup: false,
        }
    }
}

impl Default for StepSetup {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetup {
    fn id(&self) -> StepId { StepId::Setup }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_TEAM_SETUP_LOAD/SAVE/DELETE, CLIENT_SETUP_PLAYER → SKIP_STEP
            // In Rust: these are no-ops (the engine processes them transparently)
            Action::PlacePlayer { player_id, coord } => {
                UtilServerSetup::setup_player(game, player_id, *coord);
                return StepOutcome::cont(); // SKIP_STEP → stay in setup
            }
            Action::ConfirmSetup => {
                // Java: CLIENT_END_TURN from current player → fEndSetup = true → executeStep
                self.end_setup = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(label) => {
                self.goto_label_on_end = Some(label.clone());
                true
            }
            _ => false,
        }
    }
}

impl StepSetup {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let goto_label = match &self.goto_label_on_end {
            Some(l) => l.clone(),
            None => return StepOutcome::cont(), // misconfigured
        };

        // Java: checkNoPlayersInBoxOrField()
        let (no_players, no_players_events) = self.check_no_players_in_box_or_field(game);
        if no_players {
            game.turn_mode = TurnMode::NoPlayersToField;
            let mut out = StepOutcome::goto(&goto_label);
            for ev in no_players_events { out = out.with_event(ev); }
            return out;
        }

        if self.end_setup {
            // Java: getResult().setSound(SoundId.DING)
            let setup_valid = SetupMechanic::new().check_setup(game, game.home_playing);
            // client-only: show setup error dialog when !setup_valid
            if setup_valid {
                game.home_playing = !game.home_playing;
                UtilBox::refresh_boxes(game);

                if game.setup_offense {
                    // Java: game.setTurnMode(TurnMode.KICKOFF)
                    game.turn_mode = TurnMode::Kickoff;
                } else {
                    // Java: game.setSetupOffense(true)
                    game.setup_offense = true;
                    // Java: push Inducement sequences for BEFORE_SETUP phase (home + away)
                    let seq_home = Inducement::build_sequence(&InducementParams {
                        inducement_phase: InducementPhase::BeforeSetup,
                        home_team: game.home_playing,
                        check_forgo: false,
                    });
                    let seq_away = Inducement::build_sequence(&InducementParams {
                        inducement_phase: InducementPhase::BeforeSetup,
                        home_team: !game.home_playing,
                        check_forgo: false,
                    });
                    return StepOutcome::next().push_seq(seq_home).push_seq(seq_away);
                }
                return StepOutcome::next();
            } else {
                self.end_setup = false;
                return StepOutcome::cont(); // back to setup
            }
        }

        // Java: if neither trigger fired → wait for client commands
        StepOutcome::cont()
    }

    /// Java: checkNoPlayersInBoxOrField() — awards TD if one team has no eligible players.
    fn check_no_players_in_box_or_field(&self, game: &mut Game) -> (bool, Vec<GameEvent>) {
        let team_home = game.team_home.clone();
        let team_away = game.team_away.clone();
        let home_players = UtilPlayer::find_players_in_reserve_or_field(game, &team_home);
        let away_players = UtilPlayer::find_players_in_reserve_or_field(game, &team_away);
        let home_empty = home_players.is_empty();
        let away_empty = away_players.is_empty();
        if home_empty || away_empty {
            let event_team_id = if !home_empty && away_empty {
                // Away has no players — home scores
                game.home_playing = true;
                game.game_result.home.score += 1;
                let id = game.team_away.id.clone();
                // Java: getResult().addReport(new ReportNoPlayersToField(game.getTeamAway().getId()))
                game.report_list.add(ReportNoPlayersToField::new(id.clone()));
                Some(id)
            } else if home_empty && !away_empty {
                // Home has no players — away scores
                game.home_playing = false;
                game.game_result.away.score += 1;
                let id = game.team_home.id.clone();
                // Java: getResult().addReport(new ReportNoPlayersToField(game.getTeamHome().getId()))
                game.report_list.add(ReportNoPlayersToField::new(id.clone()));
                Some(id)
            } else {
                // Both empty — no team id
                // Java: getResult().addReport(new ReportNoPlayersToField(null))
                game.report_list.add(ReportNoPlayersToField::new(String::new()));
                None
            };
            let events = vec![GameEvent::NoPlayersToField { team_id: event_team_id }];
            (true, events)
        } else {
            (false, vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerState, PS_RESERVE, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    /// Creates a game with one reserve player on each team so check_no_players_in_box_or_field = false.
    fn make_game_with_reserves() -> Game {
        let mut game = make_game();
        let p_home = Player {
            id: "h1".into(), name: "h1".into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
};
        let p_away = Player { id: "a1".into(), name: "a1".into(), ..p_home.clone()
        };
        game.team_home.players.push(p_home);
        game.team_away.players.push(p_away);
        game.field_model.set_player_state("h1", PlayerState::new(PS_RESERVE));
        game.field_model.set_player_state("a1", PlayerState::new(PS_RESERVE));
        game
    }

    #[test]
    fn step_id_is_setup() {
        let step = StepSetup::new();
        assert_eq!(step.id(), StepId::Setup);
    }

    #[test]
    fn goto_label_on_end_parameter_accepted() {
        let mut step = StepSetup::new();
        let ok = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".to_string()));
        assert!(ok);
        assert_eq!(step.goto_label_on_end.as_deref(), Some("end"));
    }

    #[test]
    fn start_without_label_returns_cont() {
        let mut step = StepSetup::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No label configured → cont (misconfigured guard)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_with_label_waits_for_client() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game_with_reserves();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Not end_setup yet → Continue
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn no_players_on_either_team_gotos_label() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game(); // empty rosters
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Both teams empty → goto label
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn confirm_setup_transitions_to_next() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game_with_reserves();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        // Setup valid → NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confirm_setup_toggles_home_playing() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game_with_reserves();
        game.home_playing = true;
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn confirm_setup_sets_kickoff_when_setup_offense() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game_with_reserves();
        game.setup_offense = true;
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepSetup::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }

    #[test]
    fn no_away_players_emits_no_players_to_field_report() {
        // Java: addReport(new ReportNoPlayersToField(game.getTeamAway().getId()))
        // when away has no reserve/field players and home does.
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game(); // both rosters empty — but we add only home player
        let p_home = Player {
            id: "h1".into(), name: "h1".into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p_home);
        game.field_model.set_player_state("h1", PlayerState::new(PS_RESERVE));
        // Away roster empty → home scores, away team ID is reported
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::NO_PLAYERS_TO_FIELD));
    }

    #[test]
    fn both_teams_empty_emits_no_players_to_field_report() {
        // Java: addReport(new ReportNoPlayersToField(null)) when both teams have no players.
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game(); // both rosters empty
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::NO_PLAYERS_TO_FIELD));
    }
}
