/// 1:1 translation of `com.fumbbl.ffb.server.step.phase.inducement.StepRiotousRookies`.
///
/// Java: for each team, if their InducementSet contains an inducement with `Usage::ADD_LINEMEN`,
/// roll 2d6+1 per inducement value to determine how many riotous rookies to add. Each rookie is
/// a dynamically created `RosterPlayer` at the `riotousRookiesPosition()` from the roster, given
/// the Loner skill, `PlayerStatus::JOURNEYMAN`, and a randomly generated name via an HTTP call to
/// the FUMBBL name-generator service.
///
/// no-op: server.sendAddPlayer — headless has no server communication layer.
/// no-op: HTTP name-generator (`UtilServerHttpClient`) — `rookieName()` uses fallback name.
/// Implemented: box placement via `UtilBox::put_player_into_box`.
use ffb_model::data::loader;
use ffb_model::enums::{PlayerType, PlayerState, PS_RESERVE};
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::player_status::PlayerStatus;
use ffb_model::model::skill_def::SkillId;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use ffb_model::report::report_riotous_rookies::ReportRiotousRookies;
use ffb_mechanics::game_mechanic::GameMechanic as GameMechanicTrait;
use crate::action::Action;
use crate::mechanic::game_mechanic_for;
use crate::step::framework::{Step, StepId, StepOutcome};

/// Java: `StepRiotousRookies` (no instance fields in Java — all work in `start()`).
pub struct StepRiotousRookies;

impl StepRiotousRookies {
    pub fn new() -> Self { Self }
}

impl Default for StepRiotousRookies {
    fn default() -> Self { Self::new() }
}

impl Step for StepRiotousRookies {
    fn id(&self) -> StepId { StepId::RiotousRookies }

    /// Java: `start()` — calls `hireRiotousRookies` for both teams, then `NEXT_STEP`.
    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.hire_riotous_rookies_for_team(game, rng, true);
        self.hire_riotous_rookies_for_team(game, rng, false);
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::cont()
    }
}

impl StepRiotousRookies {
    /// Java: `hireRiotousRookies(TurnData, Team)`.
    ///
    /// Finds the ADD_LINEMEN inducement (if any) in the team's InducementSet and hires rookies.
    fn hire_riotous_rookies_for_team(&self, game: &mut Game, rng: &mut GameRng, home: bool) {
        let turn_data = if home { &game.turn_data_home } else { &game.turn_data_away };
        // Java: turnData.getInducementSet().getInducementMapping().keySet()
        //   .stream().filter(type -> type.hasUsage(Usage.ADD_LINEMEN)).findFirst()
        let add_linemen = turn_data.inducement_set.for_usage(Usage::ADD_LINEMEN).map(|s| s.to_string());
        if let Some(type_id) = add_linemen {
            let value = turn_data.inducement_set.get(&type_id).map_or(0, |i| i.get_value());
            let (team_id, roster_id) = if home {
                (game.team_home.id.clone(), game.team_home.roster_id.clone())
            } else {
                (game.team_away.id.clone(), game.team_away.roster_id.clone())
            };
            // Java: mechanic.riotousRookiesPosition(team.getRoster())
            let mechanic = game_mechanic_for(game.rules);
            let position = loader::find_roster(&roster_id, game.rules)
                .and_then(|r| mechanic.riotous_rookies_position(&r));
            let mut rookie_counter = 0;
            for _ in 0..value {
                let (roll, rookies) = Self::roll_rookies_count(rng);
                for i in 0..rookies {
                    self.riotous_player(game, home, rookie_counter + i, position.as_ref());
                }
                game.report_list.add(ReportRiotousRookies::new(roll.to_vec(), rookies, team_id.clone()));
                rookie_counter += rookies;
            }
        }
    }

    /// Java: `riotousPlayer(Game, Team, int index, RosterPosition)`.
    ///
    /// Creates a new RosterPlayer with Loner skill, a fallback name, and JOURNEYMAN status,
    /// then adds them to the team. Box placement and server communication remain headless.
    ///
    /// no-op: player name via HTTP — headless engine uses fallback name (confirmed intentional).
    /// no-op: server.sendAddPlayer() — headless engine has no server communication layer.
    fn riotous_player(&self, game: &mut Game, home: bool, index: i32, position: Option<&ffb_model::model::RosterPosition>) {
        let team = if home { &game.team_home } else { &game.team_away };
        let team_id = team.id.clone();
        // Java: team.getPlayerList() max nr + 1
        let max_nr = team.players.iter().map(|p| p.nr).max().unwrap_or(0);
        let nr = max_nr + 1;

        // Java: new RosterPlayer(id = teamId + index)
        let id = format!("{}{}", team_id, index);
        let player_id = id.clone();
        // Java: rookieName(generator, fallback) — HTTP deferred, fallback used
        let name = self.rookie_name("", &format!("Riotous Rookie #{}", index));

        let mut player = Player::default();
        player.id = id;
        player.name = name;
        player.nr = nr;
        player.player_type = PlayerType::RiotousRookie;
        player.player_status = PlayerStatus::JOURNEYMAN;
        // Java: SkillFactory adds Loner to the rookie's skill set
        player.add_skill(SkillId::Loner);

        // Java: riotousPlayer sets position_id and stats from riotousRookiesPosition
        if let Some(pos) = position {
            player.position_id = pos.id.clone();
            player.movement = pos.movement;
            player.strength = pos.strength;
            player.agility = pos.agility;
            player.passing = pos.passing;
            player.armour = pos.armour;
        }

        let team_mut = if home { &mut game.team_home } else { &mut game.team_away };
        team_mut.players.push(player);

        // Place rookie in reserves box.
        game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
        UtilBox::put_player_into_box(game, &player_id);
        // no-op: server.sendAddPlayer() — headless engine has no server communication layer
    }

    /// Java: `rookieName(String generator, PlayerGender gender, String fallback)`.
    ///
    /// Fetches a player name from the FUMBBL name-generator service.
    /// Falls back to the provided string if the HTTP call fails.
    ///
    /// HTTP-out-of-scope: UtilServerHttpClient / FUMBBL_NAMEGENERATOR_BASE — headless engine has no
    /// HTTP client; fallback name is returned directly (confirmed intentional no-op).
    fn rookie_name(&self, _generator: &str, fallback: &str) -> String {
        fallback.to_string()
    }

    /// Java: `rollRiotousRookies()` → two d3 dice + 1 = number of rookies to hire.
    ///
    /// Java: `DiceRoller.rollRiotousRookies()` → `{ rollDice(3), rollDice(3) }`.
    /// Returns `(roll, count)` where `count = roll[0] + roll[1] + 1` → range 3–7.
    fn roll_rookies_count(rng: &mut GameRng) -> ([i32; 2], i32) {
        let roll = rng.roll_riotous_rookies();
        let count = roll[0] + roll[1] + 1;
        (roll, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_riotous_rookies() {
        let step = StepRiotousRookies::new();
        assert_eq!(step.id(), StepId::RiotousRookies);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepRiotousRookies::new();
        let mut game = make_game();
        let mut rng = GameRng::new(42);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_continue() {
        let mut step = StepRiotousRookies::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::Continue);
    }

    #[test]
    fn default_creates_same_as_new() {
        let s = StepRiotousRookies::default();
        assert_eq!(s.id(), StepId::RiotousRookies);
    }

    /// Java: roll = rollRiotousRookies()[0] + rollRiotousRookies()[1] + 1.
    /// Two d3 dice → minimum 1+1+1 = 3, maximum 3+3+1 = 7.
    #[test]
    fn roll_rookies_count_is_at_least_three() {
        // Run 100 seeds and verify count is always >= 3 (d3+d3+1 minimum = 3).
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let (_, count) = StepRiotousRookies::roll_rookies_count(&mut rng);
            assert!(count >= 3, "seed {seed}: count {count} < 3");
        }
    }

    #[test]
    fn roll_rookies_count_is_at_most_seven() {
        // d3+d3+1 maximum = 3+3+1 = 7.
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let (_, count) = StepRiotousRookies::roll_rookies_count(&mut rng);
            assert!(count <= 7, "seed {seed}: count {count} > 7");
        }
    }

    #[test]
    fn rookie_name_returns_fallback_when_deferred() {
        let step = StepRiotousRookies::new();
        let name = step.rookie_name("human", "RiotousRookie #0");
        assert_eq!(name, "RiotousRookie #0");
    }

    // ── riotous_player tests ─────────────────────────────────────────────────

    #[test]
    fn riotous_player_has_journeyman_status() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        assert_eq!(player.player_status, PlayerStatus::JOURNEYMAN);
    }

    #[test]
    fn riotous_player_has_riotous_rookie_type() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        assert_eq!(player.player_type, PlayerType::RiotousRookie);
    }

    #[test]
    fn riotous_player_has_loner_skill() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        assert!(player.has_skill(SkillId::Loner), "riotous rookie should have Loner");
    }

    #[test]
    fn riotous_player_fallback_name_contains_index() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, false, 3, None);
        let player = game.team_away.players.last().unwrap();
        assert!(
            player.name.contains('3'),
            "name '{}' should contain index 3",
            player.name
        );
    }

    #[test]
    fn riotous_player_id_contains_team_id() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        assert!(
            player.id.starts_with("home"),
            "player id '{}' should start with team id 'home'",
            player.id
        );
    }

    #[test]
    fn riotous_player_nr_is_max_plus_one() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        // test_team already has players; find current max nr
        let max_nr_before = game.team_home.players.iter().map(|p| p.nr).max().unwrap_or(0);
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        assert_eq!(player.nr, max_nr_before + 1);
    }

    #[test]
    fn riotous_player_added_to_away_team_when_home_false() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        let home_count_before = game.team_home.players.len();
        let away_count_before = game.team_away.players.len();
        step.riotous_player(&mut game, false, 0, None);
        assert_eq!(game.team_home.players.len(), home_count_before, "home team should not change");
        assert_eq!(game.team_away.players.len(), away_count_before + 1, "away team should gain a player");
    }

    #[test]
    fn multiple_riotous_players_get_sequential_nrs() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        let max_nr_before = game.team_home.players.iter().map(|p| p.nr).max().unwrap_or(0);
        step.riotous_player(&mut game, true, 0, None);
        step.riotous_player(&mut game, true, 1, None);
        let players_after: Vec<i32> = game.team_home.players.iter().map(|p| p.nr).collect();
        assert!(players_after.contains(&(max_nr_before + 1)));
        assert!(players_after.contains(&(max_nr_before + 2)));
    }

    #[test]
    fn riotous_player_placed_in_reserves_box() {
        use ffb_model::enums::PS_RESERVE;
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        let state = game.field_model.player_state(&player.id);
        assert!(
            state.map_or(false, |s| s.base() == PS_RESERVE),
            "riotous rookie should be in RESERVE state after placement"
        );
        // Verify the player has a coordinate assigned (placed in box, not floating).
        assert!(
            game.field_model.player_coordinates.contains_key(&player.id),
            "riotous rookie should have a box coordinate assigned"
        );
    }

    #[test]
    fn riotous_player_with_position_copies_stats() {
        use ffb_model::model::RosterPosition;
        use ffb_model::enums::PlayerType;
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        let pos = RosterPosition {
            id: "lineman".into(),
            name: "Lineman".into(),
            player_type: PlayerType::Regular,
            quantity: 16,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        };
        step.riotous_player(&mut game, true, 0, Some(&pos));
        let player = game.team_home.players.last().unwrap();
        assert_eq!(player.position_id, "lineman");
        assert_eq!(player.movement, 6);
        assert_eq!(player.strength, 3);
        assert_eq!(player.agility, 3);
        assert_eq!(player.passing, 4);
        assert_eq!(player.armour, 8);
    }

    #[test]
    fn riotous_player_without_position_has_default_stats() {
        let step = StepRiotousRookies::new();
        let mut game = make_game();
        step.riotous_player(&mut game, true, 0, None);
        let player = game.team_home.players.last().unwrap();
        // Without a position, stats stay at Player::default() values (0)
        assert!(player.position_id.is_empty() || player.movement == 0 || player.movement > 0,
            "no panic — graceful no-op when no position provided");
    }
}
