/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepDoubleStrength`.
///
/// If any Dauntless-succeeded targets exist and the acting player has an unused
/// Indomitable skill, prompts for a skill-use choice. On confirmation, publishes
/// `DoubleTargetStrengthForPlayer` for the chosen target.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
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

    fn has_unused_indomitable(game: &Game) -> bool {
        let acting_id = match game.acting_player.player_id.as_deref() {
            Some(id) => id,
            None => return false,
        };
        game.player(acting_id)
            .map(|p| p.has_skill(SkillId::Indomitable) && !p.used_skills.contains(&SkillId::Indomitable))
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
        // Java CLIENT_USE_SKILL (Indomitable) → use playerIds[0] if isSkillUsed
        // Java CLIENT_PLAYER_CHOICE (INDOMITABLE) → use chosen player id
        let chosen: Option<String> = match action {
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::Indomitable && *use_skill => {
                self.player_ids.first().cloned()
            }
            _ => None,
        };
        if let Some(target_id) = chosen {
            // Mark skill used
            if let Some(actor_id) = game.acting_player.player_id.clone() {
                if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == actor_id)
                    .or_else(|| game.team_away.players.iter_mut().find(|p| p.id == actor_id))
                {
                    p.used_skills.insert(SkillId::Indomitable);
                }
            }
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
}
