use ffb_model::enums::TurnMode;
use ffb_model::model::blitz_turn_state::BlitzTurnState;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::select::Select;

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepBlitzTurn` (BB2020).
///
/// Step in the kickoff sequence to handle the Blitz! kickoff result.
///
/// Expects `END_TURN` to be set by a preceding step (consumed on TurnMode::BLITZ).
///
/// Two-phase behaviour:
///  1. If `game.turn_mode == TurnMode::BLITZ` — cleanup: clear blitz turn state,
///     reset turn mode to KICKOFF. Proceed to NEXT_STEP.
///  2. Otherwise — setup: find the blitzing team, pin players in tackle zones,
///     count active players. If none available → report exhausted activations.
///     Otherwise: roll d3 for activation limit, set blitz turn state, start the turn,
///     push a Select sequence, report the roll. Proceed to NEXT_STEP.
pub struct StepBlitzTurn;

impl StepBlitzTurn {
    pub fn new() -> Self { Self }
}

impl Default for StepBlitzTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlitzTurn {
    fn id(&self) -> StepId { StepId::BlitzTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepBlitzTurn {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::Blitz {
            // Java: getGameState().setBlitzTurnState(null)
            // Java: game.setTurnMode(TurnMode.KICKOFF)
            game.blitz_turn_state = None;
            game.turn_mode = TurnMode::Kickoff;
        } else {
            // Java: Team blitzingTeam = game.isHomePlaying() ? game.getTeamHome() : game.getTeamAway()
            // Java: SetupMechanic mechanic = ...; mechanic.pinPlayersInTacklezones(getGameState(), blitzingTeam, true)
            // TODO(blitz_turn): call SetupMechanic.pinPlayersInTacklezones when mechanics layer is available

            // Java: int availablePlayers = Arrays.stream(blitzingTeam.getPlayers())
            //   .filter(player -> game.getFieldModel().getPlayerState(player).isActive()).count()
            let available_players = Self::count_active_players(game);

            if available_players == 0 {
                // Java: getResult().addReport(new ReportKickoffSequenceActivationsExhausted(false))
                // TODO(blitz_turn): emit ReportKickoffSequenceActivationsExhausted event
            } else {
                // Java: int roll = getGameState().getDiceRoller().rollDice(3)
                // Java: int limit = roll + 3
                let roll = rng.d3();
                let limit = roll + 3;

                // Java: game.setTurnMode(TurnMode.BLITZ)
                game.turn_mode = TurnMode::Blitz;
                // Java: getGameState().setBlitzTurnState(new BlitzTurnState(limit, availablePlayers))
                game.blitz_turn_state = Some(BlitzTurnState::new(limit, available_players));

                // Java: if (game.isTurnTimeEnabled()) { UtilServerTimer.stopTurnTimer(...); ... startTurnTimer(...) }
                // TODO(blitz_turn): timer management when server timer infra is available

                // Java: game.startTurn()
                // TODO(blitz_turn): call game.startTurn() when method is available

                // Java: UtilServerGame.updatePlayerStateDependentProperties(this)
                // TODO(blitz_turn): updatePlayerStateDependentProperties when UtilServerGame is available

                // Java: getGameState().pushCurrentStepOnStack()
                // Java: ((Select) factory.forName(SequenceGenerator.Type.Select.name()))
                //   .pushSequence(new Select.SequenceParams(getGameState(), true))
                // In Rust: push the Select sequence via the generator stub
                // TODO(blitz_turn): Select.push_sequence needs full implementation
                let _ = Select::new();

                // Java: getResult().addReport(new ReportBlitzRoll(blitzingTeam.getId(), roll, limit))
                // TODO(blitz_turn): emit ReportBlitzRoll event
                let _ = (roll, limit);
            }
        }

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
    }

    /// Java: Arrays.stream(blitzingTeam.getPlayers())
    ///   .filter(player -> game.getFieldModel().getPlayerState(player).isActive()).count()
    fn count_active_players(game: &Game) -> i32 {
        let team = if game.home_playing { &game.team_home } else { &game.team_away };
        team.players.iter()
            .filter(|p| {
                game.field_model.player_state(&p.id)
                    .map(|s| s.is_active())
                    .unwrap_or(false)
            })
            .count() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, TurnMode, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
        }
    }

    /// When already in BLITZ mode, clears blitz_turn_state and resets turn_mode to KICKOFF.
    #[test]
    fn cleanup_phase_clears_state_and_resets_to_kickoff() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        game.blitz_turn_state = Some(BlitzTurnState::new(4, 2));

        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.blitz_turn_state.is_none());
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    /// When no active players are available, outcome is NEXT_STEP (no blitz turn started).
    #[test]
    fn no_active_players_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        game.home_playing = true;
        // No players added to home team → availablePlayers = 0

        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        // No blitz turn state should be set when no players available
        assert!(game.blitz_turn_state.is_none());
    }

    /// When active players are present, sets turn_mode to BLITZ and creates blitz turn state.
    #[test]
    fn active_players_sets_blitz_turn_state() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        game.home_playing = true;

        // Add an active player to home team
        let player = make_player("home1");
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("home1", FieldCoordinate::new(5, 7));
        use ffb_model::enums::PlayerState;
        game.field_model.set_player_state(
            "home1",
            PlayerState::new(PS_STANDING).change_active(true),
        );

        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(42));

        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Blitz);
        assert!(game.blitz_turn_state.is_some());

        // limit = d3 + 3, so must be in range [4..6]
        let bts = game.blitz_turn_state.as_ref().unwrap();
        assert!(bts.get_limit() >= 4 && bts.get_limit() <= 6,
            "limit={} must be d3+3", bts.get_limit());
        assert_eq!(bts.get_available(), 1);
    }

    /// Blitz turn state limit is d3 + 3 (range 4–6).
    #[test]
    fn blitz_limit_is_d3_plus_3() {
        for seed in 0u64..50 {
            let mut game = make_game();
            game.turn_mode = TurnMode::Kickoff;
            game.home_playing = true;

            let player = make_player("home1");
            game.team_home.players.push(player);
            game.field_model.set_player_coordinate("home1", FieldCoordinate::new(5, 7));
            use ffb_model::enums::PlayerState;
            game.field_model.set_player_state(
                "home1",
                PlayerState::new(PS_STANDING).change_active(true),
            );

            let mut step = StepBlitzTurn::new();
            step.start(&mut game, &mut GameRng::new(seed));

            if let Some(bts) = &game.blitz_turn_state {
                assert!(bts.get_limit() >= 4 && bts.get_limit() <= 6,
                    "seed={seed} limit={} must be d3+3", bts.get_limit());
            }
        }
    }

    /// handle_command delegates to execute_step.
    #[test]
    fn handle_command_delegates_to_execute_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        game.blitz_turn_state = Some(BlitzTurnState::new(5, 1));

        let mut step = StepBlitzTurn::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.blitz_turn_state.is_none());
    }

    /// set_parameter always returns false (no parameters).
    #[test]
    fn set_parameter_always_false() {
        let mut step = StepBlitzTurn::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(3)));
    }

    /// count_active_players counts only active players on the active team.
    #[test]
    fn count_active_players_counts_only_active() {
        let mut game = make_game();
        game.home_playing = true;

        let p1 = make_player("h1");
        let p2 = make_player("h2");
        let p3 = make_player("h3");
        game.team_home.players.extend([p1, p2, p3]);

        use ffb_model::enums::PlayerState;
        // h1 active, h2 inactive, h3 active
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("h2", PlayerState::new(PS_STANDING).change_active(false));
        game.field_model.set_player_state("h3", PlayerState::new(PS_STANDING).change_active(true));

        assert_eq!(StepBlitzTurn::count_active_players(&game), 2);
    }
}
