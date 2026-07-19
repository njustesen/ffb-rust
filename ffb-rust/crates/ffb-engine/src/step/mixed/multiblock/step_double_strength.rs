/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepDoubleStrength`.
///
/// If any Dauntless-succeeded targets exist and the acting player has an unused
/// Indomitable skill, prompts for a skill-use choice. On confirmation, publishes
/// `DoubleTargetStrengthForPlayer` for the chosen target.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_indomitable::ReportIndomitable;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepDoubleStrength` (mixed/multiblock, BB2020 + BB2025).
pub struct StepDoubleStrength {
    /// Java: `playerIds` — IDs of players for whom Dauntless succeeded
    player_ids: Vec<String>,
}

impl StepDoubleStrength {
    pub fn new() -> Self { Self { player_ids: Vec::new() } }

    /// Java: `UtilCards.getUnusedSkillWithProperty(actingPlayer, canDoubleStrengthAfterDauntless)`
    fn has_unused_indomitable(game: &Game) -> bool {
        let acting_id = match game.acting_player.player_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        game.player(acting_id)
            .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_DOUBLE_STRENGTH_AFTER_DAUNTLESS))
            .unwrap_or(false)
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if self.player_ids.is_empty() || !Self::has_unused_indomitable(game) {
            return StepOutcome::next();
        }
        // Java: show UseSkill or PlayerChoice dialog → CONTINUE
        // The command handler resolves by publishing DoubleTargetStrengthForPlayer
        StepOutcome::cont()
    }
}

impl Default for StepDoubleStrength {
    fn default() -> Self { Self::new() }
}

impl Step for StepDoubleStrength {
    fn id(&self) -> StepId { StepId::DoubleStrength }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: playerIds.size() == 1 → DialogSkillUseParameter → CLIENT_USE_SKILL (Indomitable),
        // always targets the sole entry in playerIds.
        // Java: playerIds.size() > 1 → DialogPlayerChoiceParameter(INDOMITABLE) →
        // CLIENT_PLAYER_CHOICE, the coach picks which Dauntless-successful target to double.
        // Java: `command.getSkill().hasSkillProperty(canDoubleStrengthAfterDauntless)` — property
        // based, not tied to the specific `Indomitable` skill id.
        let chosen: Option<String> = match action {
            Action::UseSkill { skill_id, use_skill }
                if *use_skill
                    && skill_id.properties().contains(&NamedProperties::CAN_DOUBLE_STRENGTH_AFTER_DAUNTLESS) =>
            {
                self.player_ids.first().cloned()
            }
            Action::IndomitableChoice { player_id } if self.player_ids.contains(player_id) => {
                Some(player_id.clone())
            }
            _ => None,
        };
        if let Some(target_id) = chosen {
            let actor_id = game.acting_player.player_id.clone();
            // Java: actingPlayer.markSkillUsed(NamedProperties.canDoubleStrengthAfterDauntless)
            // — marks whichever skill instance actually grants the property, not literally
            // the `Indomitable` skill id by name.
            if let Some(ref aid) = actor_id {
                if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == *aid)
                    .or_else(|| game.team_away.players.iter_mut().find(|p| p.id == *aid))
                {
                    if let Some(skill) = p.skill_id_with_property(NamedProperties::CAN_DOUBLE_STRENGTH_AFTER_DAUNTLESS) {
                        p.used_skills.insert(skill);
                    }
                }
            }
            // Java: getResult().addReport(new ReportIndomitable(actingPlayer.getPlayerId(), playerIds.get(0)))
            game.report_list.add(ReportIndomitable::new(
                actor_id,
                Some(target_id.clone()),
            ));
            self.player_ids.clear();
            return StepOutcome::next()
                .publish(StepParameter::DoubleTargetStrengthForPlayer(target_id));
        }
        // UseSkill with use_skill=false or any other command → skip
        self.player_ids.clear();
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerIdDauntlessSuccess(id) => {
                self.player_ids.push(id.clone());
                true
            }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerAction, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_skill(game: &mut Game, id: &str, skill: SkillId) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(skill)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_double_strength() {
        assert_eq!(StepDoubleStrength::new().id(), StepId::DoubleStrength);
    }

    #[test]
    fn no_targets_next_step_immediately() {
        let mut step = StepDoubleStrength::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn no_indomitable_next_step() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Block); // no Indomitable
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn with_indomitable_and_target_prompts() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::Continue));
    }

    #[test]
    fn use_indomitable_publishes_double_target() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let outcome = step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Indomitable, use_skill: true },
            &mut game, &mut rng
        );
        assert!(matches!(outcome.action, StepAction::NextStep));
        let has_double = outcome.published.iter().any(|p| {
            matches!(p, StepParameter::DoubleTargetStrengthForPlayer(id) if id == "tgt")
        });
        assert!(has_double);
    }

    #[test]
    fn indomitable_report_added_when_skill_used() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Indomitable, use_skill: true },
            &mut game, &mut rng,
        );
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::INDOMITABLE));
    }

    #[test]
    fn multi_target_choice_picks_chosen_player_not_first() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt1".into()));
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt2".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let outcome = step.handle_command(
            &Action::IndomitableChoice { player_id: "tgt2".into() },
            &mut game, &mut rng,
        );
        assert!(outcome.published.iter().any(|p| {
            matches!(p, StepParameter::DoubleTargetStrengthForPlayer(id) if id == "tgt2")
        }), "should double the chosen target (tgt2), not the first (tgt1)");
    }

    #[test]
    fn multi_target_choice_with_unknown_player_id_is_ignored() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt1".into()));
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt2".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let outcome = step.handle_command(
            &Action::IndomitableChoice { player_id: "not_a_target".into() },
            &mut game, &mut rng,
        );
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::DoubleTargetStrengthForPlayer(_))));
    }

    #[test]
    fn no_indomitable_report_when_skill_declined() {
        let mut step = StepDoubleStrength::new();
        step.set_parameter(&StepParameter::PlayerIdDauntlessSuccess("tgt".into()));
        let mut game = make_game();
        add_player_with_skill(&mut game, "att", SkillId::Indomitable);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Indomitable, use_skill: false },
            &mut game, &mut rng,
        );
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::INDOMITABLE));
    }
}
