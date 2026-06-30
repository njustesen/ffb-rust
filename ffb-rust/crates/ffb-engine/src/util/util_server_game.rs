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
//
// Property-dependent filtering (canJoinTeamIfLessThanEleven,
// grantsSingleUseTeamRerollWhenOnPitch) requires the NamedProperties
// infrastructure that is not yet present in this version.  These are
// implemented with a TODO stub that always returns false for the property
// check, which is correct for teams without Chameleon Skinks / similar.
//
// Skipped (touch IStep / server / WebSocket):
//   syncGameModel, changeActingPlayer, handleChefRolls, rollMasterChef,
//   checkForWastedSkills, updatePlayerStateDependentProperties,
//   checkForMissingPartners, addPartnerEnhancement, closeGame, handleInvalidTeam

use ffb_model::enums::{PlayerState, PS_BANNED, PS_RESERVE, PS_SETUP_PREVENTED};
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::model::turn_data::TurnData;

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
    /// SETUP_PREVENTED.
    ///
    /// NOTE: the "keen" property check requires `NamedProperties` which is not
    /// yet in ffb-model.  Until it is added, `has_keen_property` always returns
    /// false, so the branch that moves players to SETUP_PREVENTED is a no-op.
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
                    // TODO: replace `false` with
                    //   p.has_skill_property(NamedProperties::CAN_JOIN_TEAM_IF_LESS_THAN_ELEVEN)
                    //   once NamedProperties is available.
                    let keen: bool = false;
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
            .filter(|p| {
                // TODO: replace with
                //   p.has_skill_property(NamedProperties::GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH)
                let _ = p; // suppress unused warning
                false
            })
            .filter(|p| {
                field_model
                    .player_state(&p.id)
                    .map(|s| !s.is_casualty() && s.base() != PS_BANNED)
                    .unwrap_or(false)
            })
            .count() as i32;

        turn_data.single_use_rerolls = count;
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
        }
    }

    fn empty_game() -> Game {
        Game::new(
            make_team("home", vec![]),
            make_team("away", vec![]),
            Rules::Bb2025,
        )
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
}
