/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.StandFirmBehaviour.
///
/// Priority 2 modifier on StepPushback.
///
/// **BB2020 vs BB2025 difference:** BB2020's Juggernaut-cancel branch checks real coordinate
/// adjacency between attacker and defender
/// (`getPlayerCoordinate(actingPlayer).isAdjacent(getPlayerCoordinate(state.defender))`),
/// while BB2025 simplifies this to an opposing-team check.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
// NOTE ON THE HOOK STATE TYPE: this behaviour is a step modifier on `StepId::Pushback`, and the
// step the driver actually runs for BB2020 is the SHARED `step::bb2025::block::StepPushback` (there
// is no `Rules::Bb2020` arm in `make_step_for`, so bb2020 falls through the default arm's
// `use crate::step::bb2025::block::*` glob). It therefore publishes
// `bb2025::block::step_pushback::StepPushbackHookState`, NOT the bb2020 struct of the same name.
// Downcasting to the bb2020 type returned None and `.expect()` aborted the process on the FIRST
// pushback of every bb2020 game.
//
// Java genuinely has separate bb2020/bb2025 `StepPushback` classes, but they differ in exactly one
// behavioural line (bb2025 publishes `PUSHED_ON_BALL`), while Rust's bb2020 translation is the
// staler of the two (356 lines of code against 433, and its hook state predates the `published` /
// `clear_pushback_stack` fields the Stand Firm fix needs). Per the bb2016 campaign's lesson —
// edition-gate the shared step, never route to the staler edition file — this uses the live hook
// state and keeps this file's BB2020-SPECIFIC LOGIC intact.
use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;

// ── StandFirmStepModifier ─────────────────────────────────────────────────────

pub struct StandFirmStepModifier;

impl StepModifierTrait for StandFirmStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 2 }

    /// Java: StandFirmBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("StandFirmStepModifier: step_state must be StepPushbackHookState");

        let defender_id = state.defender_id.clone();

        // Java: UtilCards.hasSkill(state.defender, skill)
        let has_stand_firm = game.player(&defender_id)
            .map(|p| p.has_skill(SkillId::StandFirm))
            .unwrap_or(false);

        // Java: UtilCards.getSkillCancelling(actingPlayer.getPlayer(), skill)
        let cancelling_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| {
                p.all_skill_ids().find(|id| {
                    id.properties().contains(&"cancelsCanRefuseToBePushed")
                })
            });

        let defender_state = game.field_model.player_state(&defender_id);

        // Java: if (playerState.isRooted()) standingFirm.put(id, true)
        if defender_state.map(|s| s.is_rooted()).unwrap_or(false) {
            state.standing_firm.insert(defender_id.clone(), true);
        } else {
            let has_tacklezones = defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
            let old_has_tacklezones = state.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
            // Java: ((pushbackStack.empty() && oldDefenderState != null && !oldDefenderState.hasTacklezones())
            //         || (!pushbackStack.empty() && !playerState.hasTacklezones())) && hasSkill(defender, skill)
            if has_stand_firm && (
                (state.pushback_stack_len == 0 && !old_has_tacklezones)
                || (state.pushback_stack_len > 0 && !has_tacklezones)
            ) {
                state.standing_firm.insert(defender_id.clone(), false);
                game.report_list.add(ReportSkillUse::new(
                    Some(defender_id.clone()),
                    SkillId::StandFirm,
                    false,
                    SkillUse::NO_TACKLEZONE,
                ));
            } else if let Some(cancel_skill) = cancelling_skill {
                // Java: BLITZ == actingPlayer.getPlayerAction() && cancellingSkill != null
                //       && hasSkill(defender, skill)
                //       && getPlayerCoordinate(actingPlayer).isAdjacent(getPlayerCoordinate(defender))
                let is_blitz = game.acting_player.player_action
                    .map(|a| a.is_blitzing())
                    .unwrap_or(false);
                let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
                let is_adjacent = game.field_model.player_coordinate(&attacker_id)
                    .zip(game.field_model.player_coordinate(&defender_id))
                    .map(|(a, d)| a.is_adjacent(d))
                    .unwrap_or(false);
                if has_stand_firm && is_blitz && is_adjacent {
                    state.standing_firm.insert(defender_id.clone(), false);
                    // Java: addReport(ReportSkillUse(actingPlayerId, cancellingSkill, true, CANCEL_STAND_FIRM))
                    game.report_list.add(ReportSkillUse::new(
                        Some(attacker_id),
                        cancel_skill,
                        true,
                        SkillUse::CANCEL_STAND_FIRM,
                    ));
                }
            }
        }

        // Java: if (hasSkill(defender, skill) && standingFirm.getOrDefault(id, true))
        if !has_stand_firm {
            return false;
        }

        let using_stand_firm = *state.standing_firm.get(&defender_id).unwrap_or(&true);
        if !using_stand_firm {
            return false;
        }

        // Java: `if (!standingFirm.containsKey(id)) showDialog(DialogSkillUseParameter(defender,
        // StandFirm))` and returns true; the client's answer comes back through `handleCommandHook`
        // and the step re-executes with the entry present.
        //
        // `ParityRunner`'s SKILL_USE arm ALWAYS uses the skill except DumpOff / PrimalSavagery /
        // SafePairOfHands / Swoop (`ParityRunner.java:748-751`) — Stand Firm is not in that list, so
        // Java answers USE and refuses the push. Rust auto-DECLINED, so the defender was pushed
        // where Java leaves it standing still (halfling bb2020 seed 1 i=100: the opposing Treeman
        // `h00` ends at 12,7 in Java and 11,6 in Rust, on identical dice — Take Root 2, block die 3
        // = pushback).
        //
        // The heuristic SCORES `SkillUse` and spends two sampler draws on it (Java's driver
        // answers the DialogSkillUseParameter through its useSkill sampler), so the old inline
        // auto-ACCEPT -- written for the RANDOM contract's free always-use answer -- left Rust
        // two draws short and split the streams (dark_elf bb2020 seed 1 i=11: Helmut Wulf's
        // Stand Firm against the blitz). Park the offer exactly like the bb2025 Sidestep
        // behaviour; `StepPushback` turns it into the prompt and files the answer back into
        // `standing_firm`.
        if !state.standing_firm.contains_key(&defender_id) {
            state.pending_skill_use = Some((defender_id.clone(), SkillId::StandFirm));
            return true;
        }

        // Java: state.doPush = true; pushbackStack.clear(); publish STARTING_PUSHBACK_SQUARE=null;
        //       publish FOLLOWUP_CHOICE=false; addReport(AVOID_PUSH)
        state.do_push = true;
        state.pushback_squares.clear();
        // Java `state.pushbackStack.clear()` empties the step's CHAIN stack too, not only the
        // candidate squares -- the step honours this via `clear_pushback_stack`.
        state.clear_pushback_stack = true;
        state.starting_pushback_square = None;
        // Java: `publishParameter(new StepParameter(StepParameterKey.FOLLOWUP_CHOICE, false))`.
        // The push was AVOIDED, so the defender's square is never vacated and the attacker must not
        // follow up. Leaving `followupChoice` unset lets `StepFollowup` fall into its
        // `followupChoice == null` block, where a `forceFollowup` attacker (Frenzy) publishes
        // FOLLOWUP_CHOICE=true and walks into the still-occupied square — necromantic bb2020 seed 1
        // i=15: the Werewolf `h00` stays at 12,7 in Java but lands on the Flesh Golem's 13,8 in Rust.
        // The bb2025 sibling already publishes this; the bb2020 copy did not.
        state.published.push(crate::step::framework::StepParameter::FollowupChoice(false));

        game.report_list.add(ReportSkillUse::new(
            Some(defender_id.clone()),
            SkillId::StandFirm,
            true,
            SkillUse::AVOID_PUSH,
        ));

        true
    }
}

// ── StandFirmBehaviour ────────────────────────────────────────────────────────

/// Stand Firm: player may ignore a push result and stay in their square.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(StandFirmStepModifier));
        registry.register(SkillId::StandFirm, sb);
    }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    // Same type re-point as the behaviour above: these tests must build the hook state the
    // LIVE shared step publishes, or the modifier's downcast fails at runtime.
    use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn player_with_skills(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    fn default_hook_state(defender_id: &str) -> StepPushbackHookState {
        StepPushbackHookState::new(
            defender_id.into(), None, None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        )
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        StandFirmBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::StandFirm).expect("StandFirm must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = StandFirmStepModifier;
        assert!(m.applies_to(StepId::Pushback));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = StandFirmStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_two() {
        let m = StandFirmStepModifier;
        assert_eq!(m.priority(), 2);
    }

    #[test]
    fn no_stand_firm_skill_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
    }

    /// Java shows a `DialogSkillUseParameter(defender, StandFirm)` when the choice has not been
    /// made yet, and `ParityRunner`'s SKILL_USE arm ALWAYS uses the skill except DumpOff /
    /// PrimalSavagery / SafePairOfHands / Swoop — Stand Firm is not among them. So the undecided
    /// case must auto-ACCEPT and refuse the push.
    ///
    /// This test previously asserted auto-DECLINE, which is what let the defender be pushed where
    /// Java leaves it standing (halfling bb2020 seed 1 i=100; halfling 5→98, wood_elf 15→98).
    /// Java: `if (!standingFirm.containsKey(id)) showDialog(DialogSkillUseParameter(StandFirm))`
    /// and CONTINUE — the offer is a DIALOG the heuristic driver answers with two sampler draws.
    /// The old inline auto-accept (written for the random contract's free always-use) left Rust
    /// two draws short (dark_elf bb2020 seed 1 i=11: Helmut Wulf). The hook PARKS the offer;
    /// StepPushback turns it into the SkillUse prompt and files the answer into `standing_firm`.
    #[test]
    fn stand_firm_not_decided_parks_the_offer() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result, "the pending offer stops pushback processing (Java CONTINUEs)");
        assert_eq!(hs.pending_skill_use, Some(("def1".into(), SkillId::StandFirm)));
        assert!(!hs.standing_firm.contains_key("def1"), "undecided until the answer arrives");
    }

    #[test]
    fn stand_firm_accepted_cancels_push() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);

        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result, "should return true when stand firm is accepted");
        assert!(hs.do_push, "do_push should be true after stand firm");
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared");
        assert!(hs.pushback_squares.is_empty(), "pushback squares should be cleared");
    }

    /// Java publishes `FOLLOWUP_CHOICE = false` on the avoid-push branch. Without it `StepFollowup`
    /// still sees `followupChoice == null` and a `forceFollowup` attacker (Frenzy) forces a
    /// follow-up into the square the defender never left — necromantic bb2020 seed 1 i=15.
    #[test]
    fn stand_firm_accepted_publishes_followup_choice_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);

        assert!(
            hs.published.iter().any(|p| matches!(
                p, crate::step::framework::StepParameter::FollowupChoice(false))),
            "avoid-push must publish FOLLOWUP_CHOICE(false), got {:?}", hs.published);
    }

    /// The re-entry after an ACCEPTED answer takes the avoid-push branch and publishes it.
    #[test]
    fn stand_firm_answered_use_publishes_followup_choice_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true); // the filed answer
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);

        assert!(
            hs.published.iter().any(|p| matches!(
                p, crate::step::framework::StepParameter::FollowupChoice(false))),
            "avoid-push must publish FOLLOWUP_CHOICE(false), got {:?}", hs.published);
    }

    #[test]
    fn stand_firm_accepted_adds_avoid_push_report() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);

        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn stand_firm_declined_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), false);

        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result, "should return false when stand firm declined");
        assert!(!hs.do_push, "do_push should stay false when declined");
    }

    #[test]
    fn stand_firm_with_no_tacklezone_auto_declines() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_PRONE));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.old_defender_state = Some(PlayerState::new(PS_PRONE));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.standing_firm.get("def1"), Some(&false));
    }

    #[test]
    fn stand_firm_rooted_auto_stands_firm() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        let rooted = PlayerState::new(PS_STANDING).change_rooted(true);
        game.field_model.set_player_state("def1", rooted);

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.standing_firm.get("def1"), Some(&true));
    }

}
