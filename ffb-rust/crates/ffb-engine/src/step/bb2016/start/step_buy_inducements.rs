/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.start.StepBuyInducements`.
///
/// Step in start game sequence to buy inducements (BB2016).
/// - Receives INDUCEMENT_GOLD_HOME / INDUCEMENT_GOLD_AWAY from preceding StepBuyCards.
/// - If INDUCEMENTS option disabled: skip to leaveStep.
/// - If USE_PREDEFINED_INDUCEMENTS: auto-apply team inducement sets.
/// - If gold < 50,000: mark that team done.
/// - Shows dialog for each team still buying.
/// - On CLIENT_BUY_INDUCEMENTS: apply inducement set, add star players / mercenaries.
/// - leaveStep: push Inducement + RiotousRookies sequences; record petty_cash_used.
///
/// Receives: INDUCEMENT_GOLD_HOME, INDUCEMENT_GOLD_AWAY.
///
/// InducementTypeFactory not ported; headless auto-skips inducement buying (no dialog, no predefined inducements).
/// headless: BuyInducements action lacks star_player_position_ids / mercenary_position_ids —
///   add_star_players / add_mercenaries are implemented but require extended action fields to call.
use std::collections::HashMap;
use ffb_model::data::loader::find_position;
use ffb_model::enums::{InducementPhase, PlayerType, PlayerState, PS_RESERVE};
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::skill_def::{SkillId, SkillWithValue};
use ffb_model::model::turn_data::TurnData;
use ffb_model::option::game_option_id::{INDUCEMENTS, USE_PREDEFINED_INDUCEMENTS, ALLOW_STAR_ON_BOTH_TEAMS};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_box::UtilBox;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::common::inducement::{Inducement, InducementParams};
use crate::step::generator::common::riotous_rookies::RiotousRookies;
use crate::step::game::start::util_inducement_sequence::UtilInducementSequence;

const MINIMUM_PETTY_CASH_FOR_INDUCEMENTS: i32 = 50_000;

/// Java: `StepBuyInducements` (bb2016/start).
pub struct StepBuyInducements {
    /// Java: `fInducementGoldHome`
    inducement_gold_home: i32,
    /// Java: `fInducementGoldAway`
    inducement_gold_away: i32,
    /// Java: `fInducementsSelectedHome`
    inducements_selected_home: bool,
    /// Java: `fInducementsSelectedAway`
    inducements_selected_away: bool,
    /// Java: `fGoldUsedHome`
    gold_used_home: i32,
    /// Java: `fGoldUsedAway`
    gold_used_away: i32,
    /// Java: `fReportedHome`
    reported_home: bool,
    /// Java: `fReportedAway`
    reported_away: bool,
}

impl StepBuyInducements {
    pub fn new() -> Self {
        Self {
            inducement_gold_home: 0,
            inducement_gold_away: 0,
            inducements_selected_home: false,
            inducements_selected_away: false,
            gold_used_home: 0,
            gold_used_away: 0,
            reported_home: false,
            reported_away: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: if (!INDUCEMENTS) → leaveStep (skip inducement buying entirely)
        if !is_option_enabled(game, INDUCEMENTS) {
            self.inducements_selected_home = true;
            self.inducements_selected_away = true;
            return self.leave_step(game);
        }

        // Java: if (USE_PREDEFINED_INDUCEMENTS) → apply predefined sets, skip dialog
        // no-op: InducementTypeFactory not ported — headless auto-skips inducement dialog
        if is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            self.inducements_selected_home = true;
            self.inducements_selected_away = true;
            return self.leave_step(game);
        }
        // Auto-skip if under minimum.
        if self.inducement_gold_home < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_home = true;
        }
        if self.inducement_gold_away < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_away = true;
        }
        // client-only: show inducement buying dialog — headless auto-skips
        if self.inducements_selected_home && self.inducements_selected_away {
            return self.leave_step(game);
        }
        StepOutcome::cont()
    }

    fn leave_step(&self, game: &mut Game) -> StepOutcome {
        let home_tv = game.game_result.home.team_value;
        let away_tv = game.game_result.away.team_value;
        let (first_home, second_home) = if home_tv > away_tv { (true, false) } else { (false, true) };
        let seq1 = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: first_home,
            check_forgo: false,
        });
        let seq2 = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: second_home,
            check_forgo: false,
        });
        let seq_rr = RiotousRookies::build_sequence();
        // Java: game.getTeamHome/Away().getTeamData().setPettyCashUsed(UtilInducementSequence.calculateInducementGold(...))
        game.game_result.home.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), true);
        game.game_result.away.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), false);
        StepOutcome::next().push_seq(seq1).push_seq(seq2).push_seq(seq_rr)
    }

    /// Java: `addStarPlayers(Team, String[] positionIds)` — creates new Star players from roster
    /// positions and adds them to the team. If ALLOW_STAR_ON_BOTH_TEAMS is disabled, removes any
    /// duplicate star player from the opposing team.
    ///
    /// engine: GameEvent::PlayerAdded emitted (Phase ZT — requires method to propagate events).
    /// headless: DB update — persistence layer out of scope.
    /// no-op: ReportDoubleHiredStarPlayer — reports deferred.
    fn add_star_players(&self, game: &mut Game, home: bool, position_ids: &[String]) {
        let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let roster_id = if home { game.team_home.roster_id.clone() } else { game.team_away.roster_id.clone() };
        let allow_both = is_option_enabled(game, ALLOW_STAR_ON_BOTH_TEAMS);

        // Collect other team's star players by name for duplicate detection.
        let other_star_by_name: HashMap<String, String> = {
            let other = if home { &game.team_away } else { &game.team_home };
            other.players.iter()
                .filter(|p| p.player_type == PlayerType::Star)
                .map(|p| (p.name.clone(), p.id.clone()))
                .collect()
        };

        let mut to_add: Vec<Player> = Vec::new();
        let mut removed_ids: Vec<String> = Vec::new();
        let mut current_max_nr = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.nr).max().unwrap_or(0)
        };

        for position_id in position_ids {
            let position = match find_position(&roster_id, position_id, game.rules) {
                Some(p) => p,
                None => continue,
            };
            if !allow_both {
                if let Some(other_id) = other_star_by_name.get(&position.name) {
                    removed_ids.push(other_id.clone());
                    continue;
                }
            }
            current_max_nr += 1;
            let idx = to_add.len() + 1;
            let mut player = Player::default();
            player.id = format!("{}S{}", team_id, idx);
            player.name = position.name.clone();
            player.nr = current_max_nr;
            player.position_id = position.id.clone();
            player.player_type = PlayerType::Star;
            player.movement = position.ma;
            player.strength = position.st;
            player.agility = position.ag;
            player.passing = position.pa;
            player.armour = position.av;
            player.position_movement = position.ma;
            player.position_strength = position.st;
            player.position_agility = position.ag;
            player.position_passing = position.pa;
            player.position_armour = position.av;
            player.starting_skills = position.skills.iter()
                .filter_map(|e| SkillId::from_class_name(e.name()).map(|skill_id| SkillWithValue { skill_id, value: None }))
                .collect();
            to_add.push(player);
        }

        // Apply removals from the other team.
        let removed_count = removed_ids.len() as i32;
        for id in &removed_ids {
            if home {
                game.team_away.players.retain(|p| &p.id != id);
            } else {
                game.team_home.players.retain(|p| &p.id != id);
            }
            game.field_model.remove_player(id);
            // no-op: server.getCommunication().sendRemovePlayer() — headless engine
            // no-op: getResult().addReport(new ReportDoubleHiredStarPlayer(...)) — headless engine
        }
        if removed_count > 0 {
            Self::remove_star_player_inducements(&mut game.turn_data_home, removed_count);
            Self::remove_star_player_inducements(&mut game.turn_data_away, removed_count);
        }

        // Apply additions.
        for player in to_add {
            let player_id = player.id.clone();
            if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
            game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            UtilBox::put_player_into_box(game, &player_id);
        }
        // engine: emit GameEvent::PlayerAdded per player (Phase ZT — requires method to propagate events)
    }

    /// Java: `addMercenaries(Team, String[] positionIds, Skill[] skills)` — creates MERCENARY
    /// players with the Loner skill (and optional extra skill per slot) from roster positions.
    ///
    fn add_mercenaries(&self, game: &mut Game, home: bool, position_ids: &[String], extra_skills: &[Option<SkillId>]) {
        if position_ids.is_empty() || extra_skills.is_empty() {
            return;
        }
        let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let roster_id = if home { game.team_home.roster_id.clone() } else { game.team_away.roster_id.clone() };

        let mut to_add: Vec<Player> = Vec::new();
        let mut nr_by_pos: HashMap<String, i32> = HashMap::new();
        let mut current_max_nr = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.nr).max().unwrap_or(0)
        };

        for (i, position_id) in position_ids.iter().enumerate() {
            let position = match find_position(&roster_id, position_id, game.rules) {
                Some(p) => p,
                None => continue,
            };
            current_max_nr += 1;
            let merc_nr = {
                let entry = nr_by_pos.entry(position.id.clone()).or_insert(0);
                *entry += 1;
                *entry
            };
            let idx = to_add.len() + 1;
            let mut player = Player::default();
            player.id = format!("{}M{}", team_id, idx);
            player.name = format!("Merc {} {}", position.name, merc_nr);
            player.nr = current_max_nr;
            player.position_id = position.id.clone();
            player.player_type = PlayerType::Mercenary;
            player.movement = position.ma;
            player.strength = position.st;
            player.agility = position.ag;
            player.passing = position.pa;
            player.armour = position.av;
            player.position_movement = position.ma;
            player.position_strength = position.st;
            player.position_agility = position.ag;
            player.position_passing = position.pa;
            player.position_armour = position.av;
            player.starting_skills = position.skills.iter()
                .filter_map(|e| SkillId::from_class_name(e.name()).map(|skill_id| SkillWithValue { skill_id, value: None }))
                .collect();
            player.add_skill(SkillId::Loner);
            if let Some(Some(skill)) = extra_skills.get(i) {
                player.add_skill(*skill);
            }
            to_add.push(player);
        }

        for player in to_add {
            let player_id = player.id.clone();
            if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
            game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            UtilBox::put_player_into_box(game, &player_id);
        }
        // engine: emit GameEvent::PlayerAdded per player (Phase ZT — requires method to propagate events)
    }

    /// Java: `removeStarPlayerInducements(TurnData, int removed)` — reduces the star-player
    /// inducement count; removes the inducement entirely if the count reaches zero or below.
    fn remove_star_player_inducements(turn_data: &mut TurnData, removed: i32) {
        if let Some(type_id) = turn_data.inducement_set.for_usage(Usage::STAR).map(|s| s.to_string()) {
            let current = turn_data.inducement_set.get(&type_id).map_or(0, |i| i.get_value());
            let new_val = current - removed;
            if new_val <= 0 {
                turn_data.inducement_set.remove_inducement(&type_id);
            } else if let Some(mut ind) = turn_data.inducement_set.get(&type_id) {
                ind.value = new_val;
                turn_data.inducement_set.add_inducement(ind);
            }
        }
    }
}

impl Default for StepBuyInducements {
    fn default() -> Self { Self::new() }
}

impl Step for StepBuyInducements {
    fn id(&self) -> StepId { StepId::BuyInducements }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::BuyInducements { .. } => {
                // Java: CLIENT_BUY_INDUCEMENTS → handleBuyInducements → addStarPlayers, addMercenaries.
                // headless: Action::BuyInducements lacks star_player_position_ids / mercenary_position_ids;
                //   add_star_players and add_mercenaries are implemented but the call-site requires
                //   extended action fields not yet present in the Rust action model.
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InducementGoldHome(v) => { self.inducement_gold_home = *v; true }
            StepParameter::InducementGoldAway(v) => { self.inducement_gold_away = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::inducement::inducement::Inducement as InducementModel;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn make_game_with_rosters() -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        game.team_home.roster_id = "human".into();
        game.team_away.roster_id = "orc".into();
        game
    }

    #[test]
    fn id_is_buy_inducements() {
        assert_eq!(StepBuyInducements::new().id(), StepId::BuyInducements);
    }

    #[test]
    fn both_under_minimum_skips_to_next() {
        let mut game = make_game();
        let mut step = StepBuyInducements::new();
        // Both teams have 0 gold → both auto-selected
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_inducement_gold_home() {
        let mut step = StepBuyInducements::new();
        assert!(step.set_parameter(&StepParameter::InducementGoldHome(100_000)));
        assert_eq!(step.inducement_gold_home, 100_000);
    }

    #[test]
    fn set_parameter_inducement_gold_away() {
        let mut step = StepBuyInducements::new();
        assert!(step.set_parameter(&StepParameter::InducementGoldAway(75_000)));
        assert_eq!(step.inducement_gold_away, 75_000);
    }

    #[test]
    fn both_rich_returns_continue_when_inducements_enabled() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game();
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.inducement_gold_home = 150_000;
        step.inducement_gold_away = 150_000;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Dialog would be shown — until generator deferred, fall to Continue
        assert!(matches!(out.action, StepAction::Continue));
    }

    #[test]
    fn both_under_minimum_pushes_three_sequences() {
        let mut game = make_game();
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Two Inducement sequences + one RiotousRookies
        assert_eq!(out.pushes.len(), 3);
    }

    #[test]
    fn leave_step_sets_petty_cash_used() {
        let mut game = make_game();
        // Set TV diff so home gets petty cash
        game.game_result.home.team_value = 800_000;
        game.game_result.away.team_value = 1_000_000;
        game.game_result.away.petty_cash_transferred = 0;
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        // UtilInducementSequence: home gets 200k petty cash
        assert_eq!(game.game_result.home.petty_cash_used, 200_000);
        assert_eq!(game.game_result.away.petty_cash_used, 0);
    }

    #[test]
    fn inducements_disabled_skips_to_next_step() {
        let mut game = make_game();
        // INDUCEMENTS not set → disabled → skip to leaveStep immediately
        let mut step = StepBuyInducements::new();
        step.inducement_gold_home = 150_000;
        step.inducement_gold_away = 150_000;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    // ── add_star_players tests ────────────────────────────────────────────────

    #[test]
    fn add_star_players_empty_list_does_nothing() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let before = game.team_home.players.len();
        step.add_star_players(&mut game, true, &[]);
        assert_eq!(game.team_home.players.len(), before);
    }

    #[test]
    fn add_star_players_unknown_position_does_nothing() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let before = game.team_home.players.len();
        step.add_star_players(&mut game, true, &["unknown_position_id".into()]);
        assert_eq!(game.team_home.players.len(), before);
    }

    #[test]
    fn add_star_players_adds_player_with_star_type() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        // "human_lineman" is a valid position in the bb2016 human roster.
        // We use it here as a proxy for a star-player position (type irrelevant to the method logic).
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            let added = game.team_home.players.last().unwrap();
            assert_eq!(added.player_type, PlayerType::Star);
        }
    }

    #[test]
    fn add_star_players_sets_id_with_team_prefix() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            let added = game.team_home.players.last().unwrap();
            assert!(added.id.starts_with("home"), "id '{}' should start with team id", added.id);
        }
    }

    #[test]
    fn add_star_players_sets_stats_from_position() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            let pos = find_position("human", pos_id, Rules::Bb2016).unwrap();
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            let added = game.team_home.players.last().unwrap();
            assert_eq!(added.movement, pos.ma);
            assert_eq!(added.strength, pos.st);
            assert_eq!(added.agility, pos.ag);
        }
    }

    #[test]
    fn add_star_players_placed_in_reserve() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            let added = game.team_home.players.last().unwrap();
            let state = game.field_model.player_state(&added.id);
            assert!(state.map_or(false, |s| s.base() == PS_RESERVE));
        }
    }

    #[test]
    fn add_star_players_removes_duplicate_from_other_team_when_option_disabled() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            let pos = find_position("human", pos_id, Rules::Bb2016).unwrap();
            // Place a star player with the same name on the away team.
            let mut other_star = Player::default();
            other_star.id = "away_star_1".into();
            other_star.name = pos.name.clone();
            other_star.player_type = PlayerType::Star;
            other_star.nr = 99;
            game.team_away.players.push(other_star.clone());
            game.field_model.set_player_state("away_star_1", PlayerState::new(PS_RESERVE));

            // ALLOW_STAR_ON_BOTH_TEAMS not set → duplicate removal active.
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            assert!(!game.team_away.players.iter().any(|p| p.id == "away_star_1"),
                "duplicate star should be removed from away team");
            // Home team should NOT gain a new star (it was a duplicate).
            assert!(!game.team_home.players.iter().any(|p| p.player_type == PlayerType::Star),
                "no star should be added when duplicate found and option disabled");
        }
    }

    #[test]
    fn add_star_players_keeps_duplicate_when_option_enabled() {
        use ffb_model::option::game_option_id::ALLOW_STAR_ON_BOTH_TEAMS;
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        game.options.set(ALLOW_STAR_ON_BOTH_TEAMS, "true");
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            let pos = find_position("human", pos_id, Rules::Bb2016).unwrap();
            let mut other_star = Player::default();
            other_star.id = "away_star_1".into();
            other_star.name = pos.name.clone();
            other_star.player_type = PlayerType::Star;
            other_star.nr = 99;
            game.team_away.players.push(other_star);

            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            assert!(game.team_away.players.iter().any(|p| p.id == "away_star_1"),
                "other team's star should be kept when ALLOW_STAR_ON_BOTH_TEAMS enabled");
            assert!(game.team_home.players.iter().any(|p| p.player_type == PlayerType::Star),
                "home team should gain the star");
        }
    }

    #[test]
    fn add_star_players_wires_starting_skills() {
        use ffb_model::enums::Rules;
        use ffb_model::data::loader::find_position;
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("skaven");
        // Skaven have skills like Block on linemen; find a position with skills
        let pos_with_skills = positions.iter()
            .find(|id| find_position("skaven", id, Rules::Bb2016)
                .map(|p| !p.skills.is_empty())
                .unwrap_or(false));
        if let Some(pos_id) = pos_with_skills {
            let pos = find_position("skaven", pos_id, Rules::Bb2016).unwrap();
            step.add_star_players(&mut game, true, &[pos_id.clone()]);
            let added = game.team_home.players.last().unwrap();
            assert!(!added.starting_skills.is_empty(),
                "star player should have starting_skills from position; position had {:?}", pos.skills);
        }
    }

    #[test]
    fn add_mercenaries_wires_starting_skills() {
        use ffb_model::enums::Rules;
        use ffb_model::data::loader::find_position;
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("skaven");
        let pos_with_skills = positions.iter()
            .find(|id| find_position("skaven", id, Rules::Bb2016)
                .map(|p| !p.skills.is_empty())
                .unwrap_or(false));
        if let Some(pos_id) = pos_with_skills {
            let pos = find_position("skaven", pos_id, Rules::Bb2016).unwrap();
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[None]);
            let added = game.team_home.players.last().unwrap();
            assert!(!added.starting_skills.is_empty(),
                "mercenary should have starting_skills from position; position had {:?}", pos.skills);
        }
    }

    // ── add_mercenaries tests ─────────────────────────────────────────────────

    #[test]
    fn add_mercenaries_empty_list_does_nothing() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let before = game.team_home.players.len();
        step.add_mercenaries(&mut game, true, &[], &[]);
        assert_eq!(game.team_home.players.len(), before);
    }

    #[test]
    fn add_mercenaries_adds_player_with_mercenary_type() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[None]);
            let added = game.team_home.players.last().unwrap();
            assert_eq!(added.player_type, PlayerType::Mercenary);
        }
    }

    #[test]
    fn add_mercenaries_has_loner_skill() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[None]);
            let added = game.team_home.players.last().unwrap();
            assert!(added.has_skill(SkillId::Loner), "mercenary should have Loner skill");
        }
    }

    #[test]
    fn add_mercenaries_name_contains_merc_prefix() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[None]);
            let added = game.team_home.players.last().unwrap();
            assert!(added.name.starts_with("Merc "), "name '{}' should start with 'Merc '", added.name);
        }
    }

    #[test]
    fn add_mercenaries_extra_skill_added() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[Some(SkillId::Block)]);
            let added = game.team_home.players.last().unwrap();
            assert!(added.has_skill(SkillId::Block), "mercenary should have extra Block skill");
        }
    }

    #[test]
    fn add_mercenaries_placed_in_reserve() {
        let step = StepBuyInducements::new();
        let mut game = make_game_with_rosters();
        let positions = bb2016_position_ids_for("human");
        if let Some(pos_id) = positions.first() {
            step.add_mercenaries(&mut game, true, &[pos_id.clone()], &[None]);
            let added = game.team_home.players.last().unwrap();
            let state = game.field_model.player_state(&added.id);
            assert!(state.map_or(false, |s| s.base() == PS_RESERVE));
        }
    }

    // ── remove_star_player_inducements tests ─────────────────────────────────

    #[test]
    fn remove_star_player_inducements_reduces_value() {
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("starPlayers", 3, vec![Usage::STAR]));
        StepBuyInducements::remove_star_player_inducements(&mut game.turn_data_home, 1);
        assert_eq!(game.turn_data_home.inducement_set.get("starPlayers").map(|i| i.get_value()), Some(2));
    }

    #[test]
    fn remove_star_player_inducements_removes_when_zero() {
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("starPlayers", 1, vec![Usage::STAR]));
        StepBuyInducements::remove_star_player_inducements(&mut game.turn_data_home, 1);
        assert!(game.turn_data_home.inducement_set.get("starPlayers").is_none());
    }

    #[test]
    fn remove_star_player_inducements_no_op_when_not_present() {
        let mut game = make_game();
        // Should not panic when no star inducement present.
        StepBuyInducements::remove_star_player_inducements(&mut game.turn_data_home, 1);
    }

    // ── helper ────────────────────────────────────────────────────────────────

    /// Returns a list of non-star position IDs for the given bb2016 roster.
    fn bb2016_position_ids_for(roster_id: &str) -> Vec<String> {
        use ffb_model::data::loader::bb2016_rosters;
        bb2016_rosters()
            .into_iter()
            .find(|r| r.id == roster_id)
            .map(|r| r.positions.into_iter()
                .filter(|p| p.player_type == "Regular" || p.player_type == "Irregular")
                .map(|p| p.id)
                .collect())
            .unwrap_or_default()
    }
}
