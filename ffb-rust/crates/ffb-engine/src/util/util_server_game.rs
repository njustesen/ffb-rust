// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerGame.
//
// Methods that touch DB, HTTP, WebSocket, IStep, or the Java server's
// DiceRoller/Communication layers are skipped (marked `—` not translating).
// The pure game-model helpers are translated here.
//
// Translated methods:
//   prepare_for_setup(game)                            — two-team wrapper
//   prepare_for_setup_team(game, team_id)              — per-team RESERVE/SETUP_PREVENTED logic
//   update_single_use_rerolls(turn_data, team, fm)     — count single-use reroll providers
//   update_player_state_dependent_properties(game)     — leader re-rolls + single-use re-rolls
//                                                         (checkForMissingPartners headless)
//   mark_played_and_secret_weapons(game)               — per-drive turns_played + has_used_secret_weapon
//
// Skipped (touch IStep / server / WebSocket):
//   syncGameModel, changeActingPlayer, handleChefRolls, rollMasterChef,
//   checkForWastedSkills, addPartnerEnhancement, closeGame, handleInvalidTeam
//
// Partially implemented (checkForMissingPartners skipped — needs addSkillEnhancements):
//   updatePlayerStateDependentProperties, checkForMissingPartners

use ffb_model::enums::{PlayerState, PS_BANNED, PS_KNOCKED_OUT, PS_RESERVE, PS_SETUP_PREVENTED, SkillId, TurnMode};
use ffb_model::report::bb2020::report_two_for_one::ReportTwoForOne;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::team::Team;
use ffb_model::model::turn_data::TurnData;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use crate::mechanic::{state_mechanic_for, state_mechanic::StateMechanic as StateMechanicTrait};

pub struct UtilServerGame;

impl UtilServerGame {
    /// Java: UtilServerGame.prepareForSetup(Game)
    /// Calls the per-team helper for both home and away teams.
    pub fn prepare_for_setup(game: &mut Game) {
        let home_id = game.team_home.id.clone();
        let away_id = game.team_away.id.clone();
        Self::prepare_for_setup_team(game, &home_id);
        Self::prepare_for_setup_team(game, &away_id);
    }

    /// Java: private static void prepareForSetup(Game game, Team team)
    ///
    /// Groups the team's RESERVE players into "keen"
    /// (canJoinTeamIfLessThanEleven) and "non-keen".  If there are ≥11
    /// non-keen RESERVE players, all keen RESERVE players are moved to
    /// SETUP_PREVENTED. Callers: StepInitKickoff (first drive) and, for bb2020/bb2025,
    /// StepEndTurn's end-of-drive block (BACKLOG §M 2b(n), Josef Bugman's Keen Player).
    fn prepare_for_setup_team(game: &mut Game, team_id: &str) {
        let player_info: Vec<(String, bool)> = {
            let team = if game.team_home.id == team_id {
                &game.team_home
            } else {
                &game.team_away
            };
            team.players
                .iter()
                .filter(|p| {
                    game.field_model
                        .player_state(&p.id)
                        .map(|s| s.base() == PS_RESERVE)
                        .unwrap_or(false)
                })
                .map(|p| {
                    let keen = p.has_skill_property(NamedProperties::CAN_JOIN_TEAM_IF_LESS_THAN_ELEVEN);
                    (p.id.clone(), keen)
                })
                .collect()
        };

        let non_keen_count = player_info.iter().filter(|(_, keen)| !keen).count();
        let keen_ids: Vec<String> = player_info
            .iter()
            .filter(|(_, keen)| *keen)
            .map(|(id, _)| id.clone())
            .collect();

        if !keen_ids.is_empty() && non_keen_count >= 11 {
            for id in &keen_ids {
                if let Some(state) = game.field_model.player_state(id) {
                    if state.base() == PS_RESERVE {
                        game.field_model
                            .set_player_state(id, state.change_base(PS_SETUP_PREVENTED));
                    }
                }
            }
        }
    }

    /// Java: `UtilServerGame.updatePlayerStateDependentProperties(IStep)`.
    ///
    /// Three sub-steps (in Java order):
    ///   1. `mechanic.updateLeaderReRollsForTeam` — both teams
    ///   2. `checkForMissingPartners` — both teams
    ///   3. `updateSingleUseReRolls` — both teams
    pub fn update_player_state_dependent_properties(game: &mut Game) {
        let mechanic = state_mechanic_for(game.rules);
        mechanic.update_leader_re_rolls_for_team(game, true);
        mechanic.update_leader_re_rolls_for_team(game, false);
        Self::check_for_missing_partners(game, true);
        Self::check_for_missing_partners(game, false);
        Self::update_single_use_rerolls_for_game(game, true);
        Self::update_single_use_rerolls_for_game(game, false);
    }

    /// Java: `UtilServerGame.checkForMissingPartners(Game, Team, FieldModel, IStep)` (bb2020 Two
    /// for One, §M 2b(g)):
    ///
    /// ```java
    /// players = team players with reducesLonerRollIfPartnerIsHurt
    /// for currentPlayer in players:
    ///   skill = currentPlayer.getSkillWithProperty(reducesLonerRollIfPartnerIsHurt)
    ///   partnerPosId = currentPlayer.getPosition().getTeamWithPositionId()
    ///   if provided: partner = first of players whose position id == partnerPosId
    ///     if present and currentPlayer is casualty or KNOCKED_OUT
    ///        and !partner.hasActiveEnhancement(skill): addPartnerEnhancement(partner, currentPlayer)
    /// ```
    ///
    /// `addPartnerEnhancement` = `fieldModel.addSkillEnhancements(player, skill)` (TwoForOne's
    /// TemporaryEnhancements: Loner with value "2", source = the skill name), `player.markUsed(skill)`,
    /// `addReport(new ReportTwoForOne(player, partner, true))`.
    fn check_for_missing_partners(game: &mut Game, home: bool) {
        let prop = NamedProperties::REDUCES_LONER_ROLL_IF_PARTNER_IS_HURT;
        let rules = game.rules;
        // (id, position id, partner position id, skill) of every player with the property.
        let players: Vec<(String, String, Option<String>, Option<SkillId>)> = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter()
                .filter(|p| p.has_skill_property_in(rules, prop))
                .map(|p| (p.id.clone(), p.position_id.clone(), p.position_team_with_position_id.clone(),
                          p.skill_id_with_property_in(rules, prop)))
                .collect()
        };
        for (current_id, _, partner_pos_id, skill) in &players {
            let Some(partner_pos_id) = partner_pos_id.as_deref().filter(|s| !s.is_empty()) else { continue };
            let Some(skill) = skill else { continue };
            let Some((partner_id, ..)) = players.iter().find(|(_, pos, ..)| pos == partner_pos_id) else { continue };
            let removed = game.field_model.player_state(current_id)
                .map(|s| s.is_casualty() || s.base() == PS_KNOCKED_OUT)
                .unwrap_or(false);
            let skill_name = skill.display_name();
            let partner_has = game.player(partner_id).map(|p| p.has_active_enhancement(skill_name)).unwrap_or(true);
            if removed && !partner_has {
                // Java: addPartnerEnhancement(game, fieldModel, partner, currentPlayer, skill, step)
                if let Some(p) = game.player_mut(partner_id) {
                    // TwoForOne.postConstruct: withSkills({SkillClassWithValue(Loner.class, "2")})
                    if !p.has_active_enhancement(skill_name) {
                        p.add_prayer_skill(skill_name, SkillId::Loner, Some("2".to_string()));
                    }
                }
                game.mark_skill_used(partner_id, *skill);
                game.report_list.add(ReportTwoForOne::new(partner_id.clone(), current_id.clone(), true));
            }
        }
    }

    /// Helper: calls `update_single_use_rerolls` for home (true) or away (false) team.
    fn update_single_use_rerolls_for_game(game: &mut Game, home: bool) {
        // Collect player data before mutating turn_data to avoid borrow conflict.
        let player_data: Vec<(String, bool)> = {
            let team = if home { &game.team_home } else { &game.team_away };
            // Edition-aware: only bb2020 Lord of Chaos registers this property.
            team.players.iter()
                .map(|p| (p.id.clone(), UtilCards::has_unused_skill_with_property_for(
                    p, NamedProperties::GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH, game.rules)))
                .collect()
        };
        let count = player_data.iter()
            .filter(|(_, has_prop)| *has_prop)
            .filter(|(id, _)| game.field_model.player_state(id)
                .map(|s| !s.is_casualty() && s.base() != PS_BANNED)
                .unwrap_or(false))
            .count() as i32;
        if home {
            game.turn_data_home.single_use_rerolls = count;
        } else {
            game.turn_data_away.single_use_rerolls = count;
        }
    }

    /// Java: UtilServerGame.updateSingleUseReRolls(TurnData, Team, FieldModel)
    ///
    /// Counts on-pitch, non-casualty, non-banned players with the
    /// `grantsSingleUseTeamRerollWhenOnPitch` skill property (and an unused
    /// instance of it), then writes the count to `turn_data.single_use_rerolls`.
    ///
    /// NOTE: the property check requires `NamedProperties` not yet in ffb-model.
    /// The stub always yields 0; correct once NamedProperties is added.
    pub fn update_single_use_rerolls(
        turn_data: &mut TurnData,
        team: &Team,
        field_model: &FieldModel,
    ) {
        let count = team
            .players
            .iter()
            .filter(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH))
            .filter(|p| {
                field_model
                    .player_state(&p.id)
                    .map(|s| !s.is_casualty() && s.base() != PS_BANNED)
                    .unwrap_or(false)
            })
            .count() as i32;

        turn_data.single_use_rerolls = count;
    }

    /// Java: StepEndTurn.markPlayedAndSecretWeapons()
    ///
    /// For each player that `can_be_set_up_next_drive()` and whose base state is
    /// not RESERVE (i.e. played this drive):
    /// - If they have `getsSentOffAtEndOfDrive` → mark `has_used_secret_weapon = true`.
    /// - In REGULAR turns only, increment `turns_played` for the active team's players.
    ///
    /// Only runs for `TurnMode::Regular` and `TurnMode::Kickoff`.
    pub fn mark_played_and_secret_weapons(game: &mut Game) {
        if game.turn_mode != TurnMode::Regular && game.turn_mode != TurnMode::Kickoff {
            return;
        }
        let is_regular = game.turn_mode == TurnMode::Regular;
        let home_playing = game.home_playing;
        // Collect updates first to avoid simultaneous borrow of game fields.
        // Collect (player_id, is_home, increment_turns, has_secret_weapon) to avoid borrow conflicts.
        let mut updates: Vec<(String, bool, bool, bool)> = Vec::new();
        for is_home in [true, false] {
            let team = if is_home { &game.team_home } else { &game.team_away };
            for player in &team.players {
                let state = game.field_model.player_state(&player.id);
                let Some(state) = state else { continue };
                if state.can_be_set_up_next_drive() && state.base() != PS_RESERVE {
                    let has_secret_weapon = player.has_skill_property(NamedProperties::GETS_SENT_OFF_AT_END_OF_DRIVE);
                    let increment_turns = is_regular && (is_home == home_playing);
                    updates.push((player.id.clone(), is_home, increment_turns, has_secret_weapon));
                }
            }
        }
        for (player_id, is_home, increment_turns, has_secret_weapon) in updates {
            if has_secret_weapon && std::env::var_os("FFB_TRACE").is_some() {
                eprintln!("RSWMARK pid={player_id}");
            }
            let pr = game.game_result.team_result_mut(is_home).player_result_mut(&player_id);
            if has_secret_weapon {
                pr.has_used_secret_weapon = true;
            }
            if increment_turns {
                pr.turns_played += 1;
            }
        }
    }

    /// Java: UtilServerGame.checkForWastedSkills(Player, IStep, FieldModel).
    ///
    /// If the player's state is casualty or BANNED, check for any
    /// `grantsSingleUseTeamRerollWhenOnPitch` skills that were never used,
    /// and emit a `ReportSkillWasted` for each.
    pub fn check_for_wasted_skills(game: &mut Game, player_id: &str) {
        let state = game.field_model.player_state(player_id);
        let Some(ps) = state else { return };
        if !ps.is_casualty() && ps.base() != PS_BANNED { return; }

        // Java: Constant.CHECK_AFTER_PLAYER_REMOVAL = {grantsSingleUseTeamRerollWhenOnPitch}, read
        // off the EDITION's skill class: bb2025 Lord of Chaos does not register it, so a bb2025
        // Lord Borak leaving the pitch wastes nothing (star tier §M, orc vs chaos_ogre seed 25:
        // Rust reported `skillWasted Lord of Chaos` where Java reports nothing).
        let rules = game.rules;
        let wasted: Vec<_> = game.player(player_id)
            .map(|p| {
                p.all_skill_ids()
                    .filter(|id| id.properties_for(rules).contains(&NamedProperties::GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH))
                    .filter(|id| !p.used_skills.contains(id))
                    .collect()
            })
            .unwrap_or_default();

        let pid = player_id.to_owned();
        for skill_id in wasted {
            game.report_list.add(ReportSkillWasted::new(Some(pid.clone()), Some(skill_id)));
        }
    }
}

impl Default for UtilServerGame {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PlayerType, PlayerGender, PS_BADLY_HURT, PS_RESERVE, PS_SETUP_PREVENTED};
    use ffb_model::enums::Rules;
    use ffb_model::model::field_model::FieldModel;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::model::turn_data::TurnData;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 0,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 3,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn empty_game() -> Game {
        Game::new(
            make_team("home", vec![]),
            make_team("away", vec![]),
            Rules::Bb2025,
        )
    }

    /// Java registers `grantsSingleUseTeamRerollWhenOnPitch` only on bb2020 Lord of Chaos: a
    /// bb2025 Lord Borak leaving the pitch as a casualty wastes nothing (§M, orc vs chaos_ogre
    /// seed 25), while the bb2020 one reports his unused skill.
    #[test]
    fn lord_of_chaos_is_wasted_only_in_bb2020() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{SkillId, PS_BADLY_HURT};
        for (rules, expected) in [(Rules::Bb2020, 1usize), (Rules::Bb2025, 0usize)] {
            let mut p = make_player("borak");
            p.starting_skills.push(SkillWithValue { skill_id: SkillId::LordOfChaos, value: None });
            let mut game = Game::new(make_team("home", vec![p]), make_team("away", vec![]), rules);
            game.field_model.set_player_state("borak", PlayerState::new(PS_BADLY_HURT));
            UtilServerGame::check_for_wasted_skills(&mut game, "borak");
            assert_eq!(game.report_list.size(), expected, "{rules:?}");
        }
    }

    /// §M 2b(g) Java `checkForMissingPartners`: when one of a Two for One pair is KO'd (or a
    /// casualty) the partner gains the skill's enhancement (Loner "2", source "Two for One") ONCE,
    /// has Two for One marked used, and ReportTwoForOne(partner, hurt player) is added; a pair
    /// without partner ids gets nothing. (bb2025 is not a case: Java has only a bb2020 class and
    /// the bb2025 star catalogue carries no Two for One.)
    #[test]
    fn two_for_one_partner_gains_loner_two_once() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{SkillId, PS_KNOCKED_OUT, PS_STANDING};
        let pair = |link: bool| {
            let mut grak = make_player("grak");
            grak.position_id = "star.grak".into();
            grak.starting_skills = vec![SkillWithValue::with_value(SkillId::Loner, "4"),
                                        SkillWithValue::new(SkillId::TwoForOne)];
            let mut crum = make_player("crum");
            crum.position_id = "star.crumbleberry".into();
            crum.starting_skills = grak.starting_skills.clone();
            if link {
                grak.position_team_with_position_id = Some("star.crumbleberry".into());
                crum.position_team_with_position_id = Some("star.grak".into());
            }
            (grak, crum)
        };
        for (rules, link, expected) in [(Rules::Bb2020, true, 1usize), (Rules::Bb2020, false, 0)] {
            let (grak, crum) = pair(link);
            let mut game = Game::new(make_team("home", vec![grak, crum]), make_team("away", vec![]), rules);
            game.field_model.set_player_state("grak", PlayerState::new(PS_STANDING));
            game.field_model.set_player_state("crum", PlayerState::new(PS_KNOCKED_OUT));
            UtilServerGame::update_player_state_dependent_properties(&mut game);
            UtilServerGame::update_player_state_dependent_properties(&mut game);
            assert_eq!(game.report_list.size(), expected, "{rules:?} link={link}");
            let g = game.player("grak").unwrap();
            let rr = g.get_skill_int_value(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL);
            assert_eq!(rr, if expected == 1 { 2 } else { 4 }, "{rules:?} link={link}");
            assert_eq!(g.used_skills.contains(&SkillId::TwoForOne), expected == 1);
        }
    }

    // ── update_single_use_rerolls ───────────────────────────────────────────

    #[test]
    fn update_single_use_rerolls_zero_when_no_players() {
        let team = make_team("t1", vec![]);
        let fm = FieldModel::new();
        let mut td = TurnData::new();
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        assert_eq!(td.single_use_rerolls, 0);
    }

    #[test]
    fn update_single_use_rerolls_overwrites_previous_value() {
        let team = make_team("t1", vec![]);
        let fm = FieldModel::new();
        let mut td = TurnData::new();
        td.single_use_rerolls = 5;
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        assert_eq!(td.single_use_rerolls, 0);
    }

    #[test]
    fn update_single_use_rerolls_player_without_property_not_counted() {
        let p = make_player("p1");
        let team = make_team("t1", vec![p]);
        let mut fm = FieldModel::new();
        fm.set_player_state("p1", PlayerState(PS_RESERVE));
        let mut td = TurnData::new();
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        assert_eq!(td.single_use_rerolls, 0);
    }

    #[test]
    fn update_single_use_rerolls_casualty_not_counted() {
        let p = make_player("p1");
        let team = make_team("t1", vec![p]);
        let mut fm = FieldModel::new();
        fm.set_player_state("p1", PlayerState(PS_BADLY_HURT));
        let mut td = TurnData::new();
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        assert_eq!(td.single_use_rerolls, 0);
    }

    // ── prepare_for_setup ──────────────────────────────────────────────────

    #[test]
    fn prepare_for_setup_empty_teams_no_panic() {
        let mut game = empty_game();
        UtilServerGame::prepare_for_setup(&mut game);
    }

    #[test]
    fn prepare_for_setup_fewer_than_eleven_non_keen_no_setup_prevented() {
        // 10 non-keen RESERVE players. No keen players. Nothing changes.
        let players: Vec<Player> = (0..10).map(|i| make_player(&format!("p{}", i))).collect();
        let home = make_team("home", players.clone());
        let away = make_team("away", vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        for p in &players {
            game.field_model.set_player_state(&p.id, PlayerState(PS_RESERVE));
        }
        UtilServerGame::prepare_for_setup(&mut game);
        for p in &players {
            assert_eq!(
                game.field_model.player_state(&p.id).unwrap().base(),
                PS_RESERVE,
                "player {} should still be RESERVE",
                p.id
            );
        }
    }

    #[test]
    fn prepare_for_setup_eleven_non_keen_leaves_non_keen_unchanged() {
        // 11 non-keen RESERVE players. No player has the keen property.
        let players: Vec<Player> = (0..11).map(|i| make_player(&format!("p{}", i))).collect();
        let home = make_team("home", players.clone());
        let away = make_team("away", vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        for p in &players {
            game.field_model.set_player_state(&p.id, PlayerState(PS_RESERVE));
        }
        UtilServerGame::prepare_for_setup(&mut game);
        for p in &players {
            assert_ne!(
                game.field_model.player_state(&p.id).unwrap().base(),
                PS_SETUP_PREVENTED,
                "non-keen player {} must not be SETUP_PREVENTED",
                p.id
            );
        }
    }

    #[test]
    fn update_single_use_rerolls_idempotent_when_called_twice() {
        let team = make_team("t1", vec![]);
        let fm = FieldModel::new();
        let mut td = TurnData::new();
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        let count_first = td.single_use_rerolls;
        UtilServerGame::update_single_use_rerolls(&mut td, &team, &fm);
        assert_eq!(td.single_use_rerolls, count_first);
    }

    // ── mark_played_and_secret_weapons ─────────────────────────────────────

    fn game_with_standing_player(id: &str, is_home: bool) -> Game {
        use ffb_model::enums::PS_STANDING;
        let p = make_player(id);
        let (home_players, away_players) = if is_home {
            (vec![p], vec![])
        } else {
            (vec![], vec![make_player(id)])
        };
        let mut game = Game::new(
            make_team("home", home_players),
            make_team("away", away_players),
            Rules::Bb2025,
        );
        game.field_model.set_player_state(id, PlayerState(PS_STANDING));
        game
    }

    #[test]
    fn mark_played_noop_for_non_regular_kickoff_turnmode() {
        let mut game = game_with_standing_player("p1", true);
        game.turn_mode = TurnMode::Blitz;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert_eq!(game.game_result.home.player_result("p1").map(|p| p.turns_played), None);
    }

    #[test]
    fn mark_played_increments_turns_played_for_active_team_in_regular_mode() {
        use ffb_model::enums::PS_STANDING;
        let mut game = game_with_standing_player("p1", true);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert_eq!(game.game_result.home.player_result("p1").unwrap().turns_played, 1);
    }

    #[test]
    fn mark_played_does_not_increment_turns_played_for_inactive_team_in_regular_mode() {
        use ffb_model::enums::PS_STANDING;
        let mut game = game_with_standing_player("p1", false); // away player
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true; // home is active
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert_eq!(game.game_result.away.player_result("p1").map(|p| p.turns_played).unwrap_or(0), 0);
    }

    #[test]
    fn mark_played_does_not_increment_turns_played_in_kickoff_mode() {
        let mut game = game_with_standing_player("p1", true);
        game.turn_mode = TurnMode::Kickoff;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert_eq!(game.game_result.home.player_result("p1").map(|p| p.turns_played).unwrap_or(0), 0);
    }

    #[test]
    fn mark_played_skips_reserve_player() {
        let mut game = game_with_standing_player("p1", true);
        // Override state to RESERVE
        game.field_model.set_player_state("p1", PlayerState(PS_RESERVE));
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert_eq!(game.game_result.home.player_result("p1").map(|p| p.turns_played).unwrap_or(0), 0);
    }

    #[test]
    fn mark_played_sets_has_used_secret_weapon_for_player_with_secret_weapon_skill() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut p = make_player("sw");
        p.starting_skills.push(SkillWithValue::new(SkillId::SecretWeapon));
        let home = make_team("home", vec![p]);
        let away = make_team("away", vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        use ffb_model::enums::PS_STANDING;
        game.field_model.set_player_state("sw", PlayerState(PS_STANDING));
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert!(game.game_result.home.player_result("sw").unwrap().has_used_secret_weapon);
    }

    #[test]
    fn mark_played_does_not_set_secret_weapon_for_regular_player() {
        let mut game = game_with_standing_player("p1", true);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        UtilServerGame::mark_played_and_secret_weapons(&mut game);
        assert!(!game.game_result.home.player_result("p1").map(|p| p.has_used_secret_weapon).unwrap_or(false));
    }

    #[test]
    fn check_for_wasted_skills_no_report_when_not_casualty() {
        let game = &mut game_with_standing_player("p1", true);
        use ffb_model::enums::PS_STANDING;
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        UtilServerGame::check_for_wasted_skills(game, "p1");
        assert!(game.report_list.is_empty());
    }

    #[test]
    fn check_for_wasted_skills_no_report_when_no_single_use_skill() {
        let game = &mut game_with_standing_player("p1", true);
        use ffb_model::enums::PS_BADLY_HURT;
        game.field_model.set_player_state("p1", PlayerState(PS_BADLY_HURT));
        UtilServerGame::check_for_wasted_skills(game, "p1");
        assert!(game.report_list.is_empty());
    }
}
