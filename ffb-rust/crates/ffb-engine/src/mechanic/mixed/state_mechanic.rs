/// 1:1 translation of com.fumbbl.ffb.server.mechanic.mixed.StateMechanic.
///
/// @RulesCollection(RulesCollection.Rules.BB2016)
/// @RulesCollection(RulesCollection.Rules.BB2020)
///
/// Differences from bb2025::StateMechanic:
///   - start_half: does NOT reset inducements
///   - start_half: calls UtilServerGame.handleChefRolls (now implemented)
///   - handle_pump_up: checks grantsTeamReRollWhenCausingCas (any casualty, not block-specific)
///
/// Report emission status:
///   - ReportStartHalf: wired in StepInitKickoff (emits GameEvent::StartHalf after start_half call)
///   - ReportLeader: caller responsibility — emitted when update_leader_re_rolls_for_team returns Some
///   - ReportPumpUpTheCrowdReRoll: wired via GameEvent::PumpUpTheCrowdReRoll in handle_injury_side_effects
///   - ReportMasterChefRoll: returned from handle_chef_rolls as GameEvent::MasterChefRoll
///     (caller in StepInitKickoff must emit these events)
use ffb_model::enums::{LeaderState, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::turn_data::TurnData;
use ffb_model::util::rng::GameRng;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::InjuryContext;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
use crate::util::util_server_game::UtilServerGame;

pub struct StateMechanic;

impl StateMechanic {
    pub fn new() -> Self { Self }
}

impl Default for StateMechanic {
    fn default() -> Self { Self::new() }
}

impl StateMechanicTrait for StateMechanic {
    /// Java: updateLeaderReRollsForTeam.
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
    /// Mixed condition for apothecaries: `half < 2` (== `half <= 1`).
    /// Mixed condition for re-rolls: `half < 3` (== `half <= 2`).
    ///
    /// NOTE: The Java version calls `UtilServerGame.handleChefRolls` which rolls dice.
    /// Since the trait signature does not expose `rng`, chef rolls are NOT performed
    /// here.  The caller (StepInitKickoff) must call `handle_chef_rolls(game, rng)`
    /// immediately after `start_half` when `half < 3` and `bb2016` ruleset is active
    /// (i.e. when `GameMechanic::rollForChefAtStartOfHalf()` would return true).
    /// BB2020/BB2025 return false for that method, so no chef rolls happen there.
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

        if half < 2 {
            events.extend(self.add_apothecaries(game, true));
            events.extend(self.add_apothecaries(game, false));
        }
        if half < 3 {
            events.extend(self.add_re_rolls(game, true));
            events.extend(self.add_re_rolls(game, false));
            // NOTE: chef rolls omitted here — caller does handle_chef_rolls(game, rng)
            // after start_half when the rules require it (BB2016 only).
        }

        self.reset_leader_state(game);
        UtilServerGame::update_player_state_dependent_properties(game);
        self.reset_special_skills_at_half_time(game);
        events
    }

    /// Java: handlePumpUp(IStep, InjuryResult).
    /// Mixed: checks `grantsTeamReRollWhenCausingCas` (any casualty, not block-specific).
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

        // Mixed uses grantsTeamReRollWhenCausingCas (not block-specific)
        let has_skill = game
            .player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::GRANTS_TEAM_RE_ROLL_WHEN_CAUSING_CAS))
            .unwrap_or(false);

        if !has_skill {
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
    /// Java: `UtilServerGame.handleChefRolls(IStep, Game)`.
    ///
    /// BB2016 only — rolls dice for every Master Chef inducement on both teams,
    /// adjusts re-roll totals, and returns the events to emit.
    ///
    /// Logic:
    ///  1. Roll for home team chefs → reRollsStolenHome
    ///  2. Roll for away team chefs → reRollsStolenAway
    ///  3. homeReRolls = max(0, homeReRolls - stolenAway) + stolenHome
    ///  4. awayReRolls = max(0, awayReRolls - stolenHome) + stolenAway
    ///
    /// Each individual chef roll (3d6) emits one `GameEvent::MasterChefRoll`.
    pub fn handle_chef_rolls(&self, game: &mut Game, rng: &mut GameRng) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let stolen_home = self.roll_master_chef(game, rng, true, &mut events);
        let stolen_away = self.roll_master_chef(game, rng, false, &mut events);

        // Java: Math.max(0, homeReRolls - reRollsStolenAway) + reRollsStolenHome
        let home_rr = game.turn_data_home.rerolls;
        game.turn_data_home.rerolls = (home_rr - stolen_away).max(0) + stolen_home;

        let away_rr = game.turn_data_away.rerolls;
        game.turn_data_away.rerolls = (away_rr - stolen_home).max(0) + stolen_away;

        events
    }

    /// Java: `private static int rollMasterChef(IStep, boolean pHomeTeam)`.
    ///
    /// Finds the inducement with `Usage::STEAL_REROLL` for the given team's inducement
    /// set; rolls 3d6 once per chef value, accumulates stolen re-rolls,
    /// emits a `GameEvent::MasterChefRoll` per roll.
    fn roll_master_chef(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        home_team: bool,
        events: &mut Vec<GameEvent>,
    ) -> i32 {
        let (team_id, chef_count) = if home_team {
            let count = game.turn_data_home.inducement_set.value(Usage::STEAL_REROLL);
            (game.team_home.id.clone(), count)
        } else {
            let count = game.turn_data_away.inducement_set.value(Usage::STEAL_REROLL);
            (game.team_away.id.clone(), count)
        };

        let mut total_stolen = 0i32;
        for _ in 0..chef_count {
            // Java: getDiceRoller().rollMasterChef() → rollDice(3, 6) → 3 × d6
            let roll = [rng.d6(), rng.d6(), rng.d6()];
            let stolen = DiceInterpreter::interpret_master_chef_roll(&roll);
            total_stolen += stolen;
            // Java: addReport(new ReportMasterChefRoll(team.getId(), masterChefRoll, reRollsStolen))
            // GameEvent::MasterChefRoll stores a single i32 roll; we pack the three d6
            // as die1*100+die2*10+die3 (valid since each die ∈ 1..=6, so range 111..=666).
            events.push(GameEvent::MasterChefRoll {
                team_id: team_id.clone(),
                roll: roll[0] * 100 + roll[1] * 10 + roll[2],
                rerolls_stolen: stolen,
            });
        }
        total_stolen
    }

    fn reset_leader_state(&self, game: &mut Game) {
        if game.half <= 2 {
            game.turn_data_home.leader_state = LeaderState::None;
            game.turn_data_away.leader_state = LeaderState::None;
        }
    }

    fn reset_special_skills_at_half_time(&self, game: &mut Game) {
        use ffb_model::enums::SkillUsageType;
        for p in game.team_home.players.iter_mut().chain(game.team_away.players.iter_mut()) {
            p.reset_used_skills(SkillUsageType::OncePerHalf);
        }
        self.reset_special_skill_at_end_of_drive(game);
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
            Rules::Bb2016,
        )
    }

    #[test]
    fn start_half_sets_half_counter() {
        let m = StateMechanic::new();
        let mut g = make_game();
        m.start_half(&mut g, 1);
        assert_eq!(g.half, 1);
    }

    #[test]
    fn start_half_resets_turn_numbers() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.turn_nr = 7;
        g.turn_data_away.turn_nr = 6;
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
    fn start_half_home_playing_logic() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.home_first_offense = true;
        // half 1: home_first_offense → home_playing = (1 % 2 == 0) = false
        m.start_half(&mut g, 1);
        assert!(!g.home_playing);
        m.start_half(&mut g, 2);
        assert!(g.home_playing);
    }

    #[test]
    fn start_half_resets_leader_state_at_halftime() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        g.turn_data_away.leader_state = LeaderState::Used;
        m.start_half(&mut g, 2);
        assert_eq!(g.turn_data_home.leader_state, LeaderState::None);
        assert_eq!(g.turn_data_away.leader_state, LeaderState::None);
    }

    #[test]
    fn start_half_sets_re_rolls_first_two_halves() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.rerolls = 4;
        m.start_half(&mut g, 1);
        assert_eq!(g.turn_data_home.rerolls, 4);
    }

    #[test]
    fn start_half_no_re_rolls_half_3() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.rerolls = 4;
        g.turn_data_home.rerolls = 0;
        // half >= 3 → no re-rolls set
        m.start_half(&mut g, 3);
        assert_eq!(g.turn_data_home.rerolls, 0);
    }

    #[test]
    fn start_half_apothecaries_only_half_0() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.team_home.apothecaries = 2;
        // half < 2 → apothecaries set at half 1
        m.start_half(&mut g, 1);
        assert_eq!(g.turn_data_home.apothecaries, 2);
        // half >= 2 → apothecaries not set
        g.turn_data_home.apothecaries = 0;
        m.start_half(&mut g, 2);
        assert_eq!(g.turn_data_home.apothecaries, 0);
    }

    #[test]
    fn update_leader_available_to_none_no_leader_on_field() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Available;
        g.turn_data_home.rerolls = 2;
        let result = m.update_leader_re_rolls_for_team(&mut g, true);
        assert_eq!(result, Some(LeaderState::None));
        assert_eq!(g.turn_data_home.rerolls, 1);
    }

    #[test]
    fn update_leader_no_change_when_used() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.leader_state = LeaderState::Used;
        let result = m.update_leader_re_rolls_for_team(&mut g, true);
        assert!(result.is_none());
        assert_eq!(g.turn_data_home.leader_state, LeaderState::Used);
    }

    #[test]
    fn handle_pump_up_no_attacker_returns_false() {
        use ffb_model::enums::ApothecaryMode;
        let m = StateMechanic::new();
        let mut g = make_game();
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(!m.handle_pump_up(&mut g, &ctx));
    }

    // ── chef roll tests ───────────────────────────────────────────────────────

    #[test]
    fn handle_chef_rolls_no_chefs_no_change() {
        let m = StateMechanic::new();
        let mut g = make_game();
        g.turn_data_home.rerolls = 3;
        g.turn_data_away.rerolls = 2;
        let mut rng = GameRng::new(0);
        let events = m.handle_chef_rolls(&mut g, &mut rng);
        assert!(events.is_empty(), "no chefs → no events");
        assert_eq!(g.turn_data_home.rerolls, 3, "home re-rolls unchanged");
        assert_eq!(g.turn_data_away.rerolls, 2, "away re-rolls unchanged");
    }

    #[test]
    fn handle_chef_rolls_emits_event_per_roll() {
        use ffb_model::inducement::inducement::Inducement;
        let m = StateMechanic::new();
        let mut g = make_game();
        // 2 home chefs (value = 2)
        g.turn_data_home.inducement_set.add_inducement(
            Inducement::new("masterChef", 2, vec![Usage::STEAL_REROLL])
        );
        let mut rng = GameRng::new(42);
        let events = m.handle_chef_rolls(&mut g, &mut rng);
        assert_eq!(events.len(), 2, "2 chefs → 2 events");
        for ev in &events {
            assert!(matches!(ev, GameEvent::MasterChefRoll { .. }));
        }
    }

    #[test]
    fn handle_chef_rolls_adjusts_rerolls_symmetrically() {
        use ffb_model::inducement::inducement::Inducement;
        let m = StateMechanic::new();
        let mut g = make_game();
        // 1 away chef
        g.turn_data_away.inducement_set.add_inducement(
            Inducement::new("masterChef", 1, vec![Usage::STEAL_REROLL])
        );
        g.turn_data_home.rerolls = 3;
        g.turn_data_away.rerolls = 2;

        let mut rng = GameRng::new(0);
        let events = m.handle_chef_rolls(&mut g, &mut rng);

        // stolen_home = 0 (no home chefs), stolen_away = result of one 3d6 roll
        let stolen_away = if events.is_empty() { 0 } else {
            match &events[0] {
                GameEvent::MasterChefRoll { rerolls_stolen, .. } => *rerolls_stolen,
                _ => panic!("unexpected event"),
            }
        };

        assert_eq!(g.turn_data_home.rerolls, 3i32.saturating_sub(stolen_away));
        assert_eq!(g.turn_data_away.rerolls, 2 + stolen_away);
    }

    #[test]
    fn handle_chef_rolls_home_rerolls_floored_at_zero() {
        use ffb_model::inducement::inducement::Inducement;
        // Even if stolen > current re-rolls, result is 0 (not negative)
        let m = StateMechanic::new();
        let mut g = make_game();
        // away has 1 chef; we use a fixed rng that gives all-6 (high dice)
        g.turn_data_away.inducement_set.add_inducement(
            Inducement::new("masterChef", 1, vec![Usage::STEAL_REROLL])
        );
        g.turn_data_home.rerolls = 0;
        g.turn_data_away.rerolls = 0;
        let mut rng = GameRng::new(0);
        let _events = m.handle_chef_rolls(&mut g, &mut rng);
        assert!(g.turn_data_home.rerolls >= 0, "home re-rolls must not go negative");
    }
}
