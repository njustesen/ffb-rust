/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.SaboteurBehaviour.
///
/// Hooks into StepDropFallingPlayers (StepId::DropFallingPlayers).
/// When attacker or defender has Saboteur and is falling after a block, they may
/// attempt a roll (4+) to inflict a Saboteur-type injury on the opposing player.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::{PlayerAction, PlayerState, SkillId, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2025::report_saboteur_roll::ReportSaboteurRoll;
use ffb_model::util::rng::GameRng;

// ── Hook state ────────────────────────────────────────────────────────────────

/// Java: StepDropFallingPlayers.StepState fields relevant to Saboteur.
/// Passed as `step_state` when StepDropFallingPlayers calls executeStepHooks.
#[derive(Debug)]
pub struct StepDropFallingPlayersHookState {
    /// Java: state.oldDefenderState
    pub old_defender_state: Option<PlayerState>,
    /// Java: state.saboteurTriggeredAttacker
    pub saboteur_triggered_attacker: bool,
    /// Java: state.usingSaboteurAttacker (Boolean tristate: None/Some(true)/Some(false))
    pub using_saboteur_attacker: Option<bool>,
    /// Java: state.saboteurTriggeredDefender
    pub saboteur_triggered_defender: bool,
    /// Java: state.usingSaboteurDefender (Boolean tristate: None/Some(true)/Some(false))
    pub using_saboteur_defender: Option<bool>,
}

impl StepDropFallingPlayersHookState {
    pub fn new() -> Self {
        Self {
            old_defender_state: None,
            saboteur_triggered_attacker: false,
            using_saboteur_attacker: None,
            saboteur_triggered_defender: false,
            using_saboteur_defender: None,
        }
    }
}

impl Default for StepDropFallingPlayersHookState {
    fn default() -> Self { Self::new() }
}

// ── Behaviour shell ───────────────────────────────────────────────────────────

pub struct SaboteurBehaviour;

impl SaboteurBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(SaboteurStepModifier));
        registry.register(SkillId::Saboteur, sb);
    }
}

impl Default for SaboteurBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SaboteurBehaviour {
    fn name(&self) -> &'static str { "SaboteurBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::Saboteur))
            .unwrap_or(false);
        if !has_skill {
            return false;
        }
        false
    }
}

// ── SaboteurStepModifier ──────────────────────────────────────────────────────

pub struct SaboteurStepModifier;

impl StepModifierTrait for SaboteurStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::DropFallingPlayers }

    fn priority(&self) -> i32 { 0 }

    /// Java: SaboteurBehaviour.handleExecuteStepHook(StepDropFallingPlayers step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepDropFallingPlayersHookState>()
            .expect("SaboteurStepModifier: step_state must be StepDropFallingPlayersHookState");

        // Java: ActingPlayer actingPlayer = game.getActingPlayer();
        //       Player<?> attacker = actingPlayer != null ? actingPlayer.getPlayer() : null;
        //       Player<?> defender = game.getDefender();
        //       if (actingPlayer == null || defender == null) return false;
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return false,
        };
        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return false,
        };

        // Java: FieldModel field = game.getFieldModel();
        //       PlayerState attackerState = field.getPlayerState(attacker);
        //       PlayerState defenderState = field.getPlayerState(defender);
        let attacker_state = game.field_model.player_state(&attacker_id);
        let defender_state = game.field_model.player_state(&defender_id);

        // Java: PlayerAction action = actingPlayer.getPlayerAction();
        //       boolean blockAction = action == BLOCK || action == BLITZ || action == MULTIPLE_BLOCK;
        let action = game.acting_player.player_action;
        let block_action = action == Some(PlayerAction::Block)
            || action == Some(PlayerAction::Blitz)
            || action == Some(PlayerAction::MultipleBlock);

        // ── Attacker Saboteur path ────────────────────────────────────────────
        //
        // Java: boolean attackerEligible = blockAction
        //         && !state.saboteurTriggeredAttacker
        //         && attacker.hasSkillProperty(canSabotageBlockerOnKnockdown)
        //         && attackerState.getBase() == FALLING;
        let attacker_eligible = block_action
            && !state.saboteur_triggered_attacker
            && game.player(&attacker_id)
                .map(|p| p.has_skill_property(NamedProperties::CAN_SABOTAGE_BLOCKER_ON_KNOCKDOWN))
                .unwrap_or(false)
            && attacker_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if attacker_eligible {
            if state.using_saboteur_attacker.is_none() {
                // Java: UtilCards.getSkillWithProperty(attacker, canSabotageBlockerOnKnockdown)
                //         .ifPresent(skill -> UtilServerDialog.showDialog(...,
                //           new DialogSkillUseParameter(attacker.getId(), skill, 0), false));
                //       step.getResult().setNextAction(StepAction.CONTINUE);
                //       return true;
                // headless: UtilServerDialog.showDialog not yet ported — skip dialog, return false
                return false;
            }
            if state.using_saboteur_attacker == Some(true) {
                // Java: int roll = step.getGameState().getDiceRoller().rollSkill();  // d6
                //       boolean success = roll >= 4;
                let roll = rng.d6();
                let success = roll >= 4;
                // Java: UtilCards.getSkillWithProperty(attacker, canSabotageBlockerOnKnockdown)
                //         .ifPresent(skill -> step.getResult().addReport(
                //           new ReportSaboteurRoll(attacker.getId(), success, roll, 4, false)));
                game.report_list.add(
                    ReportSaboteurRoll::new(Some(attacker_id.clone()), success, roll, 4, false),
                );
                if success {
                    state.saboteur_triggered_attacker = true;
                    // Java: if (defenderState != null && defenderState.getBase() != FALLING)
                    //           field.setPlayerState(defender, defenderState.changeBase(FALLING));
                    if let Some(ds) = defender_state {
                        if ds.base() != PS_FALLING {
                            game.field_model.set_player_state(
                                &defender_id,
                                ds.change_base(PS_FALLING),
                            );
                        }
                    }
                }
            }
            // Java: if (state.usingSaboteurAttacker == null) return true; — handled above
        }

        // ── Defender Saboteur path ────────────────────────────────────────────
        //
        // Java: boolean defenderEligible = blockAction
        //         && !state.saboteurTriggeredDefender
        //         && defender.hasUsableSkillProperty(canSabotageBlockerOnKnockdown, oldDefenderState)
        //         && defenderState.getBase() == FALLING;
        //
        // hasUsableSkillProperty not yet ported — stubbed as has_skill_property
        // (ignores the oldDefenderState "usable" check against player state)
        let defender_state_now = game.field_model.player_state(&defender_id);
        let defender_eligible = block_action
            && !state.saboteur_triggered_defender
            && game.player(&defender_id)
                .map(|p| p.has_skill_property(NamedProperties::CAN_SABOTAGE_BLOCKER_ON_KNOCKDOWN))
                .unwrap_or(false)
            && defender_state_now.map(|s| s.base() == PS_FALLING).unwrap_or(false);

        if defender_eligible {
            if state.using_saboteur_defender.is_none() {
                // Java: UtilCards.getSkillWithProperty(defender, canSabotageBlockerOnKnockdown)
                //         .ifPresent(skill -> UtilServerDialog.showDialog(...,
                //           new DialogSkillUseParameter(defender.getId(), skill, 0), true));
                //       step.getResult().setNextAction(StepAction.CONTINUE);
                //       return true;
                // headless: UtilServerDialog.showDialog not yet ported — skip dialog, return false
                return false;
            }
            if state.using_saboteur_defender == Some(true) {
                // Java: int roll = step.getGameState().getDiceRoller().rollSkill();  // d6
                //       boolean success = roll >= 4;
                let roll = rng.d6();
                let success = roll >= 4;
                // Java: UtilCards.getSkillWithProperty(defender, canSabotageBlockerOnKnockdown)
                //         .ifPresent(skill -> step.getResult().addReport(
                //           new ReportSaboteurRoll(defender.getId(), success, roll, 4, false)));
                game.report_list.add(
                    ReportSaboteurRoll::new(Some(defender_id.clone()), success, roll, 4, false),
                );
                if success {
                    state.saboteur_triggered_defender = true;
                    // Java: if (attackerState != null && attackerState.getBase() != FALLING)
                    //           field.setPlayerState(attacker, attackerState.changeBase(FALLING));
                    let attacker_state_now = game.field_model.player_state(&attacker_id);
                    if let Some(ats) = attacker_state_now {
                        if ats.base() != PS_FALLING {
                            game.field_model.set_player_state(
                                &attacker_id,
                                ats.change_base(PS_FALLING),
                            );
                        }
                    }
                }
            }
            // Java: if (state.usingSaboteurDefender == null) return true; — handled above
        }

        // Java: return false;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PlayerType, PlayerGender, Rules, PS_FALLING, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str, coord: FieldCoordinate, state_base: u32) {
        let p = ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
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
        };
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    fn give_saboteur(game: &mut Game, team: &str, player_id: &str) {
        let player = if team == "home" {
            game.team_home.player_mut(player_id)
        } else {
            game.team_away.player_mut(player_id)
        };
        if let Some(p) = player {
            p.extra_skills.push(SkillWithValue::new(SkillId::Saboteur));
        }
    }

    // ── Behaviour shell tests ─────────────────────────────────────────────────

    #[test]
    fn name_is_not_empty() {
        assert!(!SaboteurBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false_no_acting_player() {
        let b = SaboteurBehaviour::new();
        let mut game = make_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::roster_position::RosterPosition;
        let b = SaboteurBehaviour::new();
        let mut player = ffb_model::model::player::Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    // ── StepModifier registration tests ──────────────────────────────────────

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        SaboteurBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Saboteur).expect("Saboteur must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_drop_falling_players() {
        let m = SaboteurStepModifier;
        assert!(m.applies_to(StepId::DropFallingPlayers));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = SaboteurStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    // ── handleExecuteStepHook tests ───────────────────────────────────────────

    #[test]
    fn returns_false_when_no_acting_player() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let mut state = StepDropFallingPlayersHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
    }

    #[test]
    fn returns_false_when_no_defender() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        let mut state = StepDropFallingPlayersHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
    }

    #[test]
    fn attacker_without_saboteur_skill_returns_false() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
        assert!(!game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn attacker_saboteur_not_block_action_returns_false() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Foul);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
    }

    #[test]
    fn attacker_saboteur_headless_dialog_none_returns_false() {
        // In headless: using_saboteur_attacker is None, dialog skipped, returns false.
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
    }

    #[test]
    fn attacker_saboteur_declined_no_roll() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.using_saboteur_attacker = Some(false);
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
        assert!(!state.saboteur_triggered_attacker);
    }

    #[test]
    fn attacker_saboteur_accepted_roll_report_added() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.using_saboteur_attacker = Some(true);
        m.handle_execute_step(&mut game, &mut GameRng::new(42), &mut state);
        assert!(game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn attacker_saboteur_success_triggers_defender_falling() {
        let m = SaboteurStepModifier;
        let mut found = false;
        for seed in 0u64..100 {
            let mut g = make_game();
            let coord = FieldCoordinate::new(5, 5);
            add_player(&mut g, "home", "atk", coord, PS_FALLING);
            add_player(&mut g, "away", "def", coord, PS_STANDING);
            give_saboteur(&mut g, "home", "atk");
            g.acting_player.player_id = Some("atk".into());
            g.acting_player.player_action = Some(PlayerAction::Block);
            g.defender_id = Some("def".into());
            let mut st = StepDropFallingPlayersHookState::new();
            st.using_saboteur_attacker = Some(true);
            m.handle_execute_step(&mut g, &mut GameRng::new(seed), &mut st);
            if st.saboteur_triggered_attacker {
                let ds = g.field_model.player_state("def").unwrap();
                assert_eq!(ds.base(), PS_FALLING,
                    "defender should be FALLING after successful attacker Saboteur");
                found = true;
                break;
            }
        }
        assert!(found, "should find a seed where attacker Saboteur roll >= 4 succeeds");
    }

    #[test]
    fn attacker_already_triggered_skips_second_attempt() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.saboteur_triggered_attacker = true;
        state.using_saboteur_attacker = Some(true);
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn defender_saboteur_accepted_roll_report_added() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_STANDING);
        add_player(&mut game, "away", "def", coord, PS_FALLING);
        give_saboteur(&mut game, "away", "def");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.using_saboteur_defender = Some(true);
        m.handle_execute_step(&mut game, &mut GameRng::new(42), &mut state);
        assert!(game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn defender_saboteur_success_triggers_attacker_falling() {
        let m = SaboteurStepModifier;
        for seed in 0u64..100 {
            let mut g = make_game();
            let coord = FieldCoordinate::new(5, 5);
            add_player(&mut g, "home", "atk", coord, PS_STANDING);
            add_player(&mut g, "away", "def", coord, PS_FALLING);
            give_saboteur(&mut g, "away", "def");
            g.acting_player.player_id = Some("atk".into());
            g.acting_player.player_action = Some(PlayerAction::Block);
            g.defender_id = Some("def".into());
            let mut st = StepDropFallingPlayersHookState::new();
            st.using_saboteur_defender = Some(true);
            m.handle_execute_step(&mut g, &mut GameRng::new(seed), &mut st);
            if st.saboteur_triggered_defender {
                let ats = g.field_model.player_state("atk").unwrap();
                assert_eq!(ats.base(), PS_FALLING,
                    "attacker should be FALLING after successful defender Saboteur");
                return;
            }
        }
        panic!("should find a seed where defender Saboteur roll >= 4 succeeds");
    }

    #[test]
    fn blitz_action_is_block_eligible() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.using_saboteur_attacker = Some(true);
        m.handle_execute_step(&mut game, &mut GameRng::new(42), &mut state);
        assert!(game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn multiple_block_action_is_block_eligible() {
        let m = SaboteurStepModifier;
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "home", "atk", coord, PS_FALLING);
        add_player(&mut game, "away", "def", coord, PS_STANDING);
        give_saboteur(&mut game, "home", "atk");
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::MultipleBlock);
        game.defender_id = Some("def".into());
        let mut state = StepDropFallingPlayersHookState::new();
        state.using_saboteur_attacker = Some(true);
        m.handle_execute_step(&mut game, &mut GameRng::new(42), &mut state);
        assert!(game.report_list.has_report(ffb_model::report::ReportId::SABOTEUR_ROLL));
    }

    #[test]
    fn hook_state_default_all_false_none() {
        let s = StepDropFallingPlayersHookState::default();
        assert!(!s.saboteur_triggered_attacker);
        assert!(!s.saboteur_triggered_defender);
        assert_eq!(s.using_saboteur_attacker, None);
        assert_eq!(s.using_saboteur_defender, None);
        assert_eq!(s.old_defender_state, None);
    }
}
