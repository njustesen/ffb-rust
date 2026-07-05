use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::SpecialEffect;
use crate::step::generator::bb2025::special_effect::SpecialEffectParams;

/// Resolves the bomb explosion: clears the bomb, finds adjacent players,
/// and pushes a SpecialEffect sequence for every player in the blast radius.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.special.StepResolveBomb.
pub struct StepResolveBomb {
    /// Java: fCatcherId (init param / set parameter)
    pub catcher_id: Option<String>,
}

impl StepResolveBomb {
    pub fn new() -> Self {
        Self { catcher_id: None }
    }
}

impl Default for StepResolveBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepResolveBomb {
    fn id(&self) -> StepId { StepId::ResolveBomb }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepResolveBomb {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: fBombCoordinate = game.fieldModel.getBombCoordinate()
        let bomb_coordinate = match game.field_model.bomb_coordinate {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: if fCatcherId is provided → bomb was caught, no explosion
        if self.catcher_id.is_some() {
            return StepOutcome::next();
        }

        // Java: fieldModel.setBombMoving(false); fieldModel.setBombCoordinate(null)
        game.field_model.bomb_moving = false;
        game.field_model.bomb_coordinate = None;
        // client-only: setAnimation(AnimationType.BOMB_EXPLOSION, fBombCoordinate)
        // client-only: syncGameModel, fieldModel.add(BloodSpot) — client-side display

        // Java: targetCoordinates = fieldModel.findAdjacentCoordinates(fBombCoordinate, FIELD, 1, true)
        // This includes the bomb square itself (true = include center).
        let mut target_coords: Vec<FieldCoordinate> = game.field_model.adjacent_on_pitch(bomb_coordinate);
        target_coords.push(bomb_coordinate); // include center square

        // Java: affectedPlayers = players at those coordinates (in reverse order)
        // Collect player ids at each coordinate
        let mut affected_player_ids: Vec<(String, FieldCoordinate)> = target_coords.iter()
            .filter_map(|&coord| {
                game.field_model.player_at(coord)
                    .map(|id| (id.to_string(), coord))
            })
            .collect();
        affected_player_ids.reverse();

        if affected_player_ids.is_empty() {
            return StepOutcome::next().publish(StepParameter::BombExploded(true));
        }

        // Java: for each affected player: rollForEffect = !fBombCoordinate.equals(playerCoordinate)
        let mut outcome = StepOutcome::next();
        for (player_id, player_coord) in affected_player_ids {
            let roll_for_effect = player_coord != bomb_coordinate;
            let seq = SpecialEffect::build_sequence(&SpecialEffectParams {
                special_effect_key: "BOMB".into(),
                player_id,
                roll_for_effect,
            });
            outcome = outcome.push_seq(seq);
        }

        outcome.publish(StepParameter::BombExploded(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn no_bomb_coordinate_returns_next_step_no_push() {
        let mut game = make_game();
        let mut step = StepResolveBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn bomb_with_catcher_returns_next_no_explosion() {
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepResolveBomb::new();
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn bomb_explodes_clears_bomb_state() {
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(5, 5));
        game.field_model.bomb_moving = true;
        let mut step = StepResolveBomb::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.bomb_coordinate.is_none());
        assert!(!game.field_model.bomb_moving);
    }

    #[test]
    fn bomb_with_no_players_publishes_bomb_exploded() {
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepResolveBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BombExploded(true))));
    }

    #[test]
    fn bomb_with_adjacent_player_pushes_special_effect_sequence() {
        let mut game = make_game();
        let bomb_coord = FieldCoordinate::new(5, 5);
        let player_coord = FieldCoordinate::new(6, 5); // adjacent
        game.field_model.bomb_coordinate = Some(bomb_coord);
        let mut p = Player::default();
        p.id = "p1".into();
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("p1", player_coord);

        let mut step = StepResolveBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty(), "should push SpecialEffect sequence");
        assert_eq!(out.pushes[0][0].step_id, StepId::SpecialEffect);
    }

    #[test]
    fn player_at_bomb_square_gets_no_roll_for_effect() {
        let mut game = make_game();
        let bomb_coord = FieldCoordinate::new(5, 5);
        game.field_model.bomb_coordinate = Some(bomb_coord);
        let mut p = Player::default();
        p.id = "p1".into();
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("p1", bomb_coord);

        let mut step = StepResolveBomb::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty());
        // Find RollForEffect param in the SpecialEffect step — should be false for center square
        let special_effect_step = out.pushes[0].iter()
            .find(|s| s.step_id == StepId::SpecialEffect);
        if let Some(step) = special_effect_step {
            if let Some(StepParameter::RollForEffect(v)) = step.params.iter().find(|p| matches!(p, StepParameter::RollForEffect(_))) {
                assert!(!v, "player at bomb square should not roll for effect");
            }
        }
    }

    #[test]
    fn set_catcher_id_accepted() {
        let mut step = StepResolveBomb::default();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p1".into()))));
        assert_eq!(step.catcher_id.as_deref(), Some("p1"));
    }
}
