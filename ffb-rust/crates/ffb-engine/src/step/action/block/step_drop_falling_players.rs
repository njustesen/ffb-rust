/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepDropFallingPlayers (COMMON)
/// and its BB2016 hook com.fumbbl.ffb.server.skillbehaviour.bb2016.PilingOnBehaviour.
///
/// Drops any FALLING players (attacker and/or defender) and rolls armor+injury for them.
/// Also handles the PilingOn skill prompt (BB2016-only skill).
///
/// Phase 1 (usingPilingOn == None): drop the defender, roll their initial injury, and — if the
/// attacker is eligible (has an unused PilingOn skill, adjacent, not rooted/falling, defender not
/// already a casualty, game-option gates satisfied) — prompt the attacker and return Continue.
/// Phase 2 (usingPilingOn == Some): if accepted (and any required team re-roll is spent), mark the
/// skill used, drop the attacker, and re-roll the defender's injury (armor-only re-roll, or a pure
/// injury re-roll if armor was already broken) via InjuryTypePilingOnArmour/InjuryTypePilingOnInjury,
/// with a possible InjuryTypePilingOnKnockedOut on the attacker if a double was rolled and
/// PILING_ON_TO_KO_ON_DOUBLE is enabled.
///
/// Known follow-up (documented, not a silent gap): the BB2016 Weeping Dagger/poison side-effect on
/// a badly-hurt result (Java's `rollWeepingDagger`) is not yet ported.
///
/// Expects OLD_DEFENDER_STATE parameter from a preceding step.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id::{
    PILING_ON_ARMOR_ONLY, PILING_ON_INJURY_ONLY, PILING_ON_TO_KO_ON_DOUBLE,
    PILING_ON_USES_A_TEAM_REROLL,
};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::report::report_piling_on::ReportPilingOn;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::injuryType::injury_type_piling_on_armour::InjuryTypePilingOnArmour;
use crate::injury::injuryType::injury_type_piling_on_injury::InjuryTypePilingOnInjury;
use crate::injury::injuryType::injury_type_piling_on_knocked_out::InjuryTypePilingOnKnockedOut;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury, handle_injury_by_name};
use crate::util::util_server_re_roll::UtilServerReRoll;

pub struct StepDropFallingPlayers {
    /// Java: state.injuryResultDefender — populated after defender is dropped.
    pub injury_result_defender: Option<Box<InjuryResult>>,
    /// Java: state.usingPilingOn — None = not asked, Some = decided.
    pub using_piling_on: Option<bool>,
    /// Java: state.oldDefenderState — defender state before the block result was applied.
    pub old_defender_state: Option<ffb_model::enums::PlayerState>,
}

impl StepDropFallingPlayers {
    pub fn new() -> Self {
        Self {
            injury_result_defender: None,
            using_piling_on: None,
            old_defender_state: None,
        }
    }
}

impl Default for StepDropFallingPlayers {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropFallingPlayers {
    fn id(&self) -> StepId { StepId::DropFallingPlayers }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook — CLIENT_USE_SKILL sets usingPilingOn.
        if let Action::UseSkill { use_skill, .. } = action {
            self.using_piling_on = Some(*use_skill);
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepDropFallingPlayers {
    /// Java: PilingOnBehaviour.handleExecuteStepHook.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let defender_id = game.defender_id.clone();

        let defender_state = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id));
        let defender_is_falling = defender_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);
        let defender_coord = defender_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        let attacker_state = game.field_model.player_state(&player_id);
        let attacker_is_falling = attacker_state.map(|s| s.base() == PS_FALLING).unwrap_or(false);
        let attacker_coord = game.field_model.player_coordinate(&player_id);

        // Java: unroot FALLING players — a local-only adjustment used by the eligibility
        // checks below (Java never persists this back to the field model either).
        let defender_effective_rooted = !defender_is_falling
            && defender_state.map(|s| s.is_rooted()).unwrap_or(false);
        let attacker_effective_rooted = !attacker_is_falling
            && attacker_state.map(|s| s.is_rooted()).unwrap_or(false);

        let mut outcome = StepOutcome::next();

        if (defender_is_falling && defender_coord.is_some()) || self.using_piling_on.is_some() {
            if let Some(using) = self.using_piling_on {
                // Phase 2: PilingOn dialog answered.
                let re_roll_injury = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_armor_broken())
                    .unwrap_or(false);
                game.report_list.add(ReportPilingOn::new(player_id.clone(), using, re_roll_injury));

                let uses_a_team_reroll = is_option_enabled(game, PILING_ON_USES_A_TEAM_REROLL);
                let reroll_spent = !uses_a_team_reroll
                    || crate::step::util_server_re_roll::use_reroll(game, &ReRollSource::new("TRR"), &player_id);

                if using && reroll_spent {
                    game.mark_skill_used(&player_id, ffb_model::enums::SkillId::PilingOn);
                    for p in drop_player(game, &player_id, false) {
                        outcome = outcome.publish(p);
                    }

                    let dc = defender_coord.unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
                    let did = defender_id.clone().unwrap_or_default();
                    let (new_ir, rolled_double) = if re_roll_injury {
                        let old = self.injury_result_defender.as_deref();
                        let ir = handle_injury(
                            game, rng, &mut InjuryTypePilingOnInjury::new(),
                            Some(&player_id), &did, dc, None, old, ApothecaryMode::Defender,
                        );
                        let d = ir.injury_context.get_injury_roll()
                            .map(|r| DiceInterpreter::is_double(&r))
                            .unwrap_or(false);
                        (ir, d)
                    } else {
                        let ir = handle_injury(
                            game, rng, &mut InjuryTypePilingOnArmour::new(),
                            Some(&player_id), &did, dc, None, None, ApothecaryMode::Defender,
                        );
                        let d = ir.injury_context.get_armor_roll()
                            .map(|r| DiceInterpreter::is_double(&r))
                            .unwrap_or(false);
                        (ir, d)
                    };
                    self.injury_result_defender = Some(Box::new(new_ir));

                    if rolled_double && is_option_enabled(game, PILING_ON_TO_KO_ON_DOUBLE) {
                        let ac = attacker_coord.unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
                        let ko = handle_injury(
                            game, rng, &mut InjuryTypePilingOnKnockedOut::new(),
                            None, &player_id, ac, None, None, ApothecaryMode::Attacker,
                        );
                        outcome = outcome.publish(StepParameter::InjuryResult(Box::new(ko)));
                    }
                }
            } else {
                // Phase 1: initial defender drop.
                let did = defender_id.clone().unwrap_or_default();
                for p in drop_player(game, &did, false) {
                    outcome = outcome.publish(p);
                }

                let injury_type_name = match self.old_defender_state {
                    Some(s) if s.is_stunned() => "InjuryTypeBlockStunned",
                    Some(s) if s.is_prone_or_stunned() => "InjuryTypeBlockProne",
                    _ => "InjuryTypeBlock",
                };
                let dc = defender_coord.unwrap();
                let ir = handle_injury_by_name(
                    game, rng, injury_type_name, Some(&player_id), &did, dc, None, None,
                    ApothecaryMode::Defender,
                );
                self.injury_result_defender = Some(Box::new(ir));

                // Known follow-up: BB2016 Weeping Dagger/poison side-effect on a badly-hurt
                // result (Java `rollWeepingDagger`) is not yet ported — see SESSION.md.

                let uses_a_team_reroll = is_option_enabled(game, PILING_ON_USES_A_TEAM_REROLL);
                let armor_broken = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_armor_broken())
                    .unwrap_or(false);
                let is_casualty = self.injury_result_defender.as_ref()
                    .map(|ir| ir.injury_context.is_casualty())
                    .unwrap_or(false);
                let piling_on_eligible = !attacker_is_falling
                    && game.player(&player_id)
                        .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_PILE_ON_OPPONENT))
                        .unwrap_or(false)
                    && (!uses_a_team_reroll
                        || game.player(&player_id)
                            .map(|p| UtilServerReRoll::is_team_re_roll_available(game, p))
                            .unwrap_or(false))
                    && attacker_coord.zip(defender_coord)
                        .map(|(a, d)| a.is_adjacent(d))
                        .unwrap_or(false)
                    && !is_casualty
                    && !attacker_effective_rooted
                    && (!is_option_enabled(game, PILING_ON_INJURY_ONLY) || armor_broken)
                    && (!is_option_enabled(game, PILING_ON_ARMOR_ONLY) || !armor_broken)
                    && (!game.player(&did)
                        .map(|p| p.has_skill_property(NamedProperties::PREVENT_ARMOUR_MODIFICATIONS))
                        .unwrap_or(false)
                        || armor_broken)
                    && !game.player(&player_id)
                        .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::CAN_PILE_ON_OPPONENT))
                        .unwrap_or(false)
                    && !game.player(&did)
                        .map(|p| p.has_skill_property(NamedProperties::PREVENT_DAMAGING_INJURY_MODIFICATIONS))
                        .unwrap_or(false);

                if piling_on_eligible {
                    if let Some(ir) = self.injury_result_defender.as_deref_mut() {
                        ir.report(game);
                    }
                    return StepOutcome::cont().with_prompt(AgentPrompt::PilingOn {
                        player_id: player_id.clone(),
                        target_id: did,
                    });
                }
            }
        }

        if let Some(ir) = &self.injury_result_defender {
            outcome = outcome.publish(StepParameter::InjuryResult(ir.clone()));
        }
        if let Some(v) = self.using_piling_on {
            outcome = outcome.publish(StepParameter::UsingPilingOn(v));
        }

        // Java: if (defenderFalling && defender is own team && !oldDefenderState.isProne) → END_TURN
        let defender_is_own_team = {
            let did = defender_id.as_deref().unwrap_or("");
            game.team_home.player(&player_id).is_some() && game.team_home.player(did).is_some()
                || game.team_away.player(&player_id).is_some() && game.team_away.player(did).is_some()
        };
        if defender_is_falling
            && defender_is_own_team
            && self.old_defender_state
                .map(|s| !s.is_prone_or_stunned())
                .unwrap_or(false)
        {
            outcome = outcome.publish(StepParameter::EndTurn(true));
        }

        // Java: handle attacker FALLING.
        if attacker_is_falling && attacker_coord.is_some() {
            for p in drop_player(game, &player_id, false) {
                outcome = outcome.publish(p);
            }
            let ac = attacker_coord.unwrap();
            let ir = handle_injury_by_name(
                game, rng, "InjuryTypeBlock", defender_id.as_deref(), &player_id, ac, None, None,
                ApothecaryMode::Attacker,
            );
            outcome = outcome
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::InjuryResult(Box::new(ir)));
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{ApothecaryMode, Rules, PS_PRONE, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerState, SkillId};
    use ffb_model::types::FieldCoordinate;

    fn make_game_with_falling(attacker_falling: bool, defender_falling: bool) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        let make_player = |id: &str, nr: i32| ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        home.players.push(make_player("att", 1));
        away.players.push(make_player("def", 2));
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        let att_ps = if attacker_falling { PlayerState::new(PS_FALLING).change_active(true) }
                     else { PlayerState::new(PS_STANDING).change_active(true) };
        let def_ps = if defender_falling { PlayerState::new(PS_FALLING) }
                     else { PlayerState::new(PS_STANDING) };
        game.field_model.set_player_state("att", att_ps);
        game.field_model.set_player_state("def", def_ps);
        game
    }

    #[test]
    fn neither_falling_returns_next_no_pubs() {
        let mut game = make_game_with_falling(false, false);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.is_empty());
    }

    #[test]
    fn defender_falling_placed_prone_and_injury_result_published() {
        let mut game = make_game_with_falling(false, true);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn attacker_falling_placed_prone_end_turn_published() {
        let mut game = make_game_with_falling(true, false);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn injury_result_has_correct_apothecary_mode_for_defender() {
        let mut game = make_game_with_falling(false, true);
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        let defender_result = outcome.published.iter().find_map(|p| {
            if let StepParameter::InjuryResult(r) = p {
                if r.injury_context().get_apothecary_mode() == ApothecaryMode::Defender {
                    return Some(r.clone());
                }
            }
            None
        });
        assert!(defender_result.is_some(), "should have a Defender InjuryResult");
    }

    #[test]
    fn set_parameter_stores_old_defender_state() {
        let mut step = StepDropFallingPlayers::new();
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_some());
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    fn add_piling_on_skill(game: &mut Game, player_id: &str) {
        // Add PilingOn skill (canPileOnOpponent property) to the attacker.
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.extra_skills.push(SkillWithValue::new(SkillId::PilingOn));
        }
    }

    #[test]
    fn defender_falling_piling_on_eligible_returns_continue() {
        // When attacker has PilingOn and defender is falling, step returns Continue.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        // InjuryResult not yet published — waiting for dialog response.
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn piling_on_declined_publishes_injury_result() {
        // When player declines PilingOn (UseSkill false), InjuryResult is published.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        // First call: returns Continue (dialog shown).
        step.start(&mut game, &mut GameRng::new(0));
        // Player declines PilingOn.
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn piling_on_accepted_publishes_using_piling_on_true() {
        // When player accepts PilingOn, UsingPilingOn(true) is published.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingPilingOn(true))));
    }

    #[test]
    fn no_piling_on_skill_publishes_injury_result_immediately() {
        // Without PilingOn skill, no dialog — InjuryResult published on first call.
        let mut game = make_game_with_falling(false, true);
        // attacker has no PilingOn skill (default test_team players)
        let outcome = StepDropFallingPlayers::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn piling_on_accepted_marks_skill_used_and_drops_attacker() {
        // Java: performWrestle-equivalent — accepting PilingOn marks the skill used and
        // drops the attacker prone via UtilServerInjury.dropPlayer.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(
            game.player("att").unwrap().used_skills.contains(&SkillId::PilingOn),
            "PilingOn should be marked as used on the attacker"
        );
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn piling_on_accepted_publishes_a_fresh_injury_result() {
        // Accepting PilingOn re-rolls the defender's injury (armor-only, since the
        // initial roll may not have broken armor) and publishes the re-rolled result.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        let defender_result = outcome.published.iter().find_map(|p| {
            if let StepParameter::InjuryResult(r) = p {
                if r.injury_context().get_apothecary_mode() == ApothecaryMode::Defender {
                    return Some(r.clone());
                }
            }
            None
        });
        assert!(defender_result.is_some(), "re-rolled Defender InjuryResult should be published");
    }

    #[test]
    fn piling_on_uses_a_team_reroll_option_requires_available_reroll() {
        // Java: PILING_ON_USES_A_TEAM_REROLL — accepting PilingOn without an available
        // team re-roll must NOT mark the skill used or drop the attacker.
        let mut game = make_game_with_falling(false, true);
        add_piling_on_skill(&mut game, "att");
        game.options.set(ffb_model::option::game_option_id::PILING_ON_USES_A_TEAM_REROLL, "true");
        game.turn_data_mut().rerolls = 0;
        let mut step = StepDropFallingPlayers::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::PilingOn, use_skill: true },
            &mut game, &mut GameRng::new(0),
        );
        assert!(
            !game.player("att").unwrap().used_skills.contains(&SkillId::PilingOn),
            "PilingOn must not be marked used when the required team re-roll is unavailable"
        );
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_STANDING);
    }
}
