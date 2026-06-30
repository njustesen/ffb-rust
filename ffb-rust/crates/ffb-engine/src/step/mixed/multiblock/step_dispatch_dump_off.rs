/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepDispatchDumpOff`.
///
/// Finds the first target player that currently holds the ball, sets them as
/// defender, pushes a `DumpOff` sequence, and publishes their position.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, SequenceStep};

/// Java: `StepDispatchDumpOff` (mixed/multiblock, BB2020 + BB2025).
pub struct StepDispatchDumpOff {
    /// Java: `targets` — player IDs that are candidates for DumpOff
    targets: Vec<String>,
}

impl StepDispatchDumpOff {
    pub fn new() -> Self { Self { targets: Vec::new() } }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let ball_carrier = game.field_model.ball_coordinate.and_then(|bc| {
            game.field_model.player_at(bc).map(|id| id.clone())
        });
        let Some(carrier) = ball_carrier else {
            return StepOutcome::next();
        };
        if !self.targets.contains(&carrier) {
            return StepOutcome::next();
        }

        let coord = game.field_model.player_coordinate(&carrier);
        game.defender_id = Some(carrier);

        let mut outcome = StepOutcome::next()
            .push_seq(vec![SequenceStep::new(StepId::DumpOff)]);
        if let Some(c) = coord {
            outcome = outcome.publish(StepParameter::DefenderPosition(c));
        }
        outcome
    }
}

impl Default for StepDispatchDumpOff {
    fn default() -> Self { Self::new() }
}

impl Step for StepDispatchDumpOff {
    fn id(&self) -> StepId { StepId::DispatchDumpOff }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockTargets(ids) => {
                self.targets.extend(ids.iter().cloned());
                true
            }
            StepParameter::PlayerIdToRemove(id) => {
                self.targets.retain(|t| t != id);
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
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) -> FieldCoordinate {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
        pos
    }

    #[test]
    fn id_is_dispatch_dump_off() {
        assert_eq!(StepDispatchDumpOff::new().id(), StepId::DispatchDumpOff);
    }

    #[test]
    fn no_ball_carrier_in_targets_is_noop() {
        let mut step = StepDispatchDumpOff::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into()]));
        let mut game = make_game();
        add_player(&mut game, "p1");
        // No ball on the field
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert!(outcome.pushes.is_empty());
    }

    #[test]
    fn ball_carrier_in_targets_pushes_dump_off_and_sets_defender() {
        let mut step = StepDispatchDumpOff::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["carrier".into()]));
        let mut game = make_game();
        let pos = add_player(&mut game, "carrier");
        game.field_model.ball_coordinate = Some(pos);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(game.defender_id, Some("carrier".into()));
        assert_eq!(outcome.pushes.len(), 1);
        assert_eq!(outcome.pushes[0][0].step_id, StepId::DumpOff);
        let has_pos = outcome.published.iter().any(|p| matches!(p, StepParameter::DefenderPosition(_)));
        assert!(has_pos);
    }

    #[test]
    fn remove_target_shrinks_list() {
        let mut step = StepDispatchDumpOff::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into()));
        assert_eq!(step.targets, vec!["p2"]);
    }

    #[test]
    fn ball_carrier_not_in_targets_is_noop() {
        let mut step = StepDispatchDumpOff::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["other".into()]));
        let mut game = make_game();
        let pos = add_player(&mut game, "carrier");
        game.field_model.ball_coordinate = Some(pos);
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(outcome.pushes.is_empty());
    }
}
