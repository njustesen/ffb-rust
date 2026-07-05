/// 1:1 translation of com.fumbbl.ffb.server.mechanic.bb2025.StateMechanic.
///
/// @RulesCollection(RulesCollection.Rules.BB2025)
///
/// Differences from mixed::StateMechanic:
///   - start_half: resets inducements (conditional re-rolls) at half <= 2
///   - handle_pump_up: grants re-roll for block-type casualties only
///     (grantsTeamReRollWhenCausingBlockCas vs grantsTeamReRollWhenCausingCas in mixed)
///
/// Report emission status:
///   - ReportStartHalf: wired in StepInitKickoff (emits GameEvent::StartHalf after start_half call)
///   - ReportLeader: caller responsibility — emitted when update_leader_re_rolls_for_team returns Some
///   - ReportPumpUpTheCrowdReRoll: GameEvent::PumpUpTheCrowdReRoll — caller emits when handle_pump_up returns true
use ffb_model::enums::{LeaderState, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::turn_data::TurnData;
use ffb_model::model::property::named_properties::NamedProperties;
use crate::injury::InjuryContext;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;

pub struct StateMechanic;

impl StateMechanic {
    pub fn new() -> Self { Self }
}

impl Default for StateMechanic {
    fn default() -> Self { Self::new() }
}

impl StateMechanicTrait for StateMechanic {
    /// Java: updateLeaderReRollsForTeam.
    /// BB2025 uses gameState.hasLeader(team); we approximate with team_has_leader_on_field.
    /// Returns Some(new_state) when state transitions occur; caller adds ReportLeader.
    fn update_leader_re_rolls_for_team(
        &self,
        game: &mut Game,
        home_team: bool,
    ) -> Option<LeaderState> {
        let team = if home_team { game.team_home.clone() } else { game.team_away.clone() };
        let has_leader = self.team_has_leader_on_field(&team, &game.field_model);
        let turn_data: &mut TurnData = if home_team {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };

        if turn_data.leader_state == LeaderState::Used {
            return None;
        }
        if has_leader {
            if turn_data.leader_state == LeaderState::None {
                turn_data.leader_state = LeaderState::Available;
                turn_data.rerolls += 1;
                // NOTE: caller emits GameEvent::Leader { player_id: team_id, reroll_available: true }
                return Some(LeaderState::Available);
            }
        } else if turn_data.leader_state == LeaderState::Available {
            turn_data.leader_state = LeaderState::None;
            turn_data.rerolls = (turn_data.rerolls - 1).max(0);
            // NOTE: caller emits GameEvent::Leader { player_id: team_id, reroll_available: false }
            return Some(LeaderState::None);
        }
        None
    }

    /// Java: startHalf(IStep, int pHalf).
    fn start_half(&self, game: &mut Game, half: i32) -> Vec<GameEvent> {
        let mut events: Vec<GameEvent> = Vec::new();
        game.half = half;
        game.turn_data_home.turn_nr = 0;
        game.turn_data_away.turn_nr = 0;
        if game.home_first_offense {
            game.home_playing = game.half % 2 == 0;
        } else {
            game.home_playing = game.half % 2 != 0;
        }
        game.field_model.ball_coordinate = None;
        game.field_model.ball_in_play = false;
        // NOTE: ReportStartHalf emitted by the calling step (StepInitKickoff) after start_half returns.

        if half <= 1 {
            events.extend(self.add_apothecaries(game, true));
            events.extend(self.add_apothecaries(game, false));
        }
        if half <= 2 {
            events.extend(self.add_re_rolls(game, true));
            events.extend(self.add_re_rolls(game, false));
        }

        self.reset_leader_state(game);
        self.reset_special_skills_at_half_time(game);
        self.reset_inducements(game);
        events
    }

    /// Java: handlePumpUp(IStep, InjuryResult).
    /// BB2025: grants re-roll only for block-type casualties
    /// (NamedProperties.grantsTeamReRollWhenCausingBlockCas).
    fn handle_pump_up(&self, game: &mut Game, injury_context: &InjuryContext) -> bool {
        let attacker_id = injury_context.attacker_id.clone();
        let attacker_id = match attacker_id.as_deref() {
            Some(id) => id.to_string(),
            None => return false,
        };

        let on_acting_team = game.is_active_team_player(&attacker_id);
        let is_casualty = injury_context.is_casualty();

        if !on_acting_team || !is_casualty {
            return false;
        }

        let attacker_prone_or_stunned = game
            .field_model
            .player_state(&attacker_id)
            .map(|s| s.is_prone_or_stunned())
            .unwrap_or(false);

        if attacker_prone_or_stunned {
            return false;
        }

        let has_skill = game
            .player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::GRANTS_TEAM_RE_ROLL_WHEN_CAUSING_BLOCK_CAS))
            .unwrap_or(false);

        if !has_skill {
            return false;
        }

        let is_block_injury = injury_context.injury_type_name.as_deref() == Some("Block");
        if !is_block_injury {
            return false;
        }

        if game.home_playing {
            game.turn_data_home.rerolls += 1;
            game.turn_data_home.rerolls_pump_up_the_crowd_one_drive += 1;
        } else {
            game.turn_data_away.rerolls += 1;
            game.turn_data_away.rerolls_pump_up_the_crowd_one_drive += 1;
        }

        game.mark_skill_used(&attacker_id, SkillId::PumpUpTheCrowd);
        // NOTE: caller emits GameEvent::PumpUpTheCrowdReRoll { player_id: attacker_id } when this returns true.
        // SoundId::PUMP_CROWD is client-side only — not modelled in the engine event stream.
        true
    }
}

impl StateMechanic {
    fn reset_leader_state(&self, game: &mut Game) {
        if game.half <= 2 {
            // Java: gameState.resetLeaders() → activeEffects.clearLeaders()
            // Rust: we don't use the leader registry — check is done on-the-fly via team_has_leader_on_field.
            // Resetting leader_state is the equivalent of clearing the reroll grant.
            game.turn_data_home.leader_state = LeaderState::None;
            game.turn_data_away.leader_state = LeaderState::None;
        }
    }

    fn reset_special_skills_at_half_time(&self, game: &mut Game) {
        if game.half <= 2 {
            use ffb_model::enums::SkillUsageType;
            for p in game.team_home.players.iter_mut().chain(game.team_away.players.iter_mut()) {
                p.reset_used_skills(SkillUsageType::OncePerHalf);
            }
        }
        self.reset_special_skill_at_end_of_drive(game);
    }

    fn reset_inducements(&self, game: &mut Game) {
        if game.half <= 2 {
            use ffb_model::inducement::usage::Usage;
            Self::reset_conditional_reroll(&mut game.turn_data_home.inducement_set);
            Self::reset_conditional_reroll(&mut game.turn_data_away.inducement_set);
        }
    }

    fn reset_conditional_reroll(inducement_set: &mut ffb_model::model::inducement_set::InducementSet) {
        use ffb_model::inducement::usage::Usage;
        if let Some(type_id) = inducement_set.for_usage(Usage::CONDITIONAL_REROLL).map(|s| s.to_string()) {
            if let Some(mut ind) = inducement_set.get(&type_id) {
                ind.set_uses(0);
                inducement_set.add_inducement(ind);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{LeaderState, Rules};
    use ffb_model::model::game::Game;

    fn make_game() -> Game {
        Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        )
    }

    #[test]
    fn start_half_sets_half_counter() {
        let m = StateMechanic::new();
        let mut g = make_game();
        m.start_half(&mut g, 2);
        assert_eq!(g.half, 2);
    }

    #[test]
    fn start_half_resets_turn_numbers() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.turn_nr = 4;
        g.turn_data_away.turn_nr = 3;
        m.start_half(&mut g, 1);
        assert_eq!(g.turn_data_home.turn_nr, 0);
        assert_eq!(g.turn_data_away.turn_nr, 0);
    }

    #[test]
    fn start_half_clears_ball() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.field_model.ball_in_play = true;
        m.start_half(&mut g, 1);
        assert!(!g.field_model.ball_in_play);
        assert!(g.field_model.ball_coordinate.is_none());
    }

    #[test]
    fn start_half_home_playing_home_first_offense() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_first_offense = true;
        // half 1 (odd): home_playing = (1 % 2 == 0) = false
        m.start_half(&mut g, 1);
        assert!(!g.home_playing);
        // half 2 (even): home_playing = (2 % 2 == 0) = true
        m.start_half(&mut g, 2);
        assert!(g.home_playing);
    }

    #[test]
    fn start_half_home_playing_away_first_offense() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_first_offense = false;
        // half 1 (odd): home_playing = (1 % 2 != 0) = true
        m.start_half(&mut g, 1);
        assert!(g.home_playing);
        // half 2 (even): home_playing = false
        m.start_half(&mut g, 2);
        assert!(!g.home_playing);
    }

    #[test]
    fn start_half_resets_leader_state_at_or_before_half_2() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        g.turn_data_away.leader_state = LeaderState::Used;
        m.start_half(&mut g, 2);
        assert_eq!(g.turn_data_home.leader_state, LeaderState::None);
        assert_eq!(g.turn_data_away.leader_state, LeaderState::None);
    }

    #[test]
    fn start_half_does_not_reset_leader_at_half_3() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        m.start_half(&mut g, 3);
        assert_eq!(g.turn_data_home.leader_state, LeaderState::Available);
    }

    #[test]
    fn update_leader_none_no_leader_on_field_no_change() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.rerolls = 2;
        let result = m.update_leader_re_rolls_for_team(&mut g, true);
        assert!(result.is_none());
        assert_eq!(g.turn_data_home.rerolls, 2);
    }

    #[test]
    fn update_leader_available_to_none_no_leader_on_field() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        g.turn_data_home.rerolls = 3;
        let result = m.update_leader_re_rolls_for_team(&mut g, true);
        assert_eq!(result, Some(LeaderState::None));
        assert_eq!(g.turn_data_home.leader_state, LeaderState::None);
        assert_eq!(g.turn_data_home.rerolls, 2);
    }

    #[test]
    fn update_leader_used_state_no_change() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Used;
        g.turn_data_home.rerolls = 1;
        let result = m.update_leader_re_rolls_for_team(&mut g, true);
        assert!(result.is_none());
        assert_eq!(g.turn_data_home.rerolls, 1);
    }

    #[test]
    fn update_leader_rerolls_not_negative() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        g.turn_data_home.rerolls = 0;
        m.update_leader_re_rolls_for_team(&mut g, true);
        assert_eq!(g.turn_data_home.rerolls, 0); // max(0-1, 0) = 0
    }

    fn make_pump_up_player(game: &mut Game, home: bool) {
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING};
        use std::collections::HashSet;
        let mut p = Player {
            id: "att".into(), name: "att".into(), nr: 1,
            position_id: "pos".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        p.extra_skills.push(SkillWithValue { skill_id: SkillId::PumpUpTheCrowd, value: None });
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        let coord = ffb_model::types::FieldCoordinate::new(5, 5);
        let state = ffb_model::enums::PlayerState::new(PS_STANDING);
        game.field_model.set_player_coordinate("att", coord);
        game.field_model.set_player_state("att", state);
    }

    fn make_block_injury_context(is_cas: bool, injury_type_name: Option<&str>) -> InjuryContext {
        use ffb_model::enums::{ApothecaryMode, PS_RIP, PS_STUNNED, PlayerState};
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.attacker_id = Some("att".into());
        ctx.injury = Some(PlayerState::new(if is_cas { PS_RIP } else { PS_STUNNED }));
        ctx.injury_type_name = injury_type_name.map(|s| s.to_string());
        ctx
    }

    #[test]
    fn handle_pump_up_no_attacker_returns_false() {
        use ffb_model::enums::ApothecaryMode;
        let m = StateMechanic::new();
        let mut g = make_game();
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(!m.handle_pump_up(&mut g, &ctx));
    }

    #[test]
    fn handle_pump_up_non_block_injury_type_returns_false() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_playing = true;
        make_pump_up_player(&mut g, true);
        let ctx = make_block_injury_context(true, Some("Foul"));
        assert!(!m.handle_pump_up(&mut g, &ctx));
    }

    #[test]
    fn handle_pump_up_no_injury_type_returns_false() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_playing = true;
        make_pump_up_player(&mut g, true);
        let ctx = make_block_injury_context(true, None);
        assert!(!m.handle_pump_up(&mut g, &ctx));
    }

    #[test]
    fn handle_pump_up_block_casualty_grants_reroll() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_playing = true;
        make_pump_up_player(&mut g, true);
        let ctx = make_block_injury_context(true, Some("Block"));
        let result = m.handle_pump_up(&mut g, &ctx);
        assert!(result);
        assert_eq!(g.turn_data_home.rerolls, 1);
        assert_eq!(g.turn_data_home.rerolls_pump_up_the_crowd_one_drive, 1);
    }

    #[test]
    fn start_half_re_rolls_set_from_team_first_two_halves() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.rerolls = 3;
        g.team_away.rerolls = 2;
        m.start_half(&mut g, 1);
        assert_eq!(g.turn_data_home.rerolls, 3);
        assert_eq!(g.turn_data_away.rerolls, 2);
    }

    #[test]
    fn start_half_apothecaries_only_at_half_1() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.apothecaries = 1;
        g.turn_data_home.apothecaries = 0;
        // half 2 > 1 → apothecaries not set
        m.start_half(&mut g, 2);
        assert_eq!(g.turn_data_home.apothecaries, 0);
    }

    #[test]
    fn start_half_apothecaries_set_at_half_1() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.apothecaries = 2;
        m.start_half(&mut g, 1);
        assert_eq!(g.turn_data_home.apothecaries, 2);
    }

    #[test]
    fn reset_special_skills_clears_once_per_half_skill_for_all_players() {
        use ffb_model::enums::SkillUsageType;
        let m = StateMechanic::new();
        let mut g = make_game();
        use ffb_model::model::player::Player;
        let mut p1 = Player { id: "p1".into(), ..Default::default() };
        p1.used_skills.insert(ffb_model::model::skill_def::SkillId::Leader); // OncePerHalf
        let mut p2 = Player { id: "p2".into(), ..Default::default() };
        p2.used_skills.insert(ffb_model::model::skill_def::SkillId::BeerBarrelBash); // OncePerDrive
        g.team_home.players.push(p1);
        g.team_away.players.push(p2);
        g.half = 1;
        m.reset_special_skills_at_half_time(&mut g);
        assert!(!g.team_home.players[0].used_skills.contains(&ffb_model::model::skill_def::SkillId::Leader));
        // OncePerDrive also cleared by reset_special_skill_at_end_of_drive
        assert!(!g.team_away.players[0].used_skills.contains(&ffb_model::model::skill_def::SkillId::BeerBarrelBash));
    }

    #[test]
    fn reset_special_skills_at_half_3_skips_once_per_half_but_clears_end_of_drive() {
        let m = StateMechanic::new();
        let mut g = make_game();
        use ffb_model::model::player::Player;
        let mut p1 = Player { id: "p1".into(), ..Default::default() };
        p1.used_skills.insert(ffb_model::model::skill_def::SkillId::Leader); // OncePerHalf
        p1.used_skills.insert(ffb_model::model::skill_def::SkillId::BeerBarrelBash); // OncePerDrive
        g.team_home.players.push(p1);
        g.half = 3;
        m.reset_special_skills_at_half_time(&mut g);
        // half > 2: OncePerHalf not cleared
        assert!(g.team_home.players[0].used_skills.contains(&ffb_model::model::skill_def::SkillId::Leader));
        // OncePerDrive always cleared
        assert!(!g.team_home.players[0].used_skills.contains(&ffb_model::model::skill_def::SkillId::BeerBarrelBash));
    }

    #[test]
    fn reset_inducements_resets_conditional_reroll_uses() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        let m = StateMechanic::new();
        let mut g = make_game();
        g.half = 1;
        let mut ind = Inducement::new("conditionalReroll", 2, vec![Usage::CONDITIONAL_REROLL]);
        ind.set_uses(2);
        g.turn_data_home.inducement_set.add_inducement(ind);
        m.reset_inducements(&mut g);
        let retrieved = g.turn_data_home.inducement_set.get("conditionalReroll").unwrap();
        assert_eq!(retrieved.get_uses(), 0);
    }

    #[test]
    fn reset_inducements_no_op_when_no_conditional_reroll() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.half = 1;
        // no conditional reroll inducement — must not panic
        m.reset_inducements(&mut g);
    }

    #[test]
    fn reset_inducements_skips_at_half_3() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        let m = StateMechanic::new();
        let mut g = make_game();
        g.half = 3;
        let mut ind = Inducement::new("conditionalReroll", 2, vec![Usage::CONDITIONAL_REROLL]);
        ind.set_uses(2);
        g.turn_data_home.inducement_set.add_inducement(ind);
        m.reset_inducements(&mut g);
        // half > 2: uses NOT reset
        let retrieved = g.turn_data_home.inducement_set.get("conditionalReroll").unwrap();
        assert_eq!(retrieved.get_uses(), 2);
    }

    #[test]
    fn reset_inducements_resets_away_team_too() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        let m = StateMechanic::new();
        let mut g = make_game();
        g.half = 2;
        let mut ind = Inducement::new("conditionalReroll", 1, vec![Usage::CONDITIONAL_REROLL]);
        ind.set_uses(1);
        g.turn_data_away.inducement_set.add_inducement(ind);
        m.reset_inducements(&mut g);
        let retrieved = g.turn_data_away.inducement_set.get("conditionalReroll").unwrap();
        assert_eq!(retrieved.get_uses(), 0);
    }

    #[test]
    fn start_half_returns_inducement_events_for_wandering_apo_at_half_1() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        use ffb_model::events::GameEvent;
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        let events = m.start_half(&mut g, 1);
        assert!(events.iter().any(|e| matches!(e,
            GameEvent::Inducement { inducement_type, .. } if inducement_type == "wanderingApothecary"
        )));
    }

    #[test]
    fn start_half_no_inducements_returns_empty_events() {
        use ffb_model::events::GameEvent;
        let m = StateMechanic::new();
        let mut g = make_game();
        let events = m.start_half(&mut g, 1);
        assert!(events.is_empty(), "no inducements → no events");
    }

    #[test]
    fn start_half_half_2_skips_apothecary_events() {
        use ffb_model::inducement::inducement::Inducement;
        use ffb_model::inducement::usage::Usage;
        use ffb_model::events::GameEvent;
        let m = StateMechanic::new();
        let mut g = make_game();
        // Add wandering apo — should NOT be registered at half 2 (BB2025 condition: half <= 1)
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("wanderingApothecary", 1, vec![Usage::APOTHECARY])
        );
        let events = m.start_half(&mut g, 2);
        assert!(!events.iter().any(|e| matches!(e, GameEvent::Inducement { .. })),
            "apothecaries not registered at half 2");
    }
}
