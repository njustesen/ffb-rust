use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, VictimStateKey};
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::drop_player_rng;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepHandleDropPlayerContext.
///
/// Processes a pending DropPlayerContext (injury follow-up after SteadyFooting fails).
/// Drops the player (unless already dropped or armour break required but not achieved),
/// publishes END_TURN if flagged, and routes to the context's label if one is set.
///
/// Stubbed: ModifiedInjuryContext / Pro skill re-roll — these require ModifiedInjuryContext
/// which is not yet ported. The step skips that branch and proceeds directly to the drop.
///
/// Consumed parameters: DROP_PLAYER_CONTEXT.
/// Published: parameters from drop_player() + INJURY_RESULT + END_TURN + GOTO_LABEL.
pub struct StepHandleDropPlayerContext {
    /// Java: dropPlayerContext
    pub drop_player_context: Option<Box<DropPlayerContext>>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    /// Java: playerId — the acting player offered the conditional-reroll SkillUse dialog.
    pub player_id: Option<String>,
    /// Java: skill — the modification's skill (e.g. Old Pro), from the modified context.
    pub skill: Option<ffb_model::enums::SkillId>,
    /// Java: setParameter(SUCCESSFUL_PRO) applies immediately (it has gameState access);
    /// Rust's set_parameter has no game, so the verdict is stored and applied at the next
    /// execute (the step was push_self'd below the PRO sequence, so it re-runs right after).
    pub pending_pro: Option<bool>,
}

impl StepHandleDropPlayerContext {
    pub fn new() -> Self {
        Self {
            drop_player_context: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id: None,
            skill: None,
            pending_pro: None,
        }
    }
}

impl Default for StepHandleDropPlayerContext {
    fn default() -> Self { Self::new() }
}

impl Step for StepHandleDropPlayerContext {
    fn id(&self) -> StepId { StepId::HandleDropPlayerContext }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java handleCommand CLIENT_USE_SKILL (:81-101): report the choice with the modified
        // context's SkillUse; USING a conditional-reroll skill (Old Pro) pushes a PRO
        // sub-sequence and resumes this step afterwards; USING an unconditional one swaps to
        // the alternate context immediately.
        if let Action::UseSkill { skill_id, use_skill } = action {
            // Both harness agents answer the SkillUse dialog with a PLACEHOLDER skill id
            // (SkillId::Block, "engine identifies the skill from step state") — the real skill
            // is the one stored when the dialog was shown, exactly Java's `skill` field.
            let skill = self.skill.unwrap_or(*skill_id);
            let modified_skill_use = self.drop_player_context.as_ref()
                .and_then(|c| c.injury_result.as_ref())
                .and_then(|ir| ir.injury_context().modified_injury_context.as_ref())
                .and_then(|m| m.skill_use_modification);
            let player_id = self.player_id.clone().or_else(|| game.acting_player.player_id.clone());
            game.report_list.add(ReportSkillUse::new(
                player_id.clone(),
                skill,
                *use_skill,
                modified_skill_use.unwrap_or(SkillUse::WOULD_NOT_HELP),
            ));
            if *use_skill {
                let requires_conditional =
                    crate::injury::modification::modification_for_skill(skill, game.rules)
                        .map(|m| m.requires_conditional_re_roll_skill())
                        .unwrap_or(false);
                if requires_conditional {
                    // Java: pushCurrentStepOnStack(); push Sequence[PRO(PLAYER_ID)]; NEXT_STEP.
                    use crate::step::generator::sequence::Sequence;
                    let mut seq = Sequence::new();
                    seq.add(StepId::Pro, vec![StepParameter::PlayerId(player_id.clone().unwrap_or_default())]);
                    let mut out = StepOutcome::next();
                    out.push_self = true;
                    return out.push_seq(seq.build());
                } else {
                    // Java: player.markUsed(skill) + successfulSkillUse(injuryResult).
                    if let Some(pid) = player_id.as_deref() {
                        if let Some(p) = game.player_mut(pid) { p.used_skills.insert(skill); }
                    }
                    self.successful_skill_use(game);
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::DropPlayerContext(ctx) => {
                self.drop_player_context = Some(ctx.clone());
                true
            }
            // Java: SUCCESSFUL_PRO applies markUsed + successfulSkillUse in setParameter
            // (gameState in scope there); Rust stores the verdict and applies at re-execute.
            StepParameter::SuccessfulPro(v) => {
                self.pending_pro = Some(*v);
                true
            }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys (SUCCESSFUL_PRO is consumed in Java even
    // though the Rust set_parameter does not handle it yet).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::DropPlayerContext(_) | StepParameter::SuccessfulPro(_))
    }
}

impl StepHandleDropPlayerContext {
    /// Java: `successfulSkillUse(InjuryResult)` — replay the modified context's reports,
    /// swap the injury result to the alternate context, and take over its end-turn flag.
    fn successful_skill_use(&mut self, game: &mut Game) {
        if let Some(ctx) = self.drop_player_context.as_mut() {
            if let Some(ir) = ctx.injury_result.as_mut() {
                ir.swap_to_alternate_context(game);
            }
            ctx.end_turn = ctx.modified_injury_ends_turn;
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java setParameter(SUCCESSFUL_PRO) — applied here because Rust's set_parameter has no
        // game access; the PRO sub-sequence finished and this step was resumed via push_self.
        if let Some(successful) = self.pending_pro.take() {
            if successful {
                if let (Some(pid), Some(skill)) = (self.player_id.clone(), self.skill) {
                    if let Some(p) = game.player_mut(&pid) { p.used_skills.insert(skill); }
                }
                self.successful_skill_use(game);
            } else if let Some(ctx) = self.drop_player_context.as_mut() {
                if let Some(ir) = ctx.injury_result.as_mut() {
                    ir.injury_context_mut().modified_injury_context = None;
                }
            }
        }

        // Java :128: a pending ModifiedInjuryContext (an InjuryContextModification fired, e.g.
        // Old Pro) — report the original injury, then offer the SkillUse dialog and WAIT.
        {
            let needs_dialog = self.drop_player_context.as_ref()
                .and_then(|c| c.injury_result.as_ref())
                .map(|ir| ir.injury_context().modified_injury_context.is_some()
                    && !ir.is_already_reported())
                .unwrap_or(false);
            if needs_dialog {
                let mut event = None;
                let mut skill = None;
                if let Some(ctx) = self.drop_player_context.as_mut() {
                    if let Some(ir) = ctx.injury_result.as_mut() {
                        event = ir.report(game);
                        skill = ir.injury_context().modified_injury_context.as_ref()
                            .and_then(|m| m.used_skill_id);
                    }
                }
                // Java: playerId = game.getActingPlayer().getPlayerId(); skill = modified.getUsedSkill().
                self.player_id = game.acting_player.player_id.clone();
                self.skill = skill;
                let skill = skill.unwrap_or(ffb_model::enums::SkillId::OldPro);
                let mut out = StepOutcome::cont().with_prompt(
                    ffb_model::prompts::AgentPrompt::SkillUse {
                        player_id: self.player_id.clone().unwrap_or_default(),
                        skill_id: skill as u16,
                        skill_name: format!("{:?}", skill),
                    });
                if let Some(e) = event { out = out.with_event(e); }
                return out;
            }
        }

        let ctx = match &self.drop_player_context {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let injury_result = match &ctx.injury_result {
            Some(r) => r,
            None => return StepOutcome::next(),
        };

        let mut out = StepOutcome::next();

        // Java: if (!alreadyDropped && (!requiresArmourBreak || armorBroken)) → dropPlayer(...)
        let should_drop = !ctx.already_dropped
            && (!ctx.requires_armour_break || injury_result.injury_context().is_armor_broken());

        if should_drop {
            let player_id = match &ctx.player_id {
                Some(id) => id.clone(),
                None => return StepOutcome::next(),
            };
            // Coverage event: block-knockdown (and other injury-context) falls resolve here.
            // Only emit when the player actually goes down now (not already prone/stunned) —
            // the monolith emitted PlayerFellDown at each prone placement.
            let was_upright = game.field_model.player_state(&player_id)
                .map(|s| !s.is_prone_or_stunned())
                .unwrap_or(false);
            let fell_coord = game.field_model.player_coordinate(&player_id);
            // Java: publishParameters(UtilServerInjury.dropPlayer(this, player, mode, spoh)).
            //
            // `drop_player_rng` is the full port — it does the `placedProneCausesInjuryRoll` (Ball &
            // Chain) branch, which needs an rng, AND the ball handling that Java places OUTSIDE
            // that if/else and therefore runs for a B&C player too. This step used to inline the
            // injury half and return no parameters, so dropping a Ball & Chain player standing on a
            // loose ball never set SCATTER_BALL: Java bounced the ball off the Fanatic's square
            // (a d8) and Rust did not (goblin bb2020 seed 98 i=10, rng 20).
            let drop_params = drop_player_rng(
                game,
                rng,
                &player_id,
                ctx.eligible_for_safe_pair_of_hands,
                ctx.apothecary_mode.unwrap_or(ffb_model::enums::ApothecaryMode::Defender),
            );
            if was_upright {
                if let Some(coord) = fell_coord {
                    out = out.with_event(ffb_model::events::GameEvent::PlayerFellDown {
                        player_id: player_id.clone(),
                        coord,
                    });
                }
            }
            for p in drop_params {
                out = out.publish(p);
            }

            // Java: if (endTurn) publishParameter(END_TURN, true)
            if ctx.end_turn {
                out = out.publish(StepParameter::EndTurn(true));
            }

            // Java: if (victimStateKey != null) publishParameter(victimStateKey, defenderState)
            // Java: game.getFieldModel().getPlayerState(game.getDefender()) — uses the game's
            // defenderId (the victim of the block), not the acting player.
            if let Some(vsk) = ctx.victim_state_key {
                let defender_state = game.field_model.player_state(
                    game.defender_id.as_deref().unwrap_or("")
                );
                if let Some(state) = defender_state {
                    let param = match vsk {
                        VictimStateKey::OldDefenderState => StepParameter::OldDefenderState(state),
                        VictimStateKey::ThrownPlayerState => StepParameter::ThrownPlayerState(state),
                        VictimStateKey::OldPlayerState => StepParameter::OldPlayerState(state),
                        VictimStateKey::KickedPlayerState => StepParameter::KickedPlayerState(state),
                    };
                    out = out.publish(param);
                }
            }

            // Java: additionalVictimStateKeys loop (same pattern)
            for &vsk in &ctx.additional_victim_state_keys {
                let defender_state = game.field_model.player_state(
                    game.defender_id.as_deref().unwrap_or("")
                );
                if let Some(state) = defender_state {
                    let param = match vsk {
                        VictimStateKey::OldDefenderState => StepParameter::OldDefenderState(state),
                        VictimStateKey::ThrownPlayerState => StepParameter::ThrownPlayerState(state),
                        VictimStateKey::OldPlayerState => StepParameter::OldPlayerState(state),
                        VictimStateKey::KickedPlayerState => StepParameter::KickedPlayerState(state),
                    };
                    out = out.publish(param);
                }
            }
        } else if !ctx.already_dropped && ctx.end_turn_without_knockdown && ctx.end_turn {
            // Java: else if (!alreadyDropped && endTurnWithoutKnockdown && endTurn)
            out = out.publish(StepParameter::EndTurn(true));
        }

        // Java: publishParameter(INJURY_RESULT, injuryResult) — always
        out = out.publish(StepParameter::InjuryResult(injury_result.clone()));

        // Java: if (StringTool.isProvided(label)) GOTO_LABEL(label)
        if let Some(label) = &ctx.label {
            if !label.is_empty() {
                out.action = crate::step::framework::StepAction::GotoLabel;
                out.goto_label = Some(label.clone());
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_player_context::DropPlayerContext;
    use crate::injury::{InjuryContext, InjuryResult};
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{ApothecaryMode, Rules, PS_STANDING, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    fn make_injury_result() -> Box<InjuryResult> {
        Box::new(InjuryResult::new(ApothecaryMode::Defender))
    }

    #[test]
    fn modified_injury_context_offers_skill_use_then_pushes_pro_and_swaps() {
        // Java StepHandleDropPlayerContext: a pending ModifiedInjuryContext (Old Pro fired in
        // the injury pipeline) → report + DialogSkillUseParameter (CONTINUE); CLIENT_USE_SKILL
        // with a conditional-reroll skill pushes a PRO sub-sequence; SUCCESSFUL_PRO(true) marks
        // the skill used and swaps the injury result to the alternate context.
        use ffb_model::prompts::AgentPrompt;
        let mut game = make_game();
        add_player(&mut game, "h1");
        game.acting_player.player_id = Some("h1".into());

        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.defender_id = Some("h1".into());
        ctx.armor_broken = true;
        ctx.injury = Some(ffb_model::enums::PlayerState::new(ffb_model::enums::PS_KNOCKED_OUT));
        let mut modified = ctx.clone();
        modified.armor_broken = false;
        modified.injury = Some(ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE));
        modified.used_skill_id = Some(SkillId::OldPro);
        ctx.modified_injury_context = Some(Box::new(modified));

        let mut dpc = make_dpc("h1");
        dpc.injury_result = Some(Box::new(InjuryResult {
            injury_context: ctx,
            knocked_out: false, rip: false, already_reported: false, pre_regeneration: true,
        }));
        let mut step = StepHandleDropPlayerContext::new();
        step.set_parameter(&StepParameter::DropPlayerContext(dpc));

        // Phase 1: dialog offered, step waits.
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::SkillUse { .. })));
        assert_eq!(step.skill, Some(SkillId::OldPro));

        // Phase 2: coach uses the (conditional-reroll) skill → PRO sub-sequence + push_self.
        let out = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Block, use_skill: true },
            &mut game, &mut GameRng::new(0));
        assert!(out.push_self, "the step must resume after the PRO sequence");
        assert!(out.pushes.iter().flatten().any(|s| s.step_id == StepId::Pro),
            "a PRO step must be pushed");

        // Phase 3: StepPro published SUCCESSFUL_PRO(true) → swap to the alternate context.
        assert!(step.set_parameter(&StepParameter::SuccessfulPro(true)));
        step.start(&mut game, &mut GameRng::new(0));
        let ir = step.drop_player_context.as_ref().unwrap().injury_result.as_ref().unwrap();
        assert!(ir.injury_context().modified_injury_context.is_none());
        assert!(!ir.injury_context().armor_broken,
            "the injury result must now BE the alternate (armour-saved) context");
        assert!(game.player("h1").unwrap().used_skills.contains(&SkillId::OldPro),
            "Old Pro must be marked used");
    }

    fn make_dpc(player_id: &str) -> Box<DropPlayerContext> {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        Box::new(DropPlayerContext {
            injury_result: Some(Box::new(InjuryResult {
                injury_context: ctx,
                knocked_out: false,
                rip: false,
                already_reported: false,
                pre_regeneration: true,
            })),
            player_id: Some(player_id.to_owned()),
            apothecary_mode: Some(ApothecaryMode::Defender),
            eligible_for_safe_pair_of_hands: true,
            ..DropPlayerContext::new()
        })
    }

    #[test]
    fn no_context_returns_next() {
        let mut game = make_game();
        let mut step = StepHandleDropPlayerContext::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_drop_player_context_accepted() {
        let mut step = StepHandleDropPlayerContext::new();
        let dpc = DropPlayerContext { injury_result: Some(make_injury_result()), ..DropPlayerContext::new() };
        assert!(step.set_parameter(&StepParameter::DropPlayerContext(Box::new(dpc))));
        assert!(step.drop_player_context.is_some());
    }

    #[test]
    fn publishes_injury_result() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn drops_player_when_not_already_dropped() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));

        step.start(&mut game, &mut GameRng::new(0));

        // Player should now be prone
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), ffb_model::enums::PS_PRONE);
    }

    #[test]
    fn does_not_drop_when_already_dropped() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.already_dropped = true;
        step.drop_player_context = Some(dpc);

        step.start(&mut game, &mut GameRng::new(0));

        // Player should still be standing
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn goto_label_when_set() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.label = Some("MY_LABEL".into());
        step.drop_player_context = Some(dpc);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("MY_LABEL"));
    }

    #[test]
    fn publishes_end_turn_when_flagged() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.end_turn = true;
        step.drop_player_context = Some(dpc);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn use_skill_command_adds_skill_use_report() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());

        let mut step = StepHandleDropPlayerContext::new();
        let action = Action::UseSkill { skill_id: SkillId::Pro, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "should have SKILL_USE report after UseSkill command");
    }

    #[test]
    fn use_skill_false_command_also_adds_skill_use_report() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.player_id = Some("p1".into());

        let mut step = StepHandleDropPlayerContext::new();
        let action = Action::UseSkill { skill_id: SkillId::Pro, use_skill: false };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "SKILL_USE report must be added even when skill is not used");
    }

    /// Java: victimStateKey publishes `game.getFieldModel().getPlayerState(game.getDefender())`
    /// — the defender's state, not the acting player's. p1 (acting player) is dropped and becomes
    /// prone; the victim state key must carry p2 (defender)'s still-standing state, not p1's.
    #[test]
    fn victim_state_key_uses_defender_not_acting_player() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        add_player(&mut game, "p2");
        game.acting_player.player_id = Some("p1".into());
        game.defender_id = Some("p2".into());

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.victim_state_key = Some(VictimStateKey::OldDefenderState);
        step.drop_player_context = Some(dpc);

        let out = step.start(&mut game, &mut GameRng::new(0));

        let published_state = out.published.iter().find_map(|p| {
            if let StepParameter::OldDefenderState(s) = p { Some(*s) } else { None }
        }).expect("OldDefenderState should be published");
        let p2_state = game.field_model.player_state("p2").unwrap();
        assert_eq!(published_state, p2_state);
        assert_eq!(published_state.base(), PS_STANDING);
    }

    #[test]
    fn no_drop_when_armour_break_required_but_not_broken() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        // Override: requires armour break but armor_broken = false
        if let Some(ref mut ir) = dpc.injury_result {
            ir.injury_context.armor_broken = false;
        }
        dpc.requires_armour_break = true;
        step.drop_player_context = Some(dpc);

        step.start(&mut game, &mut GameRng::new(0));

        // Should NOT have dropped the player
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    /// Java `StepHandleDropPlayerContext.executeStep` calls `UtilServerInjury.dropPlayer`, whose
    /// ball handling sits OUTSIDE the `placedProneCausesInjuryRoll` if/else and therefore runs for
    /// a Ball & Chain player too. This step used to inline the B&C injury roll and return no
    /// parameters at all, so a dropped Fanatic standing on the ball never published SCATTER_BALL
    /// and the ball did not bounce (goblin bb2020 seed 98).
    #[test]
    fn dropping_a_ball_and_chain_player_on_the_ball_publishes_scatter_ball() {
        let mut game = make_game();
        add_player(&mut game, "bc1");
        {
            let p = game.team_home.players.iter_mut().find(|p| p.id == "bc1").unwrap();
            p.starting_skills.push(ffb_model::model::skill_def::SkillWithValue::new(SkillId::BallAndChain));
        }
        let coord = game.field_model.player_coordinate("bc1").unwrap();
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;

        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("bc1"));
        let mut rng = GameRng::new(7);
        let out = step.start(&mut game, &mut rng);

        assert!(out.published.iter().any(|p| matches!(p,
            StepParameter::CatchScatterThrowInMode(
                crate::step::framework::CatchScatterThrowInMode::ScatterBall))),
            "expected SCATTER_BALL, published: {:?}", out.published);
        assert!(game.field_model.ball_moving);
        // The B&C chain injury is still rolled — that half must not be lost either.
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }
}
