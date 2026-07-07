use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rechecks whether a bomb should explode on the Bombardier's own square.
///
/// After the first explode-skill check, if the Bombardier still has an unused
/// `canForceBombExplosion` property, this step waits for the player to choose
/// whether to explode the bomb. Once the choice is received (or if the skill
/// is absent/used), the step proceeds to the next step in the sequence.
///
/// Mirrors Java `StepRecheckExplodeSkill`.
pub struct StepRecheckExplodeSkill {
    /// Java: fUseSkillChoice — set from CLIENT_USE_SKILL command.
    pub use_skill_choice: Option<bool>,
}

impl StepRecheckExplodeSkill {
    pub fn new() -> Self {
        Self { use_skill_choice: None }
    }
}

impl Default for StepRecheckExplodeSkill {
    fn default() -> Self { Self::new() }
}

impl Step for StepRecheckExplodeSkill {
    fn id(&self) -> StepId { StepId::RecheckExplodeSkill }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if let Action::UseSkill { use_skill, .. } = action {
            if self.use_skill_choice.is_none() {
                self.use_skill_choice = Some(*use_skill);
            }
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepRecheckExplodeSkill {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if self.use_skill_choice.is_some() {
            return StepOutcome::next();
        }
        // Java: if actingPlayer.hasUnusedSkillWithProperty(canForceBombExplosion) → show dialog → Continue
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|pid| game.player(pid))
            .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_FORCE_BOMB_EXPLOSION))
            .unwrap_or(false);
        if has_skill {
            return StepOutcome::cont();
        }
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step_when_no_acting_player() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_returns_cont_when_acting_player_has_explode_skill() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::acting_player::ActingPlayer;
        use ffb_model::enums::PlayerAction;

        let mut game = make_game();
        let p = Player {
            id: "b1".into(), name: "b1".into(), nr: 1,
            position_id: "pos".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Kaboom, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p);
        game.acting_player.set_player("b1".into(), PlayerAction::Block);

        // Bombardier skill has canForceBombExplosion property — step should wait for choice
        let mut step = StepRecheckExplodeSkill::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "should wait for skill use choice");
    }

    #[test]
    fn handle_use_skill_stores_choice_and_returns_next() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let action = Action::UseSkill { skill_id: SkillId::Kaboom, use_skill: true };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.use_skill_choice, Some(true));
    }

    #[test]
    fn second_choice_is_ignored() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        step.use_skill_choice = Some(true);
        let action = Action::UseSkill { skill_id: SkillId::Kaboom, use_skill: false };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.use_skill_choice, Some(true));
    }

    #[test]
    fn set_parameter_always_returns_false() {
        let mut step = StepRecheckExplodeSkill::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
