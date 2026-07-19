/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.StepWrestle (COMMON rules)
/// and its BB2016/BB2020/BB2025 hooks com.fumbbl.ffb.server.skillbehaviour.*.WrestleBehaviour.
///
/// Wrestle lets the attacker or defender convert a Both-Down result into both players going prone.
/// Two sequential dialogs: attacker first, then defender (if attacker declined).
/// Random agent always declines → neither uses Wrestle → NEXT_STEP.
///
/// When either player uses it: both attacker and defender are placed prone (simplified drop stub).
///
/// The three editions' gating logic genuinely differs (this is NOT identical across editions,
/// despite `handleExecuteStepHook`'s shared shape):
/// - **bb2016**: attacker gate = `hasSkill && !attackerState.isRooted()` (current live state).
///   defender gate = `hasSkill(defender) && !defenderState.isRooted()` (current live state, NOT
///   `oldDefenderState`). No Juggernaut-cancels-Wrestle check, no NO_TACKLEZONE decline report.
/// - **bb2020/bb2025**: attacker gate = `hasSkill` only (no rooted check at all). defender gate =
///   `hasSkill(defender) && oldDefenderState.hasTacklezones()`. Additionally checks
///   `wrestlePrevented = isBlitz && cancelsSkill(attacker, Wrestle) && hasSkill(defender)`
///   (Juggernaut cancelling Wrestle via `cancelsCanTakeDownPlayersWithHimOnBothDown`), reporting
///   `CANCEL_WRESTLE` when it fires. `performWrestle`'s decline branch also reports
///   `NO_TACKLEZONE` when the defender has Wrestle but lost tacklezones.
/// - **bb2025 only**: additionally reverts end-of-turn (`REVERT_END_TURN`) when the acting
///   player is carrying the ball, since being knocked prone would otherwise fumble it.
///
/// Expects OLD_DEFENDER_STATE parameter from a preceding step.
use ffb_model::enums::{PlayerAction, Rules, SkillId, PS_PRONE};
use ffb_model::enums::PlayerState;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::report::report_skill_use::ReportSkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWrestle {
    /// Java: state.usingWrestleAttacker — None = not asked, Some = answered.
    pub using_wrestle_attacker: Option<bool>,
    /// Java: state.usingWrestleDefender — None = not asked, Some = answered.
    pub using_wrestle_defender: Option<bool>,
    /// Java: state.oldDefenderState — defender state before the block result was applied.
    pub old_defender_state: Option<PlayerState>,
}

impl StepWrestle {
    pub fn new() -> Self {
        Self {
            using_wrestle_attacker: None,
            using_wrestle_defender: None,
            old_defender_state: None,
        }
    }
}

impl Default for StepWrestle {
    fn default() -> Self { Self::new() }
}

impl Step for StepWrestle {
    fn id(&self) -> StepId { StepId::Wrestle }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommandHook — attacker answered first, then defender.
        if let Action::UseSkill { use_skill, .. } = action {
            if self.using_wrestle_attacker.is_none() {
                self.using_wrestle_attacker = Some(*use_skill);
            } else {
                self.using_wrestle_defender = Some(*use_skill);
            }
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

impl StepWrestle {
    /// Java: WrestleBehaviour.handleExecuteStepHook — logic genuinely diverges bb2016 vs
    /// bb2020/bb2025 (see module doc comment for the differences).
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let is_bb2016 = game.rules == Rules::Bb2016;

        // Java: askAttackerForWrestleUse.
        // bb2016: hasSkill && !attackerState.isRooted() (current live state).
        // bb2020/bb2025: hasSkill only (no rooted check).
        if self.using_wrestle_attacker.is_none() {
            let has_skill = game.player(&player_id).map(|p| p.has_skill(SkillId::Wrestle)).unwrap_or(false);
            let attacker_can_use = if is_bb2016 {
                has_skill && !game.field_model.player_state(&player_id).map(|s| s.is_rooted()).unwrap_or(false)
            } else {
                has_skill
            };
            if attacker_can_use {
                return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                    player_id,
                    skill_id: SkillId::Wrestle as u16,
                    skill_name: format!("{:?}", SkillId::Wrestle),
                });
            } else {
                self.using_wrestle_attacker = Some(false);
            }
        }

        // Java: askDefenderForWrestleUse.
        // bb2016: hasSkill(defender) && !defenderState.isRooted() (current live state); no
        //   Juggernaut-cancels-Wrestle check.
        // bb2020/bb2025: hasSkill(defender) && oldDefenderState.hasTacklezones(); plus
        //   wrestlePrevented = isBlitz && cancelsSkill(attacker, Wrestle) && hasSkill(defender),
        //   reporting CANCEL_WRESTLE when it fires.
        if self.using_wrestle_defender.is_none() {
            let defender_id = game.defender_id.clone();
            let defender_has_skill = defender_id.as_deref()
                .map(|id| game.player(id).map(|p| p.has_skill(SkillId::Wrestle)).unwrap_or(false))
                .unwrap_or(false);
            let defender_can_use = if is_bb2016 {
                defender_has_skill
                    && defender_id.as_deref()
                        .map(|id| !game.field_model.player_state(id).map(|s| s.is_rooted()).unwrap_or(false))
                        .unwrap_or(false)
            } else {
                defender_has_skill
                    && self.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(false)
            };
            let is_blitz = game.acting_player.player_action == Some(PlayerAction::Blitz);
            // Java: cancelsSkill(actingPlayer.getPlayer(), skill) — e.g. Juggernaut cancels Wrestle
            // via cancelsCanTakeDownPlayersWithHimOnBothDown. bb2016 has no such check.
            let wrestle_prevented = !is_bb2016
                && is_blitz
                && defender_has_skill
                && game.player(&player_id)
                    .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN))
                    .unwrap_or(false);
            let attacker_declined = self.using_wrestle_attacker == Some(false);
            if attacker_declined && defender_can_use && !wrestle_prevented {
                if let Some(did) = defender_id {
                    return StepOutcome::cont().with_prompt(AgentPrompt::SkillUse {
                        player_id: did,
                        skill_id: SkillId::Wrestle as u16,
                        skill_name: format!("{:?}", SkillId::Wrestle),
                    });
                }
            } else {
                if wrestle_prevented {
                    // Java: addReport(new ReportSkillUse(actingPlayerId, getSkillCancelling(...), true, CANCEL_WRESTLE))
                    let cancelling = game.player(&player_id)
                        .and_then(|p| UtilCards::get_skill_cancelling_property(p, NamedProperties::CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN));
                    if let Some(skill_id) = cancelling {
                        game.report_list.add(ReportSkillUse::new(
                            Some(player_id.clone()),
                            skill_id,
                            true,
                            SkillUse::CANCEL_WRESTLE,
                        ));
                    }
                }
                self.using_wrestle_defender = Some(false);
            }
        }

        // Java: performWrestle
        self.perform_wrestle(game, &player_id, is_bb2016)
    }

    fn perform_wrestle(&self, game: &mut Game, player_id: &str, is_bb2016: bool) -> StepOutcome {
        let using_attacker = self.using_wrestle_attacker.unwrap_or(false);
        let using_defender = self.using_wrestle_defender.unwrap_or(false);
        let defender_id = game.defender_id.clone();
        // Java: boolean defenderHasTacklezones = state.oldDefenderState.hasTacklezones();
        let defender_has_tacklezones = self.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);

        let mut events = Vec::new();
        let skill_num = SkillId::Wrestle as u16;

        if using_attacker {
            // Java: addReport(new ReportSkillUse(actingPlayerId, skill, true, BRING_DOWN_OPPONENT))
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.to_string()),
                SkillId::Wrestle,
                true,
                SkillUse::BRING_DOWN_OPPONENT,
            ));
            events.push(GameEvent::SkillUse { player_id: player_id.to_string(), skill_id: skill_num, used: true });
        } else if using_defender {
            if let Some(did) = &defender_id {
                // Java: addReport(new ReportSkillUse(defenderId, skill, true, BRING_DOWN_OPPONENT))
                game.report_list.add(ReportSkillUse::new(
                    Some(did.clone()),
                    SkillId::Wrestle,
                    true,
                    SkillUse::BRING_DOWN_OPPONENT,
                ));
                events.push(GameEvent::SkillUse { player_id: did.clone(), skill_id: skill_num, used: true });
            }
        } else {
            let attacker_has = game.player(player_id).map(|p| p.has_skill(SkillId::Wrestle)).unwrap_or(false);
            let defender_has = defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill(SkillId::Wrestle))
                .unwrap_or(false);
            // Java (bb2020/bb2025 only): if (!defenderHasTacklezones && hasSkill(defender)) →
            // NO_TACKLEZONE report; bb2016 has no such branch.
            if !is_bb2016 && !defender_has_tacklezones && defender_has {
                game.report_list.add(ReportSkillUse::new(
                    defender_id.clone(),
                    SkillId::Wrestle,
                    false,
                    SkillUse::NO_TACKLEZONE,
                ));
            } else if attacker_has || defender_has {
                events.push(GameEvent::SkillUse { player_id: player_id.to_string(), skill_id: skill_num, used: false });
            }
        }

        // Java (bb2025 only): if actingPlayer carries the ball, revert end-of-turn (they're
        // about to be knocked prone, which would otherwise end the turn on ball-carrier down).
        // Java: `if (state.usingWrestleAttacker || state.usingWrestleDefender) { if
        // (UtilPlayer.hasBall(game, actingPlayer.getPlayer())) ... }` — gated on either flag,
        // but always checks the ACTING PLAYER's ball possession (not the defender's).
        let revert_end_turn = (using_attacker || using_defender)
            && game.rules == Rules::Bb2025
            && UtilPlayer::has_ball(game, player_id);

        if using_attacker || using_defender {
            // Java: UtilServerInjury.dropPlayer → place both PRONE.
            // Simplified stub: set both to PRONE, deactivate.
            let attacker_state = game.field_model.player_state(player_id)
                .unwrap_or_default();
            game.field_model.set_player_state(player_id, attacker_state.change_base(PS_PRONE).change_active(false));

            if let Some(did) = &defender_id {
                let defender_state = game.field_model.player_state(did).unwrap_or_default();
                game.field_model.set_player_state(did, defender_state.change_base(PS_PRONE).change_active(false));
            }
        }

        let mut outcome = StepOutcome::next();
        for e in events {
            outcome = outcome.with_event(e);
        }
        if revert_end_turn {
            outcome = outcome.publish(StepParameter::RevertEndTurn(true));
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        skills: Vec<SkillId>,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills
                .into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
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
        });
    }

    fn make_game(attacker_skills: Vec<SkillId>, defender_skills: Vec<SkillId>) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, attacker_skills);
        add_player(&mut away, "def", 2, defender_skills);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game(vec![], vec![]);
        game.acting_player.player_id = None;
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn neither_has_wrestle_returns_next_no_events() {
        let mut game = make_game(vec![], vec![]);
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn attacker_has_wrestle_prompts_skill_use() {
        // Java: askAttackerForWrestleUse → CONTINUE with a dialog when attacker can use Wrestle.
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(matches!(outcome.prompt, Some(AgentPrompt::SkillUse { .. })));
    }

    #[test]
    fn defender_has_wrestle_prompts_skill_use_after_attacker_declines() {
        // Attacker already declined; defender has Wrestle and (bb2025) oldDefenderState still has
        // tacklezones → prompt defender. Rules::Bb2025 uses oldDefenderState.hasTacklezones(),
        // not the defender's current live state, so OLD_DEFENDER_STATE must be set (as a
        // preceding step would set it in the real sequence).
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(matches!(outcome.prompt, Some(AgentPrompt::SkillUse { .. })));
    }

    #[test]
    fn bb2020_2025_defender_gate_uses_old_defender_state_not_live_state() {
        // Real bug: the defender gate for bb2020/bb2025 must read `oldDefenderState`
        // (state.oldDefenderState.hasTacklezones()), not the defender's current live
        // field-model state. Here the defender's *current* state has no tacklezones (stunned),
        // but their *old* state (before the block result was applied) did — so the dialog
        // must still be offered, matching Java's `state.oldDefenderState.hasTacklezones()`.
        use ffb_model::enums::PS_STUNNED;
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        game.field_model.set_player_state("def", PlayerState::new(PS_STUNNED));
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        step.old_defender_state = Some(PlayerState::new(PS_STANDING));
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue, "must gate on oldDefenderState, not live state");
        assert!(matches!(outcome.prompt, Some(AgentPrompt::SkillUse { .. })));
    }

    #[test]
    fn bb2016_attacker_gate_has_no_rooted_check_regression_guard() {
        // bb2016's askAttackerForWrestleUse DOES check !isRooted (unlike bb2020/bb2025). This
        // guards against accidentally dropping bb2016's rooted check while fixing bb2020/2025.
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        game.rules = Rules::Bb2016;
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_rooted(true));
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        // Rooted attacker in bb2016 cannot use Wrestle → falls through (declines) → NextStep
        // (no defender skill either in this fixture).
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn bb2025_attacker_gate_ignores_rooted_state() {
        // bb2020/bb2025's askAttackerForWrestleUse has NO rooted check (unlike bb2016) — a
        // rooted attacker with Wrestle must still be offered the dialog.
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_rooted(true));
        let outcome = StepWrestle::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue, "bb2025 has no rooted gate for the attacker");
    }

    #[test]
    fn neither_can_use_wrestle_after_decline_emits_declined_event() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: false, .. })));
    }

    #[test]
    fn attacker_uses_wrestle_drops_both_prone() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(true);
        step.using_wrestle_defender = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn defender_uses_wrestle_drops_both_prone() {
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        step.using_wrestle_defender = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_state("att").unwrap().base(), PS_PRONE);
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn attacker_uses_wrestle_emits_skill_used_event() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(true);
        step.using_wrestle_defender = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert!(outcome.events.iter().any(|e| matches!(e, GameEvent::SkillUse { used: true, .. })));
    }

    #[test]
    fn attacker_uses_wrestle_adds_skill_use_report() {
        let mut game = make_game(vec![SkillId::Wrestle], vec![]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(true);
        step.using_wrestle_defender = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE),
            "attacker using Wrestle should add ReportSkillUse"
        );
    }

    #[test]
    fn defender_uses_wrestle_adds_skill_use_report() {
        let mut game = make_game(vec![], vec![SkillId::Wrestle]);
        let mut step = StepWrestle::new();
        step.using_wrestle_attacker = Some(false);
        step.using_wrestle_defender = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE),
            "defender using Wrestle should add ReportSkillUse"
        );
    }

    #[test]
    fn set_parameter_stores_old_defender_state() {
        let mut step = StepWrestle::new();
        assert!(step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING))));
        assert!(step.old_defender_state.is_some());
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
