/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockChainsaw`.
///
/// Step in block sequence to handle skill CHAINSAW.
///
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_FAILURE.
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_SUCCESS.
///
/// Sets stepParameter END_TURN for all steps on the stack.
/// Sets stepParameter INJURY_RESULT for all steps on the stack.
use ffb_model::enums::{ApothecaryMode, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use crate::action::Action;
use crate::injury::injuryType::injury_type_chainsaw::InjuryTypeChainsaw;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::util_server_injury::drop_player;

/// Java: `StepBlockChainsaw` (bb2016/block).
pub struct StepBlockChainsaw {
    /// Java: `fGotoLabelOnSuccess` — init parameter (mandatory).
    pub goto_label_on_success: String,
    /// Java: `fGotoLabelOnFailure` — init parameter (mandatory).
    pub goto_label_on_failure: String,
    /// AbstractStepWithReRoll embedded state.
    pub re_roll: ReRollState,
}

impl StepBlockChainsaw {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            goto_label_on_failure: String::new(),
            re_roll: ReRollState::new(),
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (actingPlayer.getPlayer().hasSkillProperty(NamedProperties.blocksLikeChainsaw))
        let attacker_has_chainsaw = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
            .unwrap_or(false);

        if !attacker_has_chainsaw {
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            return StepOutcome::next();
        }

        let mut drop_chainsaw_player = false;
        let mut pending_event: Option<GameEvent> = None;

        // Java: if (ReRolledActions.CHAINSAW == getReRolledAction()) {
        //         if ((getReRollSource() == null) || !UtilServerReRoll.useReRoll(...))
        //           dropChainsawPlayer = true; }
        let is_chainsaw_reroll = self.re_roll.re_rolled_action.as_ref()
            .map(|a| a.name == "CHAINSAW")
            .unwrap_or(false);

        if is_chainsaw_reroll {
            if let Some(ref source) = self.re_roll.re_roll_source.clone() {
                if !use_reroll(game, source, &acting_id) {
                    drop_chainsaw_player = true;
                }
            } else {
                drop_chainsaw_player = true;
            }
        }

        if !drop_chainsaw_player {
            // Java: boolean reRolled = ((getReRolledAction() == CHAINSAW) && (getReRollSource() != null))
            let re_rolled = is_chainsaw_reroll && self.re_roll.re_roll_source.is_some();
            // Java: if (!reRolled) getResult().setSound(SoundId.CHAINSAW) — not ported

            // Java: int roll = getGameState().getDiceRoller().rollChainsaw()  (rolls d8)
            let roll = rng.d8();
            // Java: DiceInterpreter.getInstance().minimumRollChainsaw() → 4
            let minimum_roll = 4;
            let successful = roll >= minimum_roll;

            // Java: getResult().addReport(new ReportChainsawRoll(actingPlayer.getPlayerId(), successful, roll, minimumRoll, reRolled, null))
            {
                use ffb_model::report::report_chainsaw_roll::ReportChainsawRoll;
                game.report_list.add(ReportChainsawRoll::new(
                    Some(acting_id.clone()),
                    successful,
                    roll,
                    minimum_roll,
                    re_rolled,
                    vec![],
                    game.defender_id.clone(),
                ));
            }
            let chainsaw_event = GameEvent::ChainsawRoll {
                player_id: acting_id.clone(),
                roll,
                minimum_roll,
                success: successful,
                rerolled: re_rolled,
            };

            if successful {
                let defender_id = game.defender_id.clone().unwrap_or_default();
                let defender_coord = game.field_model.player_coordinate(&defender_id)
                    .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

                // Java: InjuryResult injuryResultDefender = UtilServerInjury.handleInjury(
                //         this, new InjuryTypeChainsaw(), actingPlayer.getPlayer(), game.getDefender(),
                //         defenderCoordinate, null, null, ApothecaryMode.DEFENDER)
                let mut injury_type = InjuryTypeChainsaw::new();
                let injury_result = crate::step::util_server_injury::handle_injury(
                    game, rng, &mut injury_type,
                    Some(&acting_id), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                );

                let mut outcome = StepOutcome::goto(&self.goto_label_on_success)
                    .with_event(chainsaw_event);

                // Java: if (injuryResultDefender.injuryContext().isArmorBroken()) {
                //         publishParameters(UtilServerInjury.dropPlayer(this, game.getDefender(), DEFENDER)); }
                if injury_result.injury_context().armor_broken {
                    for p in drop_player(game, &defender_id, true) { outcome = outcome.publish(p); }
                }
                // Java: publishParameter(new StepParameter(INJURY_RESULT, injuryResultDefender))
                // Java: getResult().setNextAction(StepAction.GOTO_LABEL, fGotoLabelOnSuccess)
                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
                return outcome;
            } else {
                // Java: if (!UtilServerReRoll.askForReRollIfAvailable(...)) dropChainsawPlayer = true
                if let Some(prompt) = ask_for_reroll_if_available(game, "CHAINSAW", minimum_roll, false) {
                    self.re_roll.set_re_rolled_action(ReRolledAction::new("CHAINSAW"));
                    self.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
                    return StepOutcome::cont().with_event(chainsaw_event).with_prompt(prompt);
                } else {
                    pending_event = Some(chainsaw_event);
                    drop_chainsaw_player = true;
                }
            }
        }

        if drop_chainsaw_player {
            let attacker_coord = game.field_model.player_coordinate(&acting_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

            // Java: InjuryResult injuryResultAttacker = UtilServerInjury.handleInjury(
            //         this, new InjuryTypeChainsaw(), null, actingPlayer.getPlayer(),
            //         attackerCoordinate, null, null, ApothecaryMode.ATTACKER)
            let mut injury_type = InjuryTypeChainsaw::new();
            let injury_result = crate::step::util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &acting_id,
                attacker_coord, None, None, ApothecaryMode::Attacker,
            );

            let mut outcome = StepOutcome::goto(&self.goto_label_on_failure);
            if let Some(ev) = pending_event { outcome = outcome.with_event(ev); }

            // Java: if (injuryResultAttacker.injuryContext().isArmorBroken()) {
            //         publishParameters(UtilServerInjury.dropPlayer(this, actingPlayer.getPlayer(), ATTACKER))
            //         publishParameter(new StepParameter(END_TURN, true)) }
            if injury_result.injury_context().armor_broken {
                for p in drop_player(game, &acting_id, false) { outcome = outcome.publish(p); }
                outcome = outcome.publish(StepParameter::EndTurn(true));
            }
            // Java: publishParameter(new StepParameter(INJURY_RESULT, injuryResultAttacker))
            // Java: getResult().setNextAction(StepAction.GOTO_LABEL, fGotoLabelOnFailure)
            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
            return outcome;
        }

        StepOutcome::cont()
    }
}

impl Default for StepBlockChainsaw {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockChainsaw {
    fn id(&self) -> StepId { StepId::BlockChainsaw }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: StepCommandStatus commandStatus = super.handleCommand(pReceivedCommand)
        // Java: if (commandStatus == EXECUTE_STEP) executeStep()
        match action {
            Action::UseReRoll { use_reroll: false } => {
                // Java: declining re-roll → source cleared → next executeStep sets dropChainsawPlayer
                self.re_roll.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(s) => { self.goto_label_on_success = s.clone(); true }
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "special".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_block_chainsaw() {
        assert_eq!(StepBlockChainsaw::new().id(), StepId::BlockChainsaw);
    }

    #[test]
    fn non_chainsaw_player_returns_next() {
        // Java: else { getResult().setNextAction(StepAction.NEXT_STEP) }
        let mut step = StepBlockChainsaw::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into()));
        step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into()));
        let mut game = make_game();
        add_player(&mut game, "att");
        game.acting_player.player_id = Some("att".into());
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_labels_accepted() {
        let mut step = StepBlockChainsaw::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("s".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("f".into())));
        assert_eq!(step.goto_label_on_success, "s");
        assert_eq!(step.goto_label_on_failure, "f");
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepBlockChainsaw::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn chainsaw_player_produces_goto_or_cont_outcome() {
        // Chainsaw player rolls: either GOTO (success/failure label) or CONT (re-roll offered)
        let mut step = StepBlockChainsaw::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into()));
        step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "saw", SkillId::Chainsaw);
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("saw".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(42);
        let out = step.start(&mut game, &mut rng);
        assert!(
            out.action == StepAction::GotoLabel || out.action == StepAction::Continue,
            "unexpected action: {:?}", out.action
        );
    }

    #[test]
    fn chainsaw_emits_report_chainsaw_roll() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepBlockChainsaw::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into()));
        step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "saw", SkillId::Chainsaw);
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("saw".into());
        game.defender_id = Some("def".into());
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.report_list.has_report(ReportId::CHAINSAW_ROLL), "ReportChainsawRoll must be emitted");
    }

    #[test]
    fn declined_reroll_drops_chainsaw_player() {
        // When re-roll source is cleared via UseReRoll(false), attacker is injured
        let mut step = StepBlockChainsaw::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into()));
        step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into()));
        // Set up as if re-roll was offered after a miss
        step.re_roll.set_re_rolled_action(ReRolledAction::new("CHAINSAW"));
        step.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
        let mut game = make_game();
        add_player_with_skill(&mut game, "saw", SkillId::Chainsaw);
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("saw".into());
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        // Decline the re-roll — this clears re_roll_source and sets dropChainsawPlayer
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut rng,
        );
        // Should go to failure label (attacker injured)
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }
}
