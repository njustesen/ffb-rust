/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.block.StepBothDown`.
///
/// Handles the Both Down block result.  The defender falls unless it has the Block skill
/// AND still has tackle zones (not confused/hypnotised/etc.).  The attacker falls unless
/// it has the Block skill.
///
/// Expects `OLD_DEFENDER_STATE` parameter from a preceding step (e.g. StepInitBlocking).
use ffb_model::enums::{PlayerState, SkillId};
use ffb_model::enums::PS_FALLING;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBothDown` (mixed/block, BB2020 + BB2025).
pub struct StepBothDown {
    /// Java: `fOldDefenderState`
    old_defender_state: Option<PlayerState>,
}

impl StepBothDown {
    pub fn new() -> Self {
        Self { old_defender_state: None }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // ── Defender ──────────────────────────────────────────────────────────
        if let Some(defender_id) = game.defender_id.clone() {
            let defender_has_block = game.player(&defender_id)
                .map(|p| p.has_skill(SkillId::Block))
                .unwrap_or(false);
            let defender_has_tacklezones = game.field_model.player_state(&defender_id)
                .map(|s| s.has_tacklezones())
                .unwrap_or(false);

            if !(defender_has_block && defender_has_tacklezones) {
                // Java: defenderState.changeBase(PlayerState.FALLING)
                if let Some(state) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, state.change_base(PS_FALLING));
                }
            } else if let Some(old) = self.old_defender_state {
                game.field_model.set_player_state(&defender_id, old);
            }
        }

        // ── Attacker ──────────────────────────────────────────────────────────
        if let Some(attacker_id) = game.acting_player.player_id.clone() {
            let attacker_has_block = game.player(&attacker_id)
                .map(|p| p.has_skill(SkillId::Block))
                .unwrap_or(false);

            if !attacker_has_block {
                if let Some(state) = game.field_model.player_state(&attacker_id) {
                    game.field_model.set_player_state(&attacker_id, state.change_base(PS_FALLING));
                }
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepBothDown {
    fn default() -> Self { Self::new() }
}

impl Step for StepBothDown {
    fn id(&self) -> StepId { StepId::BothDown }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::OldDefenderState(v) => { self.old_defender_state = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING, PS_FALLING};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32, skills: &[SkillId]) -> FieldCoordinate {
        let pos = FieldCoordinate::new(5, 5);
        let mut p = Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
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
};
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
        pos
    }

    #[test]
    fn id_is_both_down() {
        assert_eq!(StepBothDown::new().id(), StepId::BothDown);
    }

    #[test]
    fn defender_without_block_falls() {
        let mut step = StepBothDown::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        add_player(&mut game, "def", PS_STANDING, &[]);
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.defender_id = Some("def".into());
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::Block);

        step.start(&mut game, &mut rng);

        let defender_state = game.field_model.player_state("def").unwrap();
        assert!(
            defender_state.0 & 0x0000f == PS_FALLING,
            "defender should be falling, got {:#x}", defender_state.0
        );
    }

    #[test]
    fn attacker_without_block_falls() {
        let mut step = StepBothDown::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);

        add_player(&mut game, "def", PS_STANDING, &[SkillId::Block]);
        add_player(&mut game, "att", PS_STANDING, &[]);
        game.defender_id = Some("def".into());
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::Block);

        step.start(&mut game, &mut rng);

        let attacker_state = game.field_model.player_state("att").unwrap();
        assert!(
            attacker_state.0 & 0x0000f == PS_FALLING,
            "attacker should be falling, got {:#x}", attacker_state.0
        );
    }

    #[test]
    fn defender_with_block_and_tacklezones_restores_old_state() {
        let mut step = StepBothDown::new();
        let old_state = PlayerState::new(PS_STANDING);
        step.set_parameter(&StepParameter::OldDefenderState(old_state));

        let mut game = make_game();
        let mut rng = GameRng::new(0);

        add_player(&mut game, "def", PS_STANDING, &[SkillId::Block]);
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.defender_id = Some("def".into());
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::Block);

        // Put defender into some modified state first
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING | 0x10000));

        step.start(&mut game, &mut rng);

        let defender_state = game.field_model.player_state("def").unwrap();
        assert_eq!(defender_state, old_state, "defender should be restored to old state");
    }

    #[test]
    fn both_have_block_neither_falls() {
        let mut step = StepBothDown::new();
        let old_state = PlayerState::new(PS_STANDING);
        step.set_parameter(&StepParameter::OldDefenderState(old_state));

        let mut game = make_game();
        let mut rng = GameRng::new(0);

        add_player(&mut game, "def", PS_STANDING, &[SkillId::Block]);
        add_player(&mut game, "att", PS_STANDING, &[SkillId::Block]);
        game.defender_id = Some("def".into());
        game.acting_player.set_player("att".into(), ffb_model::enums::PlayerAction::Block);

        step.start(&mut game, &mut rng);

        // Attacker should NOT be falling
        let att_state = game.field_model.player_state("att").unwrap();
        assert!(att_state.0 & 0x0000f != PS_FALLING, "attacker with Block should not fall");
    }

    #[test]
    fn no_acting_player_is_noop() {
        let mut step = StepBothDown::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        // No acting player set → should not panic
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
    }
}
