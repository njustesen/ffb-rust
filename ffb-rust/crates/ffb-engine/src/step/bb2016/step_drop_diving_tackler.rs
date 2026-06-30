/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepDropDivingTackler`.
///
/// Drops the defender prone when the DivingTackle skill was used, then resets the
/// defender ID.  Expects COORDINATE_FROM and USING_DIVING_TACKLE init parameters.
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury::drop_player_no_sph;
use crate::util::UtilServerPlayerMove;

/// Java: `StepDropDivingTackler` (bb2016).
pub struct StepDropDivingTackler {
    /// Java: `fUsingDivingTackle`
    using_diving_tackle: bool,
    /// Java: `fCoordinateFrom`
    coordinate_from: Option<FieldCoordinate>,
}

impl StepDropDivingTackler {
    pub fn new() -> Self {
        Self { using_diving_tackle: false, coordinate_from: None }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let mut outcome = StepOutcome::next();

        if self.using_diving_tackle {
            if let Some(defender_id) = game.defender_id.clone() {
                if !defender_id.is_empty() {
                    if let Some(coord) = self.coordinate_from {
                        let old_coord = game.field_model.player_coordinate(&defender_id);
                        game.field_model.set_player_coordinate(&defender_id, coord);
                        if let Some(old) = old_coord {
                            if game.field_model.ball_coordinate == Some(old) {
                                game.field_model.ball_coordinate = Some(coord);
                            }
                        }
                        outcome = outcome.publish(StepParameter::PlayerEnteringSquare(defender_id.clone()));
                    }

                    let params = drop_player_no_sph(game, &defender_id);
                    for p in params {
                        outcome = outcome.publish(p);
                    }
                    UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
                }
            }
        }

        // Reset DivingTackle & Shadowing attributes: publish defender id, then clear
        let defender_id = game.defender_id.clone();
        outcome = outcome.publish(StepParameter::PlayerId(defender_id.unwrap_or_default()));
        game.defender_id = None;

        outcome
    }
}

impl Default for StepDropDivingTackler {
    fn default() -> Self { Self::new() }
}

impl Step for StepDropDivingTackler {
    fn id(&self) -> StepId { StepId::DropDivingTackler }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = *v; true }
            StepParameter::CoordinateFrom(v)    => { self.coordinate_from = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) {
        game.team_away.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
    }

    #[test]
    fn id_is_drop_diving_tackler() {
        assert_eq!(StepDropDivingTackler::new().id(), StepId::DropDivingTackler);
    }

    #[test]
    fn without_diving_tackle_clears_defender() {
        let mut step = StepDropDivingTackler::new();
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_STANDING, "not dropped without diving tackle");
    }

    #[test]
    fn with_diving_tackle_drops_defender() {
        let mut step = StepDropDivingTackler::new();
        step.set_parameter(&StepParameter::UsingDivingTackle(true));
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let state = game.field_model.player_state("def").unwrap();
        assert_eq!(state.base(), PS_PRONE, "defender should be dropped");
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn coordinate_from_moves_defender() {
        let mut step = StepDropDivingTackler::new();
        step.set_parameter(&StepParameter::UsingDivingTackle(true));
        let new_pos = FieldCoordinate::new(10, 7);
        step.set_parameter(&StepParameter::CoordinateFrom(new_pos));
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let coord = game.field_model.player_coordinate("def").unwrap();
        assert_eq!(coord, new_pos);
        let has_entering = outcome.published.iter().any(|p| matches!(p, StepParameter::PlayerEnteringSquare(_)));
        assert!(has_entering);
    }

    #[test]
    fn always_publishes_player_id() {
        let mut step = StepDropDivingTackler::new();
        let mut game = make_game();
        add_player(&mut game, "def", PS_STANDING);
        game.defender_id = Some("def".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_player_id = outcome.published.iter().any(|p| matches!(p, StepParameter::PlayerId(_)));
        assert!(has_player_id);
    }

    #[test]
    fn no_defender_clears_defender_id() {
        let mut step = StepDropDivingTackler::new();
        let mut game = make_game();
        game.defender_id = None;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
    }
}
