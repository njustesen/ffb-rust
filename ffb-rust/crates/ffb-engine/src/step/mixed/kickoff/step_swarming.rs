use ffb_model::enums::{PS_PRONE, PS_RESERVE, SkillId, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_setup::UtilServerSetup;

/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.kickoff.StepSwarming`.
///
/// Handles the Swarming skill mechanic during kickoff setup.
///
/// The actual logic lived in `SwarmingBehaviour.handleExecuteStepHook()` (BB2016 and BB2020
/// variants) and is inlined here. The BB2020 variant has `rolled_amount`/`limiting_amount`
/// and a separate `leave()` method; BB2016 inlines `leave()` and skips `rolled_amount`.
/// This implementation follows the BB2020 behaviour (which supersedes BB2016).
///
/// State fields (Java `StepState`):
/// - `handle_receiving_team` — if true, the receiving team acts; else kicking team
/// - `end_turn` — set when `CLIENT_END_TURN` is received
/// - `allowed_amount` — cap on how many swarmers may be placed
/// - `rolled_amount` — the d3 roll value
/// - `limiting_amount` — swarmers already on pitch (used with rolled_amount for min)
/// - `team_id` — the acting team's id
///
/// BB2016 / BB2020.
pub struct StepSwarming {
    /// Java: state.handleReceivingTeam
    pub handle_receiving_team: bool,
    /// Java: state.endTurn
    pub end_turn: bool,
    /// Java: state.allowedAmount
    pub allowed_amount: i32,
    /// Java: state.rolledAmount
    pub rolled_amount: i32,
    /// Java: state.limitingAmount
    pub limiting_amount: i32,
    /// Java: state.teamId
    pub team_id: Option<String>,
}

impl StepSwarming {
    pub fn new() -> Self {
        Self {
            handle_receiving_team: false,
            end_turn: false,
            allowed_amount: 0,
            rolled_amount: -1,
            limiting_amount: -1,
            team_id: None,
        }
    }
}

impl Default for StepSwarming {
    fn default() -> Self { Self::new() }
}

impl Step for StepSwarming {
    fn id(&self) -> StepId { StepId::Swarming }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                // no-op: player positions tracked directly in field_model
                self.end_turn = true;
            }
            Action::PlacePlayer { player_id, coord } => {
                UtilServerSetup::setup_player(game, player_id, *coord);
                return StepOutcome::cont();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::HandleReceivingTeam(v) => { self.handle_receiving_team = *v; true }
            _ => false,
        }
    }
}

impl StepSwarming {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java BB2020 SwarmingBehaviour.handleExecuteStepHook inlined.
        if game.turn_mode == TurnMode::Swarming {
            // Second entry: coach has placed their swarmers and clicked end turn.
            if self.end_turn {
                self.end_turn = false;
                // Count active non-box players in the swarming team.
                let placed = self.count_placed_active_players(game);
                if placed > self.allowed_amount {
                    // client-only: DialogSwarmingErrorParameter
                    return StepOutcome::cont();
                }
                self.leave(game, placed);
                return StepOutcome::next();
            }
            return StepOutcome::cont();
        }

        // First entry: determine swarming team and set up the placement phase.

        // Java: if !handleReceivingTeam → gameState.setKickingSwarmers(0)
        if !self.handle_receiving_team {
            game.kicking_swarmers = 0;
        }

        let team_id = self.swarming_team_id(game);
        self.team_id = Some(team_id.clone());

        // Collect on-pitch and reserve players for the swarming team.
        let player_ids: Vec<String> = {
            let team = if game.team_home.id == team_id { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.id.clone()).collect()
        };

        let mut swarmers_on_pitch = 0i32;
        let mut players_on_pitch: Vec<String> = Vec::new();
        let mut reserve_no_swarming: Vec<String> = Vec::new();
        let mut has_swarming_reserves = false;

        for pid in &player_ids {
            let on_field = game.field_model.player_coordinate(pid)
                .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                .unwrap_or(false);
            let base = game.field_model.player_state(pid)
                .map(|s| s.base())
                .unwrap_or(0);
            let has_swarming = {
                let team = if game.team_home.id == team_id { &game.team_home } else { &game.team_away };
                team.players.iter()
                    .find(|p| p.id == *pid)
                    .map(|p| p.has_skill(SkillId::Swarming))
                    .unwrap_or(false)
            };

            if on_field {
                if has_swarming { swarmers_on_pitch += 1; }
                players_on_pitch.push(pid.clone());
            } else if base == PS_RESERVE {
                if has_swarming {
                    has_swarming_reserves = true;
                } else {
                    reserve_no_swarming.push(pid.clone());
                }
            }
        }

        self.limiting_amount = swarmers_on_pitch;

        if !has_swarming_reserves {
            return StepOutcome::next();
        }

        // Deactivate all on-pitch players.
        for pid in &players_on_pitch {
            if let Some(state) = game.field_model.player_state(pid) {
                game.field_model.set_player_state(pid, state.change_active(false));
            }
        }

        // Set non-swarming reserve players to PRONE.
        for pid in &reserve_no_swarming {
            if let Some(state) = game.field_model.player_state(pid) {
                game.field_model.set_player_state(pid, state.change_base(PS_PRONE));
            }
        }

        // Flip home_playing when handling the receiving team.
        if self.handle_receiving_team {
            game.home_playing = !game.home_playing;
        }

        game.turn_mode = TurnMode::Swarming;

        // Java: pushCurrentStepOnStack() → Rust StepAction::Repeat (driver re-enters start()).
        // Roll how many swarmers may be placed: d3 (Java: DiceRoller.rollSwarmingPlayers() = d3).
        self.rolled_amount = rng.d3();
        self.allowed_amount = self.limiting_amount.min(self.rolled_amount);

        let event = GameEvent::SwarmingPlayersRoll {
            team_id: team_id.clone(),
            roll: self.rolled_amount,
        };

        if self.allowed_amount == 0 {
            // Java: if allowedAmount == 0 → leave() immediately + NEXT_STEP
            self.leave(game, 0);
            return StepOutcome::next().with_event(event);
        }

        // client-only: DialogSwarmingPlayersParameter(allowedAmount)
        StepOutcome::cont().with_event(event)
    }

    /// Java BB2020 SwarmingBehaviour.leave() — restores PRONE→RESERVE, resets turn mode.
    fn leave(&mut self, game: &mut Game, placed_swarming_players: i32) {
        if let Some(ref team_id) = self.team_id.clone() {
            let player_ids: Vec<String> = {
                let team = if game.team_home.id == *team_id { &game.team_home } else { &game.team_away };
                team.players.iter().map(|p| p.id.clone()).collect()
            };
            for pid in &player_ids {
                if let Some(state) = game.field_model.player_state(pid) {
                    if state.base() == PS_PRONE {
                        game.field_model.set_player_state(pid, state.change_base(PS_RESERVE));
                    }
                }
            }
        }

        game.turn_mode = TurnMode::Kickoff;
        let mechanic = crate::mechanic::game_mechanic_for(game.rules);
        UtilPlayer::refresh_players_for_turn_start(game, &mechanic.enhancements_to_remove_at_end_of_turn(), &mechanic.enhancements_to_remove_at_end_of_turn_when_not_setting_active());
        game.field_model.clear_track_numbers();

        if self.handle_receiving_team {
            game.home_playing = !game.home_playing;
        } else {
            // Java: gameState.setKickingSwarmers(placedSwarmingPlayers)
            game.kicking_swarmers = placed_swarming_players;
        }
        // Java: step.getGameState().getStepStack().pop() → handled by Rust returning NextStep
    }

    /// Java BB2020 SwarmingBehaviour.swarmingTeam() — identifies the acting team.
    fn swarming_team_id(&self, game: &Game) -> String {
        if self.handle_receiving_team {
            if game.home_playing { game.team_away.id.clone() } else { game.team_home.id.clone() }
        } else {
            if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() }
        }
    }

    /// Count active non-box players in the swarming team.
    fn count_placed_active_players(&self, game: &Game) -> i32 {
        let team_id = match &self.team_id {
            Some(id) => id.clone(),
            None => return 0,
        };
        let team = if game.team_home.id == team_id { &game.team_home } else { &game.team_away };
        team.players.iter().filter(|p| {
            let active = game.field_model.player_state(&p.id).map(|s| s.is_active()).unwrap_or(false);
            let not_box = game.field_model.player_coordinate(&p.id)
                .map(|c| !c.is_box_coordinate())
                .unwrap_or(false);
            active && not_box
        }).count() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_player(id: &str, nr: u8) -> Player {
        use ffb_model::enums::{PlayerGender, PlayerType};
        Player {
            id: id.into(), name: id.into(), nr: nr as i32, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_swarmer(id: &str, nr: u8) -> Player {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut p = make_player(id, nr);
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::Swarming, value: None });
        p
    }

    #[test]
    fn id_is_swarming() {
        assert_eq!(StepSwarming::new().id(), StepId::Swarming);
    }

    #[test]
    fn no_swarming_reserves_returns_next_step() {
        // No players → no swarming reserves → NextStep
        let mut step = StepSwarming::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn reserve_swarmer_only_no_pitch_swarmers_leaves_immediately() {
        // No swarmers on pitch → limiting_amount = 0 → allowed_amount = 0 → leave() → NextStep
        let mut game = make_game();
        game.home_playing = true;
        let swarmer = make_swarmer("h_swarmer", 1);
        game.team_home.players.push(swarmer);
        game.field_model.set_player_state("h_swarmer", PlayerState::new(PS_RESERVE));

        let mut step = StepSwarming::new();
        let out = step.start(&mut game, &mut GameRng::new(3));
        // allowed_amount = min(0, rolled) = 0 → leave() called → NextStep, TurnMode::Kickoff
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn swarmer_on_pitch_plus_reserve_enters_swarming_mode() {
        let mut game = make_game();
        game.home_playing = true;
        // Put a swarmer on the pitch (limiting_amount = 1)
        let on_pitch = make_swarmer("h_s1", 1);
        game.team_home.players.push(on_pitch);
        game.field_model.set_player_coordinate("h_s1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("h_s1", PlayerState::new(PS_STANDING));
        // And a swarmer in reserve
        let reserve = make_swarmer("h_s2", 2);
        game.team_home.players.push(reserve);
        game.field_model.set_player_state("h_s2", PlayerState::new(PS_RESERVE));

        let mut step = StepSwarming::new();
        // Use seed 1 → d3 will return at least 1; limiting_amount=1 so allowed≥1 → Continue
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(game.turn_mode, TurnMode::Swarming);
        assert_eq!(step.limiting_amount, 1);
        assert!(step.rolled_amount >= 1);
        assert_eq!(step.allowed_amount, 1.min(step.rolled_amount));
        // allowed_amount = min(1, rolled) ≥ 1 → Continue (dialog)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_with_valid_placement_leaves() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::Swarming;

        let mut step = StepSwarming::new();
        step.team_id = Some("home".into());
        step.allowed_amount = 2;
        // No active non-box players placed → placed = 0 ≤ 2 → leave()
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn end_turn_with_too_many_placed_stays() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::Swarming;

        let p1 = make_player("h1", 1);
        game.team_home.players.push(p1);
        // Active on-pitch (not box)
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepSwarming::new();
        step.team_id = Some("home".into());
        step.allowed_amount = 0; // 0 allowed but 1 placed

        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        // Placed (1) > allowed (0) → client-only: dialog → Continue
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::Swarming);
    }

    #[test]
    fn set_parameter_handle_receiving_team() {
        let mut step = StepSwarming::new();
        assert!(step.set_parameter(&StepParameter::HandleReceivingTeam(true)));
        assert!(step.handle_receiving_team);
    }

    #[test]
    fn set_parameter_unrecognised_returns_false() {
        let mut step = StepSwarming::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn place_player_updates_coordinate_and_continues() {
        let mut step = StepSwarming::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Swarming;
        // Player must belong to the active team for UtilServerSetup::setup_player to accept it.
        game.team_home.players.push(make_player("p1", 1));
        game.field_model.set_player_state("p1", ffb_model::enums::PlayerState::new(PS_RESERVE));
        let coord = FieldCoordinate::new(5, 3);
        let out = step.handle_command(
            &Action::PlacePlayer { player_id: "p1".into(), coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.field_model.player_coordinate("p1"), Some(coord));
    }

    #[test]
    fn swarming_team_receiving_is_away_when_home_playing() {
        let game = make_game();
        let mut step = StepSwarming::new();
        step.handle_receiving_team = true;
        assert_eq!(step.swarming_team_id(&game), "away");
    }

    #[test]
    fn swarming_team_kicking_is_home_when_home_playing() {
        let game = make_game();
        let step = StepSwarming::new();
        assert_eq!(step.swarming_team_id(&game), "home");
    }

    #[test]
    fn non_swarming_reserves_set_prone() {
        let mut game = make_game();
        game.home_playing = true;
        // Non-swarmer reserve player → should be set PRONE during swarming phase
        let p = make_player("h_ns", 1);
        game.team_home.players.push(p);
        game.field_model.set_player_state("h_ns", PlayerState::new(PS_RESERVE));
        // Swarmer on pitch → limiting_amount = 1 so allowed_amount = min(1, roll) ≥ 1 → stay in swarming
        let on_pitch = make_swarmer("h_s_on", 2);
        game.team_home.players.push(on_pitch);
        game.field_model.set_player_coordinate("h_s_on", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("h_s_on", PlayerState::new(PS_STANDING));
        // Swarmer in reserve (triggers the swarming path)
        let sw = make_swarmer("h_s", 3);
        game.team_home.players.push(sw);
        game.field_model.set_player_state("h_s", PlayerState::new(PS_RESERVE));

        let mut step = StepSwarming::new();
        step.start(&mut game, &mut GameRng::new(0));

        // h_ns should be PRONE (not restored since we didn't call leave yet)
        let state = game.field_model.player_state("h_ns").unwrap();
        assert_eq!(state.base(), PS_PRONE, "non-swarmer reserve should be set PRONE during setup");
        assert_eq!(game.turn_mode, TurnMode::Swarming);
    }

    #[test]
    fn leave_restores_prone_to_reserve() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Swarming;
        // Player was set PRONE during setup
        let p = make_player("h1", 1);
        game.team_home.players.push(p);
        game.field_model.set_player_state("h1", PlayerState::new(PS_PRONE));

        let mut step = StepSwarming::new();
        step.team_id = Some("home".into());
        step.leave(&mut game, 0);

        let state = game.field_model.player_state("h1").unwrap();
        assert_eq!(state.base(), PS_RESERVE, "PRONE → RESERVE after leave");
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn receiving_team_flips_home_playing() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_mode = TurnMode::Swarming;

        let mut step = StepSwarming::new();
        step.team_id = Some("away".into());
        step.handle_receiving_team = true;
        step.leave(&mut game, 0);

        assert!(!game.home_playing, "home_playing should be flipped back");
    }

    #[test]
    fn swarming_roll_event_emitted() {
        let mut game = make_game();
        game.home_playing = true;
        let sw = make_swarmer("h_s", 1);
        game.team_home.players.push(sw.clone());
        // Swarmer on pitch (limiting_amount=1) + reserve swarmer
        game.field_model.set_player_coordinate("h_s", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("h_s", PlayerState::new(PS_STANDING));
        let sw2 = make_swarmer("h_s2", 2);
        game.team_home.players.push(sw2);
        game.field_model.set_player_state("h_s2", PlayerState::new(PS_RESERVE));

        let mut step = StepSwarming::new();
        let out = step.start(&mut game, &mut GameRng::new(1));
        let has_event = out.events.iter().any(|e| matches!(e, GameEvent::SwarmingPlayersRoll { .. }));
        assert!(has_event, "SwarmingPlayersRoll event should be emitted");
    }
}
