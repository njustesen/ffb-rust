/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.foul.StepPileDriver`.
///
/// Asks the player whether to use their Chainsaw skill before fouling.
/// Sets `game.defender_id` and publishes `UsingChainsaw` on exit.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPileDriver` (mixed/foul, BB2020 + BB2025).
///
/// Phase SELECT_CHAINSAW: if the acting player has an unused Chainsaw skill, prompt
/// for dialog (CONTINUE); otherwise go straight to DONE.
/// Phase DONE: commit defender_id and publish UsingChainsaw, then NextStep.
pub struct StepPileDriver {
    /// Java: `targetPlayerId`
    target_player_id: Option<String>,
    /// Java: `gotoLabelEnd`
    goto_label_end: String,
    /// Java: `usingChainsaw`
    using_chainsaw: bool,
    /// Java: `phase`
    phase: Phase,
}

#[derive(PartialEq)]
enum Phase {
    SelectChainsaw,
    Done,
}

impl StepPileDriver {
    pub fn new() -> Self {
        Self {
            target_player_id: None,
            goto_label_end: String::new(),
            using_chainsaw: false,
            phase: Phase::SelectChainsaw,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        match self.phase {
            Phase::SelectChainsaw => {
                if self.target_player_id.is_none() {
                    return StepOutcome::goto(&self.goto_label_end);
                }
                // Java: UtilCards.hasUnusedSkillWithProperty(actingPlayer, blocksLikeChainsaw)
                let acting_id = game.acting_player.player_id.clone();
                let has_unused_chainsaw = acting_id
                    .as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_unused_skill_with_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                    .unwrap_or(false);
                if has_unused_chainsaw {
                    // Java: UtilServerDialog.showDialog → CONTINUE waiting for CLIENT_USE_CHAINSAW
                    StepOutcome::cont()
                } else {
                    self.leave_step(game)
                }
            }
            Phase::Done => self.leave_step(game),
        }
    }

    fn leave_step(&self, game: &mut Game) -> StepOutcome {
        game.defender_id = self.target_player_id.clone();
        StepOutcome::next()
            .publish(StepParameter::UsingChainsaw(self.using_chainsaw))
    }
}

impl Default for StepPileDriver {
    fn default() -> Self { Self::new() }
}

impl Step for StepPileDriver {
    fn id(&self) -> StepId { StepId::PileDriver }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_CHAINSAW command → set usingChainsaw, phase=DONE
        // Rust maps this to Action::UseSkill { skill_id: Chainsaw, use_skill }
        if let Action::UseSkill { skill_id, use_skill } = action {
            if *skill_id == SkillId::Chainsaw {
                self.using_chainsaw = *use_skill;
                self.phase = Phase::Done;
            }
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TargetPlayerId(v) => { self.target_player_id = v.clone(); true }
            StepParameter::GotoLabelOnEnd(v)  => { self.goto_label_end = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerAction, PS_STANDING, SkillId as MSkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_chainsaw(game: &mut Game, id: &str) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue::new(MSkillId::Chainsaw)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        game.acting_player.set_player(id.into(), PlayerAction::Foul);
    }

    #[test]
    fn id_is_pile_driver() {
        assert_eq!(StepPileDriver::new().id(), StepId::PileDriver);
    }

    #[test]
    fn no_target_player_gotos_label() {
        let mut step = StepPileDriver::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
    }

    #[test]
    fn no_chainsaw_skill_leaves_step_immediately() {
        let mut step = StepPileDriver::new();
        step.set_parameter(&StepParameter::TargetPlayerId(Some("def".into())));
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        // Add player without chainsaw
        game.team_home.players.push(Player {
            id: "att".into(), name: "att".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.acting_player.set_player("att".into(), PlayerAction::Foul);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        // Should go to leave_step → NextStep
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.defender_id, Some("def".into()));
        let has_using = outcome.published.iter().any(|p| matches!(p, StepParameter::UsingChainsaw(false)));
        assert!(has_using);
    }

    #[test]
    fn chainsaw_player_prompts() {
        let mut step = StepPileDriver::new();
        step.set_parameter(&StepParameter::TargetPlayerId(Some("def".into())));
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        add_player_with_chainsaw(&mut game, "att");
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::Continue), "should wait for dialog");
    }

    /// Regression test: Java gates on `hasUnusedSkillWithProperty`, so an already-used
    /// Chainsaw must not trigger the dialog prompt — it should leave the step immediately
    /// (as if the player had no chainsaw at all). Before this fix, the Rust check only
    /// tested `has_skill(Chainsaw)` and ignored `used_skills`.
    #[test]
    fn already_used_chainsaw_skips_dialog() {
        let mut step = StepPileDriver::new();
        step.set_parameter(&StepParameter::TargetPlayerId(Some("def".into())));
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        add_player_with_chainsaw(&mut game, "att");
        if let Some(p) = game.team_home.player_mut("att") {
            p.used_skills.insert(MSkillId::Chainsaw);
        }
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep), "an already-used chainsaw must not prompt a dialog");
        assert_eq!(game.defender_id, Some("def".into()));
        let has_using = outcome.published.iter().any(|p| matches!(p, StepParameter::UsingChainsaw(false)));
        assert!(has_using);
    }

    #[test]
    fn use_chainsaw_command_sets_defender_and_publishes() {
        let mut step = StepPileDriver::new();
        step.set_parameter(&StepParameter::TargetPlayerId(Some("def".into())));
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        add_player_with_chainsaw(&mut game, "att");
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // Simulate CLIENT_USE_CHAINSAW with using=true
        let outcome = step.handle_command(&Action::UseSkill { skill_id: MSkillId::Chainsaw, use_skill: true }, &mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.defender_id, Some("def".into()));
        let has_chainsaw = outcome.published.iter().any(|p| matches!(p, StepParameter::UsingChainsaw(true)));
        assert!(has_chainsaw);
    }
}
