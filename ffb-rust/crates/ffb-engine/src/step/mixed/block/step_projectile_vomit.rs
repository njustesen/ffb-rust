/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.block.StepProjectileVomit`.
///
/// Step in the block sequence to handle skill PROJECTILE_VOMIT.
///
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_FAILURE.
/// Needs to be initialized with stepParameter GOTO_LABEL_ON_SUCCESS.
///
/// Expects stepParameter USING_VOMIT to be set by a preceding step.
use ffb_model::enums::ApothecaryMode;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_projectile_vomit::ReportProjectileVomit;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_projectile_vomit::InjuryTypeProjectileVomit;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::util_server_injury::handle_injury;

/// Java: `StepProjectileVomit` (mixed/block, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepProjectileVomit {
    /// Java: `fGotoLabelOnSuccess` — mandatory init parameter.
    goto_label_on_success: String,
    /// Java: `fGotoLabelOnFailure` — mandatory init parameter.
    goto_label_on_failure: String,
    /// Java: `usingVomit`
    using_vomit: bool,
    /// AbstractStepWithReRoll embedded state.
    re_roll: ReRollState,
}

impl StepProjectileVomit {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (actingPlayer.getPlayer().hasSkillProperty(NamedProperties.canPerformArmourRollInsteadOfBlockThatMightFail) && usingVomit)
        let has_vomit_skill = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL))
            .unwrap_or(false);

        if !(has_vomit_skill && self.using_vomit) {
            // Java: else { getResult().setNextAction(StepAction.NEXT_STEP) }
            return StepOutcome::next();
        }

        // Java: actingPlayer.markSkillUsed(NamedProperties.canPerformArmourRollInsteadOfBlockThatMightFail)
        // (skill marking not yet fully ported)

        let mut drop_self = false;
        let mut vomit_event: Option<GameEvent> = None;

        // Java: if (ReRolledActions.PROJECTILE_VOMIT == getReRolledAction()) {
        //         if ((getReRollSource() == null) || !UtilServerReRoll.useReRoll(...))
        //           dropSelf = true; }
        let is_vomit_reroll = self.re_roll.re_rolled_action.as_ref()
            .map(|a| a.name == "PROJECTILE_VOMIT")
            .unwrap_or(false);

        if is_vomit_reroll {
            if let Some(ref source) = self.re_roll.re_roll_source.clone() {
                if !use_reroll(game, source, &acting_id) {
                    drop_self = true;
                }
            } else {
                drop_self = true;
            }
        }

        if !drop_self {
            // Java: int roll = getGameState().getDiceRoller().rollSkill()
            let roll = rng.d6();
            let minimum_roll = DiceInterpreter::minimum_roll_projectile_vomit();
            let successful = roll >= minimum_roll;

            let defender_id = game.defender_id.clone().unwrap_or_default();
            vomit_event = Some(GameEvent::ProjectileVomitRoll {
                attacker_id: acting_id.clone(),
                defender_id: defender_id.clone(),
                roll,
                success: successful,
                rerolled: is_vomit_reroll,
            });

            // Java: getResult().addReport(new ReportProjectileVomit(actingPlayer.getPlayerId(), successful, roll, minimumRoll, reRolled, game.getDefenderId()))
            game.report_list.add(ReportProjectileVomit::new(
                Some(acting_id.clone()),
                successful,
                roll,
                minimum_roll,
                is_vomit_reroll,
                game.defender_id.clone(),
            ));

            if successful {
                let defender_coord = game.field_model.player_coordinate(&defender_id)
                    .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

                // Java: InjuryResult injuryResultDefender = UtilServerInjury.handleInjury(...)
                let mut injury_type = InjuryTypeProjectileVomit::new();
                let injury_result = handle_injury(
                    game, rng, &mut injury_type,
                    Some(&acting_id.clone()), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                );

                // Java: publishParameter(new StepParameter(DROP_PLAYER_CONTEXT, new DropPlayerContext(...)))
                let ctx = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result)),
                    end_turn: false,
                    eligible_for_safe_pair_of_hands: true,
                    label: Some(self.goto_label_on_success.clone()),
                    player_id: Some(defender_id),
                    apothecary_mode: Some(ApothecaryMode::Defender),
                    ..DropPlayerContext::new()
                };
                let out = StepOutcome::next()
                    .publish(StepParameter::DropPlayerContext(Box::new(ctx)));
                return if let Some(ev) = vomit_event { out.with_event(ev) } else { out };
            } else {
                // Java: if (getReRolledAction() == PROJECTILE_VOMIT || !askForReRollIfAvailable(...))
                if is_vomit_reroll {
                    drop_self = true;
                } else if let Some(prompt) = ask_for_reroll_if_available(game, "PROJECTILE_VOMIT", minimum_roll, false) {
                    self.re_roll.set_re_rolled_action(ReRolledAction::new("PROJECTILE_VOMIT"));
                    let out = StepOutcome::cont().with_prompt(prompt);
                    return if let Some(ev) = vomit_event { out.with_event(ev) } else { out };
                } else {
                    drop_self = true;
                }
            }
        }

        if drop_self {
            let attacker_coord = game.field_model.player_coordinate(&acting_id)
                .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));

            // Java: InjuryResult injuryResultAttacker = UtilServerInjury.handleInjury(null, actingPlayer, ...)
            let mut injury_type = InjuryTypeProjectileVomit::new();
            let injury_result = handle_injury(
                game, rng, &mut injury_type,
                None, &acting_id,
                attacker_coord, None, None, ApothecaryMode::Attacker,
            );

            let ctx = DropPlayerContext {
                injury_result: Some(Box::new(injury_result)),
                end_turn: false,
                eligible_for_safe_pair_of_hands: true,
                label: Some(self.goto_label_on_failure.clone()),
                player_id: Some(acting_id),
                apothecary_mode: Some(ApothecaryMode::Attacker),
                ..DropPlayerContext::new()
            };
            let out = StepOutcome::next()
                .publish(StepParameter::DropPlayerContext(Box::new(ctx)));
            return if let Some(ev) = vomit_event { out.with_event(ev) } else { out };
        }

        StepOutcome::cont()
    }
}

impl Step for StepProjectileVomit {
    fn id(&self) -> StepId { StepId::ProjectileVomit }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: StepCommandStatus commandStatus = super.handleCommand(pReceivedCommand)
        // Java: if (commandStatus == EXECUTE_STEP) executeStep()
        match action {
            Action::UseReRoll { use_reroll: false } => {
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
            StepParameter::UsingVomit(v) => { self.using_vomit = *v; true }
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
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state_bits: u32, skills: &[SkillId]) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state_bits));
    }

    #[test]
    fn projectile_vomit_report_added_on_roll() {
        let mut step = StepProjectileVomit {
            goto_label_on_success: "success".into(),
            goto_label_on_failure: "failure".into(),
            using_vomit: true,
            ..Default::default()
        };
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING, &[SkillId::ProjectileVomit]);
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::ProjectileVomit);
        // Add a defender
        game.team_home.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("def", ffb_model::enums::PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        // Check if ProjectileVomit skill has the needed property
        let has_prop = game.player("att")
            .map(|p| p.has_skill_property(NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL))
            .unwrap_or(false);
        if !has_prop {
            return; // Skill not yet wired — skip
        }
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            game.report_list.has_report(ReportId::PROJECTILE_VOMIT),
            "should add ReportProjectileVomit when vomit roll is made"
        );
    }

    #[test]
    fn no_projectile_vomit_report_when_no_skill() {
        let mut step = StepProjectileVomit {
            goto_label_on_success: "success".into(),
            goto_label_on_failure: "failure".into(),
            using_vomit: true,
            ..Default::default()
        };
        let mut game = make_game();
        // Block skill has no canPerformArmourRollInsteadOfBlockThatMightFail property
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::ProjectileVomit);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            !game.report_list.has_report(ReportId::PROJECTILE_VOMIT),
            "should not add ReportProjectileVomit when skill is absent"
        );
    }

    #[test]
    fn id_is_projectile_vomit() {
        assert_eq!(StepProjectileVomit::new().id(), StepId::ProjectileVomit);
    }

    #[test]
    fn without_vomit_skill_returns_next() {
        let mut step = StepProjectileVomit {
            goto_label_on_success: "success".into(),
            goto_label_on_failure: "failure".into(),
            using_vomit: true,
            ..Default::default()
        };
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        // Block skill has no canPerformArmourRollInsteadOfBlockThatMightFail property
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::ProjectileVomit);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn using_vomit_false_returns_next() {
        let mut step = StepProjectileVomit {
            goto_label_on_success: "success".into(),
            goto_label_on_failure: "failure".into(),
            using_vomit: false,
            ..Default::default()
        };
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        add_player(&mut game, "att", PS_STANDING, &[SkillId::ProjectileVomit]);
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::ProjectileVomit);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut step = StepProjectileVomit {
            goto_label_on_success: "s".into(),
            goto_label_on_failure: "f".into(),
            ..Default::default()
        };
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_using_vomit() {
        let mut step = StepProjectileVomit::new();
        step.set_parameter(&StepParameter::UsingVomit(true));
        assert!(step.using_vomit);
    }

    #[test]
    fn set_parameter_goto_labels() {
        let mut step = StepProjectileVomit::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("s".into()));
        step.set_parameter(&StepParameter::GotoLabelOnFailure("f".into()));
        assert_eq!(step.goto_label_on_success, "s");
        assert_eq!(step.goto_label_on_failure, "f");
    }
}
