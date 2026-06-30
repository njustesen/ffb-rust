use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepMoveBallAndChain.
///
/// Handles movement of a Ball-and-Chain player. The player moves randomly in a
/// direction determined by a scatter roll (ThrowInMechanic). If the destination
/// is out of bounds, the player is crowd-pushed. If there is a player in the
/// destination square, a block resolves. Otherwise, the player moves there.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GOTO_LABEL_ON_FALL_DOWN (mandatory),
///              COORDINATE_FROM (mandatory), COORDINATE_TO (mandatory).
///
/// Logic (executeStep):
/// 1. Check movesRandomly property (Ball-and-Chain carrier). If !movesRandomly → NEXT_STEP.
/// 2. Scatter via ThrowInMechanic.scatter(coordinateFrom, roll) → coordinateTo.
/// 3. If coordinateTo is out of bounds:
///    - Publish INJURY_TYPE(InjuryTypeCrowdPush)
///    - GOTO_LABEL(fGotoLabelOnFallDown)
/// 4. If there is a player at coordinateTo (blockDefenderId):
///    - Publish BLOCK_DEFENDER_ID
///    - GOTO_LABEL(fGotoLabelOnEnd)
/// 5. Else:
///    - NEXT_STEP
///
/// TODO(movesRandomly): NamedProperties.MOVES_RANDOMLY check not yet ported.
/// TODO(throwInMechanic): scatter direction from player movement direction not yet ported.
/// TODO(injuryTypeCrowdPush): InjuryTypeCrowdPush publish not yet ported.
/// TODO(blockDefenderId): getPlayerAt(coordinateTo) not yet ported.
pub struct StepMoveBallAndChain {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: fGotoLabelOnFallDown
    pub goto_label_on_fall_down: String,
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo (intended destination after scatter)
    pub coordinate_to: Option<FieldCoordinate>,
}

impl StepMoveBallAndChain {
    pub fn new(goto_label_on_end: String, goto_label_on_fall_down: String) -> Self {
        Self {
            goto_label_on_end,
            goto_label_on_fall_down,
            coordinate_from: None,
            coordinate_to: None,
        }
    }
}

impl Default for StepMoveBallAndChain {
    fn default() -> Self { Self::new(String::new(), String::new()) }
}

impl Step for StepMoveBallAndChain {
    fn id(&self) -> StepId { StepId::MoveBallAndChain }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::GotoLabelOnFallDown(v) => { self.goto_label_on_fall_down = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            _ => false,
        }
    }
}

impl StepMoveBallAndChain {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(movesRandomly): check actingPlayer MOVES_RANDOMLY property
        // For now, always proceed (Ball-and-Chain carrier is always moving randomly)
        let moves_randomly = true;

        if !moves_randomly {
            return StepOutcome::next();
        }

        // Java: ThrowInMechanic.scatter(coordinateFrom, direction_roll) → coordinateTo
        // TODO(throwInMechanic): scatter from movement direction not yet ported
        // For now, use the pre-set coordinateTo if available
        let coordinate_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        let coordinate_to = match self.coordinate_to {
            Some(c) => c,
            None => {
                // TODO(throwInMechanic): scatter direction from movement direction not yet ported
                // Java: ThrowInMechanic.scatter(coordinateFrom, directionRoll)
                // For stub: no pre-set coordinate → return next step
                return StepOutcome::next();
            }
        };

        // Check if out of bounds
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
            // Java: INJURY_TYPE(InjuryTypeCrowdPush) + GOTO_LABEL_ON_FALL_DOWN
            // TODO(injuryTypeCrowdPush): publish InjuryTypeCrowdPush not yet ported
            let label = self.goto_label_on_fall_down.clone();
            return StepOutcome::goto(&label);
        }

        // Check if there is a blocking defender at coordinateTo
        let block_defender_id = game.field_model.player_at(coordinate_to).cloned();

        if let Some(defender_id) = block_defender_id {
            // Java: BLOCK_DEFENDER_ID + GOTO_LABEL_ON_END
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::BlockDefenderId(defender_id));
        }

        // Empty square — move there
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, coord);
    }

    #[test]
    fn empty_target_square_returns_next_step() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn out_of_bounds_target_gotos_fall_down_label() {
        let mut game = make_game();
        let from = FieldCoordinate::new(0, 5);
        // FieldCoordinate x=-1 or x=26 is OOB — use a coordinate known to be OOB
        let oob = FieldCoordinate::new(26, 5);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(oob);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fall"));
    }

    #[test]
    fn occupied_target_square_gotos_end_label() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "defender", to);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn occupied_square_publishes_block_defender_id() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "defender", to);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has_defender = out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(_)));
        assert!(has_defender, "should publish BlockDefenderId when square is occupied");
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepMoveBallAndChain::new("old".into(), "fall".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn set_parameter_goto_label_on_fall_down_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFallDown("new".into())));
        assert_eq!(step.goto_label_on_fall_down, "new");
    }

    #[test]
    fn set_parameter_coordinate_from_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn set_parameter_coordinate_to_accepted() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        let coord = FieldCoordinate::new(6, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateTo(coord)));
        assert_eq!(step.coordinate_to, Some(coord));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn no_coordinate_from_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
