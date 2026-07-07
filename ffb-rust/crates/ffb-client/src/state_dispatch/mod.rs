use ffb_model::model::game::Game;
use ffb_model::enums::{ClientStateId, TurnMode};

/// Determine which client state is currently active based on the game state
/// and which side the Rust client is playing.
///
/// Mirrors `ClientStateFactory.java` — a pure function mapping (TurnMode, game context)
/// to a `ClientStateId`.
pub fn current_state(game: &Game, our_team_id: &str) -> ClientStateId {
    let is_our_turn = game.active_team().id == our_team_id;

    match game.turn_mode {
        TurnMode::StartGame => ClientStateId::StartGame,

        TurnMode::Setup => {
            if is_our_turn {
                ClientStateId::Setup
            } else {
                ClientStateId::WaitForSetup
            }
        }

        TurnMode::Kickoff => ClientStateId::Kickoff,

        TurnMode::HighKick => ClientStateId::HighKick,
        TurnMode::QuickSnap => ClientStateId::QuickSnap,
        TurnMode::Touchback => ClientStateId::Touchback,
        TurnMode::SolidDefence => ClientStateId::SolidDefence,
        TurnMode::PerfectDefence => ClientStateId::Setup,

        TurnMode::KickoffReturn => ClientStateId::KickoffReturn,

        TurnMode::Regular | TurnMode::BetweenTurns => {
            if is_our_turn {
                ClientStateId::SelectPlayer
            } else {
                ClientStateId::WaitForOpponent
            }
        }

        TurnMode::Blitz => {
            if is_our_turn {
                ClientStateId::Blitz
            } else {
                ClientStateId::WaitForOpponent
            }
        }

        TurnMode::Interception => ClientStateId::Interception,

        TurnMode::PassBlock => {
            if is_our_turn {
                ClientStateId::PassBlock
            } else {
                ClientStateId::WaitForOpponent
            }
        }

        TurnMode::DumpOff => ClientStateId::DumpOff,

        TurnMode::Swarming => ClientStateId::Swarming,

        TurnMode::Wizard => {
            if is_our_turn {
                ClientStateId::Wizard
            } else {
                ClientStateId::WaitForOpponent
            }
        }

        TurnMode::SelectBlitzTarget => ClientStateId::SelectBlitzTarget,
        TurnMode::SelectGazeTarget => ClientStateId::SelectGazeTarget,
        TurnMode::SafePairOfHands => ClientStateId::PlaceBall,
        TurnMode::SelectBlockKind => ClientStateId::SelectBlockKind,

        TurnMode::IllegalSubstitution => ClientStateId::IllegalSubstitution,

        TurnMode::BombHome | TurnMode::BombAway => ClientStateId::Bomb,
        TurnMode::BombHomeBlitz | TurnMode::BombAwayBlitz => ClientStateId::Bomb,

        TurnMode::RaidingParty => ClientStateId::RaidingParty,
        TurnMode::HitAndRun => ClientStateId::HitAndRun,
        TurnMode::Trickster => ClientStateId::Trickster,
        TurnMode::ThenIStartedBlastin => ClientStateId::ThenIStartedBlastin,

        TurnMode::NoPlayersToField => ClientStateId::WaitForSetup,

        TurnMode::EndGame => ClientStateId::WaitForOpponent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
        }
    }

    fn make_game_in_mode(mode: TurnMode) -> ffb_model::model::game::Game {
        let mut game = ffb_model::model::game::Game::new(
            make_team("home"), make_team("away"), ffb_model::enums::Rules::Bb2020,
        );
        game.turn_mode = mode;
        game
    }

    #[test]
    fn start_game_state() {
        let game = make_game_in_mode(TurnMode::StartGame);
        assert_eq!(current_state(&game, "home"), ClientStateId::StartGame);
    }

    #[test]
    fn regular_mode_our_turn_is_select_player() {
        let game = make_game_in_mode(TurnMode::Regular);
        // home_playing = true by default in a new game
        assert_eq!(current_state(&game, "home"), ClientStateId::SelectPlayer);
    }

    #[test]
    fn regular_mode_opponent_turn_is_wait() {
        let game = make_game_in_mode(TurnMode::Regular);
        assert_eq!(current_state(&game, "away"), ClientStateId::WaitForOpponent);
    }

    #[test]
    fn setup_mode_our_turn_is_setup() {
        let mut game = make_game_in_mode(TurnMode::Setup);
        game.home_playing = true;
        assert_eq!(current_state(&game, "home"), ClientStateId::Setup);
    }

    #[test]
    fn setup_mode_opponent_turn_is_wait_for_setup() {
        let game = make_game_in_mode(TurnMode::Setup);
        assert_eq!(current_state(&game, "away"), ClientStateId::WaitForSetup);
    }

    #[test]
    fn kickoff_mode_is_kickoff_state() {
        let game = make_game_in_mode(TurnMode::Kickoff);
        assert_eq!(current_state(&game, "home"), ClientStateId::Kickoff);
        assert_eq!(current_state(&game, "away"), ClientStateId::Kickoff);
    }

    #[test]
    fn end_game_mode_is_wait_for_opponent() {
        let game = make_game_in_mode(TurnMode::EndGame);
        assert_eq!(current_state(&game, "home"), ClientStateId::WaitForOpponent);
    }
}
