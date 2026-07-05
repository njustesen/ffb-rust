/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.EnhancementRemover`.
/// Removes prayer-granted temporary enhancements from all players of a team.
///
/// Java: iterates `selector.determineTeam(team, game).getPlayers()` and calls
/// `game.getFieldModel().removePrayerEnhancements(player, prayer)` for each.
use crate::inducements::mixed::prayers::prayer_player_effect::remove_prayer_player_effect;

#[derive(Debug, Default, Clone)]
pub struct EnhancementRemover;

impl EnhancementRemover {
    pub fn new() -> Self {
        Self
    }

    /// Java: `removeEnhancement(GameState, Team, PlayerSelector, Prayer)`.
    /// Removes the prayer enhancement tracking for all players of the team
    /// that the selector resolves (own or opponent).
    pub fn remove_enhancement(
        &self,
        game: &mut ffb_model::model::game::Game,
        team_id: &str,
        prayer_name: &str,
    ) {
        let player_ids: Vec<String> = {
            let team = if game.team_home.id == team_id {
                &game.team_home
            } else {
                &game.team_away
            };
            team.players.iter().map(|p| p.id.clone()).collect()
        };
        for id in player_ids {
            remove_prayer_player_effect(game, &id, prayer_name);
            game.field_model.remove_prayer_enhancement(&id, prayer_name);
        }
    }

    /// Removes the prayer enhancement for a specific player only.
    pub fn remove_enhancement_for_player(
        &self,
        game: &mut ffb_model::model::game::Game,
        player_id: &str,
        prayer_name: &str,
    ) {
        remove_prayer_player_effect(game, player_id, prayer_name);
        game.field_model.remove_prayer_enhancement(player_id, prayer_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str) {
        use ffb_model::model::player::Player;
        use ffb_model::model::player_status::PlayerStatus;
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn remove_enhancement_is_callable() {
        let remover = EnhancementRemover::new();
        let mut game = make_game();
        remover.remove_enhancement(&mut game, "home", "STILETTO");
    }

    #[test]
    fn remove_enhancement_clears_tracking() {
        let remover = EnhancementRemover::new();
        let mut game = make_game();
        add_player(&mut game, "home", "h1");
        game.field_model.add_prayer_enhancement("h1", "STILETTO");
        assert!(game.field_model.has_prayer_enhancement("h1", "STILETTO"));
        remover.remove_enhancement(&mut game, "home", "STILETTO");
        assert!(!game.field_model.has_prayer_enhancement("h1", "STILETTO"));
    }

    #[test]
    fn remove_enhancement_only_affects_own_team() {
        let remover = EnhancementRemover::new();
        let mut game = make_game();
        add_player(&mut game, "home", "h1");
        add_player(&mut game, "away", "a1");
        game.field_model.add_prayer_enhancement("h1", "STILETTO");
        game.field_model.add_prayer_enhancement("a1", "STILETTO");
        remover.remove_enhancement(&mut game, "home", "STILETTO");
        assert!(!game.field_model.has_prayer_enhancement("h1", "STILETTO"));
        assert!(game.field_model.has_prayer_enhancement("a1", "STILETTO"));
    }

    #[test]
    fn remove_enhancement_for_player() {
        let remover = EnhancementRemover::new();
        let mut game = make_game();
        game.field_model.add_prayer_enhancement("h1", "GREASY_CLEATS");
        remover.remove_enhancement_for_player(&mut game, "h1", "GREASY_CLEATS");
        assert!(!game.field_model.has_prayer_enhancement("h1", "GREASY_CLEATS"));
    }

    #[test]
    fn enhancement_remover_is_default_constructible() {
        let _ = EnhancementRemover::default();
    }

    #[test]
    fn enhancement_remover_is_clonable() {
        let a = EnhancementRemover::new();
        let _b = a.clone();
    }
}
