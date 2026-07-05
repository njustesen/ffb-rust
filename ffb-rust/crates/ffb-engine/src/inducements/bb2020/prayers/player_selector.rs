/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.PlayerSelector`.
/// Selects eligible players for prayer enhancements. Extends mixed::prayers::PlayerSelector.
///
/// BB2020: during START_GAME → RESERVE state; otherwise must be on the field (in bounds).
/// Excludes Loner players (hasToRollToUseTeamReroll). Excludes players that already have all addedSkills.
use ffb_model::model::game::Game;
use ffb_model::enums::{TurnMode, PS_RESERVE, SkillId};
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

pub struct PlayerSelector;

impl PlayerSelector {
    pub const INSTANCE: PlayerSelector = PlayerSelector;

    pub fn new() -> Self { Self }
}

impl Default for PlayerSelector {
    fn default() -> Self { Self::new() }
}

impl PlayerSelectorTrait for PlayerSelector {
    /// Java: `eligiblePlayers(Team team, Game game, Set<Skill> skills)` filtered by
    /// `selectPlayers(team, game, amount, skills)`.
    ///
    /// Rules:
    /// - If `TurnMode::StartGame`: eligible if `player_state.base() == PS_RESERVE`
    /// - Otherwise: eligible if `player_coordinate` is in `FieldCoordinateBounds::FIELD`
    /// - Must NOT have the Loner property (hasToRollToUseTeamReroll)
    /// - Must NOT already have all addedSkills
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String> {
        let team = if game.team_home.id == team_id {
            &game.team_home
        } else {
            &game.team_away
        };

        let is_start_game = game.turn_mode == TurnMode::StartGame;

        let mut eligible: Vec<&str> = team.players.iter()
            .filter(|p| {
                if is_start_game {
                    game.field_model.player_state(&p.id)
                        .map_or(false, |s| s.base() == PS_RESERVE)
                } else {
                    game.field_model.player_coordinate(&p.id)
                        .map_or(false, |c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                }
            })
            .filter(|p| !p.has_skill_property(ffb_model::model::property::named_properties::NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL))
            .filter(|p| added_skills.is_empty() || !added_skills.iter().all(|s| p.has_skill(*s)))
            .map(|p| p.id.as_str())
            .collect();

        // Java: for each slot, Collections.shuffle then remove first — equivalent to shuffle-pick.
        // Fisher-Yates shuffle for random selection.
        let n = eligible.len();
        for i in (1..n).rev() {
            let j = rng.range(i + 1);
            eligible.swap(i, j);
        }
        eligible.truncate(nr_of_players as usize);
        eligible.iter().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str, state: PlayerState) {
        use ffb_model::model::player::Player;
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
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn player_selector_default() {
        let _ = PlayerSelector::default();
    }

    #[test]
    fn selects_reserve_player_at_start_game() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["h1".to_string()]);
    }

    #[test]
    fn excludes_standing_player_at_start_game() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_STANDING));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn selects_on_pitch_player_during_regular_play() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(13, 7));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert_eq!(result, vec!["h1".to_string()]);
    }

    #[test]
    fn excludes_off_pitch_player_during_regular_play() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        // No coordinate set → not on field
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn respects_count_limit() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        for i in 0..5 {
            let id = format!("h{i}");
            add_player(&mut game, "home", &id, PlayerState::new(PS_RESERVE));
        }
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 3, &mut GameRng::new(0), &[]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn excludes_loner_players() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::model::skill_def::SkillId;
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        // Give h1 the Loner skill (hasToRollToUseTeamReroll property)
        game.team_home.players[0].extra_skills.push(SkillWithValue { skill_id: SkillId::Loner, value: None });
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut GameRng::new(0), &[]);
        assert!(result.is_empty(), "Loner player should be excluded");
    }
}
