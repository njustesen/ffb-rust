/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.PlayerSelector`.
/// Java: interface PlayerSelector { List<Player<?>> selectPlayers(Team, Game, int, Set<Skill>) }
/// In Rust, modelled as a trait.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;

pub trait PlayerSelector: Send + Sync {
    /// Java: selectPlayers(Team, Game, int nrOfPlayers, Set<Skill> addedSkills)
    /// Returns selected player IDs (up to `nr_of_players`).
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String>;

    /// Java: `determineTeam(Team team, Game game)` — resolves which team is actually
    /// affected by this selector. Default: the praying team itself (identity).
    /// `OpponentPlayerSelector` overrides this to return the opposing team's id.
    fn determine_team_id<'a>(&self, _game: &Game, team_id: &'a str) -> String {
        team_id.to_string()
    }
}

/// Null-object selector that selects no players.
/// Used in tests and as a placeholder when a concrete selector is not yet determined.
/// Concrete implementations: `bb2020::prayers::PlayerSelector`, `bb2020::prayers::OpponentPlayerSelector`.
#[derive(Debug, Default)]
pub struct StubPlayerSelector;

impl PlayerSelector for StubPlayerSelector {
    fn select_players(&self, _game: &Game, _team_id: &str, _nr_of_players: i32, _rng: &mut GameRng, _added_skills: &[SkillId]) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;

    // NOTE (test equalization): stub/object-safety/Default tests pruned - Rust-structural
    // (StubPlayerSelector and the Default impl are Rust test scaffolding with no Java
    // counterpart; Java's abstract PlayerSelector is covered via the edition selector twins).




}
