use ffb_model::enums::Direction;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_scatter_player::ReportScatterPlayer;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::SteadyFootingContext;
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
/// ThrowInMechanic.scatter → D8 roll mapped via Direction::for_roll + FieldCoordinate::step → wired.
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
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!actingPlayer.getPlayer().hasSkillProperty(MOVES_RANDOMLY)) NEXT_STEP
        let moves_randomly = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);

        if !moves_randomly {
            return StepOutcome::next();
        }

        let coordinate_from = match self.coordinate_from {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: ThrowInMechanic.scatter(coordinateFrom, directionRoll) → coordinateTo
        // Roll D8, map to scatter direction, step one square.
        let (coordinate_to, scatter_direction, scatter_roll) = if let Some(pre_set) = self.coordinate_to {
            (pre_set, None, None)
        } else {
            let roll = rng.d8();
            let direction = Direction::for_roll(roll).unwrap_or(Direction::North);
            (coordinate_from.step(direction, 1), Some(direction), Some(roll))
        };

        // Java: getResult().addReport(new ReportScatterPlayer(coordinateFrom, coordinateTo, directions, false))
        {
            let dirs = scatter_direction.map(|d| vec![d]).unwrap_or_default();
            let rolls = scatter_roll.map(|r| vec![r]).unwrap_or_default();
            game.report_list.add(ReportScatterPlayer::new(
                coordinate_from,
                coordinate_to,
                dirs,
                rolls,
                Some(false),
            ));
        }

        // Check if out of bounds
        if !FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
            // Java: publishParameter(INJURY_TYPE, InjuryTypeCrowdPush) + GOTO_LABEL_ON_FALL_DOWN
            let label = self.goto_label_on_fall_down.clone();
            let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeCrowdPush".into());
            return StepOutcome::goto(&label)
                .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
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
    use ffb_model::report::report_id::ReportId;
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
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
    }

    fn add_ball_and_chain_player(game: &mut Game, id: &str, coord: FieldCoordinate) {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "ballchain".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 2, strength: 5, agility: 1, passing: 6, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BallAndChain, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, coord);
        game.acting_player.player_id = Some(id.into());
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
        let oob = FieldCoordinate::new(26, 5);
        add_ball_and_chain_player(&mut game, "carrier", from);
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
        add_ball_and_chain_player(&mut game, "carrier", from);
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
        add_ball_and_chain_player(&mut game, "carrier", from);
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

    #[test]
    fn scatter_player_report_added_with_preset_coordinate() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_ball_and_chain_player(&mut game, "carrier", from);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "should have SCATTER_PLAYER report when ball-and-chain player moves");
    }

    #[test]
    fn scatter_player_report_added_without_preset_coordinate() {
        let mut game = make_game();
        let from = FieldCoordinate::new(12, 6);
        add_ball_and_chain_player(&mut game, "carrier", from);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = None;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "SCATTER_PLAYER report should be added when D8 scatter is rolled");
    }

    #[test]
    fn scatter_without_preset_coordinate_to_moves_player() {
        // Without a preset coordinate_to, the step rolls D8 to scatter.
        // Any D8 roll from center (12, 6) stays in bounds → NextStep.
        let mut game = make_game();
        let from = FieldCoordinate::new(12, 6);
        let mut step = StepMoveBallAndChain::new("end".into(), "fall".into());
        step.coordinate_from = Some(from);
        step.coordinate_to = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // D8 roll of 1 (seed 0) → North → (12,5) — in bounds → NextStep.
        assert_eq!(out.action, StepAction::NextStep);
    }
}
