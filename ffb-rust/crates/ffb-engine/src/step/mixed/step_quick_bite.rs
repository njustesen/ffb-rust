/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepQuickBite`.
///
/// Handles the Quick Bite skill (canAttackOpponentForBallAfterCatch) after a catch.
///
/// Java logic (executeStep):
///   - If `use_skill` == None: find adjacent opponents with Quick Bite property.
///     - 0 opponents → skip (next step).
///     - 1 opponent → show DialogSkillUseParameter → CONTINUE.
///     - 2+ opponents → show DialogPlayerChoiceParameter(QUICK_BITE) → CONTINUE.
///   - If `use_skill` == Some(true): run injury (InjuryTypeQuickBite), drop player context.
///     - If armor broken: if touchback → publish TOUCHBACK; else publish PLAYER_ID + REVERT_END_TURN.
///   - Next step always.
///
/// Java fields: `catcherId`, `playerId`, `playerIds`, `useSkill`.
///
/// Java: `StepQuickBite extends AbstractStep` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepQuickBite` (mixed, BB2020 + BB2025).
pub struct StepQuickBite {
    /// Java: `catcherId` (CATCHER_ID parameter — consumed on receipt)
    pub catcher_id: Option<String>,
    /// Java: `playerId` — the Quick Bite attacker chosen from the dialog
    pub player_id: Option<String>,
    /// Java: `playerIds` — candidates with unused Quick Bite skill
    pub player_ids: Vec<String>,
    /// Java: `useSkill` — None until dialog answered
    pub use_skill: Option<bool>,
}

impl StepQuickBite {
    pub fn new() -> Self {
        Self {
            catcher_id: None,
            player_id: None,
            player_ids: Vec::new(),
            use_skill: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let catcher_id = match &self.catcher_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        if self.use_skill.is_none() {
            // Java: Player<?>[] opponents = UtilPlayer.findAdjacentOpposingPlayersWithProperty(
            //   game, catcher, game.getFieldModel().getBallCoordinate(),
            //   NamedProperties.canAttackOpponentForBallAfterCatch, false, true)
            let ball_coord = match game.field_model.ball_coordinate {
                Some(c) => c,
                None => return StepOutcome::next(),
            };
            let opponents: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_property_ext(
                game,
                &catcher_id,
                ball_coord,
                NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH,
                false,
                true, // requireUnusedSkill
            ).into_iter().cloned().collect();

            if opponents.is_empty() {
                return StepOutcome::next();
            }

            self.player_ids.extend(opponents);

            // Java: Player<?> firstOpponent = opponents[0];
            //       UtilCards.getUnusedSkillWithProperty(firstOpponent, canAttackOpponentForBallAfterCatch)
            //           .ifPresent(skill -> { ...showDialog...; setNextAction(CONTINUE); });
            //       return;
            // When the first opponent has no UNUSED matching skill Java shows no dialog and leaves
            // nextAction unset; `findAdjacentOpposingPlayersWithProperty(.., requireUnusedSkill=true)`
            // makes that unreachable, so the miss falls through to NEXT_STEP here.
            let first_opponent = self.player_ids[0].clone();
            let skill = game.player(&first_opponent).and_then(|p| {
                UtilCards::get_unused_skill_with_property(
                    p,
                    NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH,
                )
            });
            if let Some(skill) = skill {
                if self.player_ids.len() == 1 {
                    // Java: DialogSkillUseParameter(firstOpponent.getId(), skill, 0)
                    return StepOutcome::cont().with_prompt(
                        ffb_model::prompts::AgentPrompt::SkillUse {
                            player_id: first_opponent,
                            skill_id: skill as u16,
                            skill_name: skill.class_name().to_string(),
                        },
                    );
                }
                // Java: DialogPlayerChoiceParameter(firstOpponent.getTeam().getId(),
                //       PlayerChoiceMode.QUICK_BITE, opponents, null, 1)
                return StepOutcome::cont().with_prompt(
                    ffb_model::prompts::AgentPrompt::PlayerChoice {
                        eligible_players: self.player_ids.clone(),
                        reason: "QUICK_BITE".into(),
                        descriptions: Vec::new(),
                    },
                );
            }
            return StepOutcome::next();
        } else if self.use_skill == Some(true) {
            // Java: Skill skill = player.getSkillWithProperty(NamedProperties.canAttackOpponentForBallAfterCatch)
            //       getResult().addReport(new ReportSkillUse(playerId, skill, true, SkillUse.QUICK_BITE))
            //       player.markUsed(skill, game)
            if let Some(pid) = &self.player_id {
                let skill = game.player(pid)
                    .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH));
                game.report_list.add(ReportSkillUse::new(Some(pid.clone()), skill.unwrap_or(ffb_model::enums::SkillId::QuickBite), true, SkillUse::QUICK_BITE));
                game.mark_skill_used(pid, ffb_model::enums::SkillId::QuickBite);
            }

            if let Some(ref player_id) = self.player_id.clone() {
                let catcher_coord = game.field_model.player_coordinate(&catcher_id)
                    .unwrap_or(ffb_model::types::FieldCoordinate::new(5, 5));
                let injury_result = crate::step::util_server_injury::handle_injury_by_name(
                    game,
                    rng,
                    "quickBite",
                    Some(player_id),
                    &catcher_id,
                    catcher_coord,
                    None,
                    None,
                    ffb_model::enums::ApothecaryMode::QuickBite,
                );
                let is_armor_broken = injury_result.injury_context.armor_broken;
                // Java: new DropPlayerContext(injuryResultDefender, false, false, null,
                //           catcherId, ApothecaryMode.QUICK_BITE, true)
                // — the 7-arg overload (injuryResult, endTurn, eligibleForSafePairOfHands, label,
                // playerId, apothecaryMode, requiresArmourBreak); alreadyDropped defaults to false.
                // `with_injury`'s 4th parameter is eligibleForSafePairOfHands, so passing `true`
                // there set the WRONG flag and left requiresArmourBreak false — the drop step then
                // knocked the catcher down even on an unbroken armour roll (nurgle bb2025 seed 32
                // i=5: Java a00 Standing, Rust a00 Prone).
                let drop_ctx = crate::drop_player_context::DropPlayerContext {
                    requires_armour_break: true,
                    ..crate::drop_player_context::DropPlayerContext::with_injury(
                        injury_result,
                        catcher_id.clone(),
                        ffb_model::enums::ApothecaryMode::QuickBite,
                        false,
                    )
                };
                let mut outcome = StepOutcome::next()
                    .publish(StepParameter::DropPlayerContext(Box::new(drop_ctx)));

                if is_armor_broken {
                    // Java: game.getFieldModel().setBallCoordinate(getPlayerCoordinate(player))
                    if let Some(player_coord) = game.field_model.player_coordinate(player_id) {
                        game.field_model.ball_coordinate = Some(player_coord);
                    }
                    // Java: bounds = isHomePlaying() ? HALF_AWAY : HALF_HOME;
                    //       if (turnMode == KICKOFF && !bounds.isInBounds(ballCoordinate))
                    //           publish TOUCHBACK(true)
                    //       else { publish PLAYER_ID; if (player.getTeam() == actingTeam) REVERT_END_TURN }
                    let bounds = if game.home_playing {
                        ffb_model::types::FieldCoordinateBounds::HALF_AWAY
                    } else {
                        ffb_model::types::FieldCoordinateBounds::HALF_HOME
                    };
                    let out_of_half = game.field_model.ball_coordinate
                        .map(|c| !bounds.is_in_bounds(c))
                        .unwrap_or(false);
                    if game.turn_mode == ffb_model::enums::TurnMode::Kickoff && out_of_half {
                        outcome = outcome.publish(StepParameter::Touchback(true));
                    } else {
                        // Java: publish PLAYER_ID, then REVERT_END_TURN only when the attacker is on
                        // the acting team (prevents the turnover for passes/hand-offs only).
                        outcome = outcome.publish(StepParameter::PlayerId(player_id.clone()));
                        if game.is_active_team_player(player_id) {
                            outcome = outcome.publish(StepParameter::RevertEndTurn(true));
                        }
                    }
                }
                return outcome;
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepQuickBite {
    fn default() -> Self { Self::new() }
}

impl Step for StepQuickBite {
    fn id(&self) -> StepId { StepId::QuickBite }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE — array of selected player IDs
            Action::PlayerChoice { player_ids, .. } => {
                if let Some(pid) = player_ids.first() {
                    if self.player_ids.contains(pid) {
                        self.player_id = Some(pid.clone());
                        self.use_skill = Some(true);
                    } else {
                        self.use_skill = Some(false);
                    }
                } else {
                    self.use_skill = Some(false);
                }
            }
            // Java: CLIENT_USE_SKILL — playerIds.contains(commandUseSkill.getPlayerId())
            //   && commandUseSkill.getSkill().hasSkillProperty(canAttackOpponentForBallAfterCatch)
            // The only candidate offered via the single-opponent skill-use dialog is playerIds[0],
            // so a property-matching command implicitly refers to that player (Action::UseSkill
            // carries no explicit player id field).
            Action::UseSkill { skill_id, use_skill } => {
                if skill_id.properties().contains(&NamedProperties::CAN_ATTACK_OPPONENT_FOR_BALL_AFTER_CATCH) {
                    if let Some(pid) = self.player_ids.first() {
                        self.player_id = Some(pid.clone());
                    }
                    self.use_skill = Some(*use_skill);
                } else {
                    // Java: falls through to `default:` → UNHANDLED_COMMAND (no executeStep, the
                    // dialog stays up). Rust has no "keep waiting" outcome that is not a
                    // prompt-less Continue (the driver's stall shape), so this mirrors the
                    // established StepPickMeUp convention and leaves the step instead.
                    return StepOutcome::next();
                }
            }
            // Java: CLIENT_PLAYER_CHOICE with an empty (declined) selection —
            //   `ArrayTool.isProvided(selected) && playerIds.contains(selected[0])` is false
            //   → useSkill = false → EXECUTE_STEP. The parity agents answer a declined
            //   PlayerChoice with `Action::SelectPlayer { player_id: "" }` (ParityRunner's
            //   PLAYER_CHOICE default sends an empty selection for QUICK_BITE), so without this
            //   arm the dialog would refire forever.
            Action::SelectPlayer { player_id } => {
                if !player_id.is_empty() && self.player_ids.contains(player_id) {
                    self.player_id = Some(player_id.clone());
                    self.use_skill = Some(true);
                } else {
                    self.use_skill = Some(false);
                }
            }
            // Java: `default:` → UNHANDLED_COMMAND, executeStep() is NOT called (see above).
            _ => return StepOutcome::next(),
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: CATCHER_ID — consumed immediately
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::CatcherId(_))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::prompts::AgentPrompt;
    use ffb_model::enums::PS_STANDING;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_mechanics::skills::SkillId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepQuickBite::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepQuickBite::new();
        let accepted = step.set_parameter(&StepParameter::CatcherId(Some("catcher".into())));
        assert!(accepted);
        assert_eq!(step.catcher_id, Some("catcher".into()));
    }

    #[test]
    fn handle_use_skill_true_sets_use_skill() {
        // Java: CLIENT_USE_SKILL only matches when playerIds.contains(getPlayerId())
        // && getSkill().hasSkillProperty(canAttackOpponentForBallAfterCatch); the lone
        // dialog candidate is playerIds[0], so player_id must be populated from it too.
        let mut step = StepQuickBite::new();
        step.player_ids.push("opp".into());
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::QuickBite, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(true));
        assert_eq!(step.player_id, Some("opp".into()));
    }

    #[test]
    fn handle_use_skill_false_sets_use_skill() {
        let mut step = StepQuickBite::new();
        step.player_ids.push("opp".into());
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::QuickBite, use_skill: false };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(false));
    }

    #[test]
    fn handle_use_skill_ignores_wrong_skill_property() {
        // Java: commandUseSkill.getSkill().hasSkillProperty(canAttackOpponentForBallAfterCatch)
        // guards the branch — a command carrying an unrelated skill must fall through unhandled.
        let mut step = StepQuickBite::new();
        step.player_ids.push("opp".into());
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.use_skill, None);
        assert_eq!(step.player_id, None);
    }

    #[test]
    fn quick_bite_report_added_when_use_skill_true() {
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_id = Some("qb_player".into());
        step.use_skill = Some(true);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn no_quick_bite_report_when_use_skill_none() {
        let mut step = StepQuickBite::new();
        // use_skill stays None → opponent-search path → no report
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    fn add_low_armour_player(game: &mut Game, on_home_team: bool, id: &str) {
        let player = ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 2,
            ..Default::default()
        };
        if on_home_team { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
    }

    #[test]
    fn revert_end_turn_published_when_attacker_on_acting_team() {
        // Java: `if (player.getTeam() == game.getActingTeam())` publish REVERT_END_TURN(true)
        // only when the Quick Bite attacker (`playerId`) belongs to the acting team.
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_id = Some("attacker".into());
        step.use_skill = Some(true);
        let mut game = make_game();
        // game.home_playing defaults to true, so the home team is the acting team.
        assert!(game.home_playing);
        add_low_armour_player(&mut game, true, "attacker");
        let mut rng = GameRng::new(1); // seed known to break armour(2) per InjuryTypeQuickBite tests
        let out = step.start(&mut game, &mut rng);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::RevertEndTurn(true))),
            "expected REVERT_END_TURN(true) when attacker is on the acting team");
    }

    // -- Offer / dialog regression tests --------------------------------------
    //
    // Java `StepQuickBite.executeStep` shows a dialog and CONTINUEs when adjacent opponents
    // with an unused Quick Bite skill are found. Rust used to `return StepOutcome::next()`
    // behind a "client-only" comment, so the offer never happened and the mechanic was dead in
    // both parity agents (nurgle bb2025 seed 32 i=4: Java spent 2 extra dice on Guffle
    // Pusmaw's Quick Bite armour roll that Rust never rolled).

    /// Puts `catcher` on the away team holding the ball, and `n` home-team biters adjacent to
    /// it, each with an unused Quick Bite skill.
    fn make_quick_bite_game(n: usize) -> Game {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let ball = FieldCoordinate::new(13, 7);

        let catcher = ffb_model::model::player::Player {
            id: "catcher".into(), name: "catcher".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 10,
            ..Default::default()
        };
        game.team_away.players.push(catcher);
        game.field_model.set_player_coordinate("catcher", ball);
        game.field_model.set_player_state("catcher", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.field_model.ball_coordinate = Some(ball);

        let spots = [FieldCoordinate::new(12, 6), FieldCoordinate::new(12, 7),
                     FieldCoordinate::new(12, 8), FieldCoordinate::new(13, 6)];
        for i in 0..n {
            let id = format!("biter{i}");
            let biter = ffb_model::model::player::Player {
                id: id.clone(), name: id.clone(), nr: (i as i32) + 1, position_id: "star".into(),
                movement: 5, strength: 4, agility: 4, passing: 6, armour: 10,
                starting_skills: vec![SkillWithValue::new(SkillId::QuickBite)],
                ..Default::default()
            };
            game.team_home.players.push(biter);
            game.field_model.set_player_coordinate(&id, spots[i]);
            game.field_model.set_player_state(&id, ffb_model::enums::PlayerState::new(PS_STANDING));
        }
        game
    }

    #[test]
    fn single_opponent_offers_skill_use_prompt() {
        // Java: opponents.length == 1 -> DialogSkillUseParameter(firstOpponent.getId(), skill, 0)
        //       + setNextAction(CONTINUE).
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        let mut game = make_quick_bite_game(1);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue, "Java CONTINUEs on the dialog");
        match out.prompt {
            Some(AgentPrompt::SkillUse { ref player_id, ref skill_name, .. }) => {
                assert_eq!(player_id, "biter0");
                // ParityRunner dispatches on getClass().getSimpleName(); the Rust agents match on
                // this exact string, so a rename here silently kills the offer.
                assert_eq!(skill_name, "QuickBite");
            }
            ref other => panic!("expected a SkillUse(QuickBite) prompt, got {other:?}"),
        }
        assert_eq!(step.player_ids, vec!["biter0".to_string()]);
    }

    #[test]
    fn two_opponents_offer_quick_bite_player_choice_prompt() {
        // Java: opponents.length > 1 -> DialogPlayerChoiceParameter(team, QUICK_BITE, opponents,
        //       null, 1) + setNextAction(CONTINUE).
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        let mut game = make_quick_bite_game(2);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
        match out.prompt {
            Some(AgentPrompt::PlayerChoice { ref eligible_players, ref reason, .. }) => {
                assert_eq!(reason, "QUICK_BITE");
                assert_eq!(eligible_players.len(), 2);
            }
            ref other => panic!("expected a QUICK_BITE PlayerChoice prompt, got {other:?}"),
        }
    }

    #[test]
    fn no_prompt_when_no_adjacent_opponent_has_quick_bite() {
        // Java: opponents empty -> falls through to setNextAction(NEXT_STEP), no dialog.
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        let mut game = make_quick_bite_game(0);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.prompt.is_none());
    }

    #[test]
    fn declined_player_choice_sets_use_skill_false() {
        // Java: CLIENT_PLAYER_CHOICE with an empty selection -> useSkill = false -> NEXT_STEP.
        // The parity agents answer a declined PlayerChoice with Action::SelectPlayer{""}; without
        // an arm for it the step would re-run executeStep and re-prompt forever.
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_ids = vec!["biter0".into(), "biter1".into()];
        let mut game = make_quick_bite_game(2);
        let mut rng = GameRng::new(0);
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: String::new() }, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(false));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.prompt.is_none(), "a declined choice must not re-prompt");
    }

    #[test]
    fn selected_player_choice_uses_the_named_biter() {
        // Java: ArrayTool.isProvided(selected) && playerIds.contains(selected[0])
        //       -> playerId = selected[0]; useSkill = true.
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_ids = vec!["biter0".into(), "biter1".into()];
        let mut game = make_quick_bite_game(2);
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::SelectPlayer { player_id: "biter1".into() }, &mut game, &mut rng);
        assert_eq!(step.use_skill, Some(true));
        assert_eq!(step.player_id, Some("biter1".into()));
    }

    #[test]
    fn drop_player_context_requires_armour_break_and_is_not_safe_pair_of_hands() {
        // Java: new DropPlayerContext(injuryResult, false, false, null, catcherId,
        //       ApothecaryMode.QUICK_BITE, true) -- the 7-arg overload, whose LAST argument is
        //       requiresArmourBreak. Rust called the `with_injury` shorthand whose 4th argument is
        //       eligibleForSafePairOfHands, so it set the wrong flag: requiresArmourBreak stayed
        //       false and the drop step knocked the catcher down on an UNBROKEN armour roll
        //       (nurgle bb2025 seed 32 i=5: Java a00 Standing, Rust a00 Prone).
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_id = Some("biter0".into());
        step.use_skill = Some(true);
        let mut game = make_quick_bite_game(1);
        let mut rng = GameRng::new(1);
        let out = step.start(&mut game, &mut rng);
        let ctx = out.published.iter().find_map(|p| match p {
            StepParameter::DropPlayerContext(c) => Some(c),
            _ => None,
        }).expect("expected a DROP_PLAYER_CONTEXT parameter");
        assert!(ctx.requires_armour_break, "Java passes requiresArmourBreak = true");
        assert!(!ctx.eligible_for_safe_pair_of_hands, "Java passes eligibleForSafePairOfHands = false");
        assert!(!ctx.already_dropped, "the 7-arg overload leaves alreadyDropped = false");
        assert!(!ctx.end_turn, "Java passes endTurn = false");
    }

    #[test]
    fn revert_end_turn_not_published_when_attacker_off_acting_team() {
        let mut step = StepQuickBite::new();
        step.catcher_id = Some("catcher".into());
        step.player_id = Some("attacker".into());
        step.use_skill = Some(true);
        let mut game = make_game();
        assert!(game.home_playing);
        // attacker on the away (non-acting) team this time.
        add_low_armour_player(&mut game, false, "attacker");
        let mut rng = GameRng::new(1);
        let out = step.start(&mut game, &mut rng);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::RevertEndTurn(_))),
            "REVERT_END_TURN must not be published when the attacker is off the acting team");
    }
}
