/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBothDown`.
///
/// Handles the Both Down block result (BB2016).  The defender falls unless it has the
/// `preventFallOnBothDown` skill property (i.e. the Block skill), in which case its
/// state is restored to `fOldDefenderState`.  The attacker falls unless it also has
/// that property.  No tacklezones check — that distinction is the BB2020+ version.
///
/// Expects `OLD_DEFENDER_STATE` parameter from a preceding step.
use ffb_model::enums::PS_FALLING;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use ffb_model::enums::PlayerState;

/// Java: `StepBothDown` (bb2016/block).
pub struct StepBothDown {
    /// Java: `fOldDefenderState`
    old_defender_state: Option<PlayerState>,
}

impl StepBothDown {
    pub fn new() -> Self {
        Self { old_defender_state: None }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // ── Defender ─────────────────────────────────────────────────────────
        if let Some(defender_id) = game.defender_id.clone() {
            let has_property = game.player(&defender_id)
                .map(|p| p.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN))
                .unwrap_or(false);

            if !has_property {
                if let Some(state) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, state.change_base(PS_FALLING));
                }
            } else if let Some(old) = self.old_defender_state {
                game.field_model.set_player_state(&defender_id, old);
            }
        }

        // ── Attacker ─────────────────────────────────────────────────────────
        if let Some(attacker_id) = game.acting_player.player_id.clone() {
            let has_property = game.player(&attacker_id)
                .map(|p| p.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN))
                .unwrap_or(false);

            if !has_property {
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
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, state_base: u32) {
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
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    #[test]
    fn id_is_both_down() {
        assert_eq!(StepBothDown::new().id(), StepId::BothDown);
    }

    #[test]
    fn returns_next_step() {
        let mut step = StepBothDown::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn defender_without_property_falls() {
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut step = StepBothDown::new();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // has_skill_property stub returns false → defender should fall
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_FALLING);
    }

    #[test]
    fn attacker_without_property_falls() {
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        game.acting_player.player_id = Some("att".into());
        let mut step = StepBothDown::new();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("att").unwrap();
        assert_eq!(state.base(), PS_FALLING);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBothDown::new();
        let state = PlayerState::new(PS_PRONE);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_defender_state, Some(state));
    }

    #[test]
    fn no_defender_no_panic() {
        let mut game = make_game();
        game.defender_id = None;
        let mut step = StepBothDown::new();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }
}
