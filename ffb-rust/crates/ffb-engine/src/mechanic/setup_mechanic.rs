/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.SetupMechanic` (abstract base).
///
/// Java abstract class → Rust trait. Concrete edition implementations:
///   mixed::SetupMechanic (BB2016/BB2020), bb2025::SetupMechanic (BB2025).
use ffb_model::model::game::Game;

pub trait SetupMechanic: Send + Sync {
    /// Java: `checkSetup(GameState, boolean pHomeTeam)`.
    fn check_setup(&self, game: &mut Game, home_team: bool) -> bool;

    /// Java: `checkSetup(GameState, boolean, int additionalSwarmers)`.
    fn check_setup_with_swarmers(
        &self,
        game: &mut Game,
        home_team: bool,
        additional_swarmers: i32,
    ) -> bool;

    /// Java: `pinPlayersInTacklezones(GameState, Team)`.
    fn pin_players_in_tacklezones(&self, game: &mut Game, team_id: &str);

    /// Java: `pinPlayersInTacklezones(GameState, Team, boolean pinBallAndChain)`.
    fn pin_players_in_tacklezones_chain(
        &self,
        game: &mut Game,
        team_id: &str,
        pin_ball_and_chain: bool,
    );
}
