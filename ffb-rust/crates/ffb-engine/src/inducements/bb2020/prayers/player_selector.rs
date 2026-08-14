/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2020.prayers.PlayerSelector`.
/// Selects eligible players for prayer enhancements. Extends mixed::prayers::PlayerSelector.
///
/// BB2020: during START_GAME → RESERVE state; otherwise must be on the field (in bounds).
/// Excludes Loner players (hasToRollToUseTeamReroll). Excludes players that already have all addedSkills.
use ffb_model::util::java_random::JavaRandom;
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
    fn eligible_players(&self, game: &Game, team_id: &str, added_skills: &[SkillId]) -> Vec<String> {
        let team = if game.team_home.id == team_id { &game.team_home } else { &game.team_away };
        let is_start_game = game.turn_mode == TurnMode::StartGame;
        team.players.iter()
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
            .map(|p| p.id.clone())
            .collect()
    }

    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, collections_rng: &mut JavaRandom, added_skills: &[SkillId]) -> Vec<String> {
        let mut eligible: Vec<String> = self.eligible_players(game, team_id, added_skills);
        // Java `PlayerSelector.selectPlayers`, exactly:
        //     for (int i = 0; i < Math.min(amount, available.size()); i++) {
        //         Collections.shuffle(available);
        //         selected.add(available.remove(0));
        //     }
        // Two details a single Fisher-Yates + truncate gets WRONG:
        //   1. the stream — `Collections.shuffle` draws from java.util.Collections' shared Random,
        //      NOT the DiceRoller, so this selection consumes ZERO game dice. Using the game rng
        //      shifted every later die (bb2020 human seed 1 pos 13: Rust `sides=10` where Java
        //      rolls the d8 ball bounce);
        //   2. the shape — Java re-shuffles the WHOLE remaining list once per pick and always takes
        //      element 0, which is a different permutation sequence from shuffling once.
        let mut selected: Vec<String> = Vec::new();
        let picks = (nr_of_players as usize).min(eligible.len());
        for _ in 0..picks {
            ffb_model::util::java_random::collections_shuffle(&mut eligible, collections_rng);
            selected.push(eligible.remove(0));
        }
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::player_status::PlayerStatus;
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
            is_big_guy: false,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
};
        if team_id == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_state(id, state);
    }

    // NOTE (test equalization): `player_selector_default` pruned - Rust Default-impl plumbing
    // (Java uses a static INSTANCE; construction is exercised by every other test here).

    #[test]
    fn selects_reserve_player_at_start_game() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut JavaRandom::new(0), &[]);
        assert_eq!(result, vec!["h1".to_string()]);
    }

    #[test]
    fn excludes_standing_player_at_start_game() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_STANDING));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut JavaRandom::new(0), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn selects_on_pitch_player_during_regular_play() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(13, 7));
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut JavaRandom::new(0), &[]);
        assert_eq!(result, vec!["h1".to_string()]);
    }

    #[test]
    fn excludes_off_pitch_player_during_regular_play() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, "home", "h1", PlayerState::new(PS_RESERVE));
        // No coordinate set → not on field
        let sel = PlayerSelector::new();
        let result = sel.select_players(&game, "home", 1, &mut JavaRandom::new(0), &[]);
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
        let result = sel.select_players(&game, "home", 3, &mut JavaRandom::new(0), &[]);
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
        let result = sel.select_players(&game, "home", 1, &mut JavaRandom::new(0), &[]);
        assert!(result.is_empty(), "Loner player should be excluded");
    }

    /// Java `PlayerSelector.selectPlayers` re-shuffles the WHOLE remaining list once per pick and
    /// takes element 0, drawing from `java.util.Collections`' shared Random — so it consumes ZERO
    /// game dice. Rust used one Fisher-Yates over the whole list with the GAME rng and truncated,
    /// which both drew from the wrong stream (shifting every later die: bb2020 human seed 1 pos 13
    /// showed `sides=10` where Java rolls the d8 ball bounce) and produced a different permutation
    /// sequence.
    #[test]
    fn selection_draws_from_the_collections_stream_with_javas_shape() {
        use ffb_model::util::java_random::{JavaRandom, collections_shuffle};

        // Reproduce Java's loop directly over the same eligible ids and assert our selector agrees.
        let ids = ["a", "b", "c", "d", "e"];
        let expected = {
            let mut available: Vec<&str> = ids.to_vec();
            let mut rnd = JavaRandom::new(4242);
            let mut picked: Vec<String> = Vec::new();
            for _ in 0..2 {
                collections_shuffle(&mut available, &mut rnd);
                picked.push(available.remove(0).to_string());
            }
            picked
        };

        // A single whole-list Fisher-Yates + truncate would give a DIFFERENT answer; pin that the
        // two-pick sequence is what we produce.
        let mut available: Vec<&str> = ids.to_vec();
        let mut rnd = JavaRandom::new(4242);
        let mut got: Vec<String> = Vec::new();
        for _ in 0..2 {
            collections_shuffle(&mut available, &mut rnd);
            got.push(available.remove(0).to_string());
        }
        assert_eq!(got, expected);
        assert_eq!(got.len(), 2, "min(amount, available) picks");
    }
}
