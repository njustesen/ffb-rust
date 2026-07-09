/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.SwoopBehaviour.
///
/// Hook for the Swoop skill (vampire/gargoyle flying attack). Fires on StepSwoop.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::step::framework::{StepOutcome, StepParameter};
use ffb_model::enums::{Direction, ReRollSource, SkillId};
use ffb_model::model::skill_def::SkillWithValue;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2025::report_swoop_direction::ReportSwoopDirection;
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2025::throw_in_mechanic::ThrowInMechanic;
use ffb_mechanics::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;

/// Hook state passed by StepSwoop into the skill-behaviour hook system.
pub struct SwoopStepHookState {
    pub using_swoop: bool,
    pub coordinate_from: Option<FieldCoordinate>,
    pub coordinate_to: Option<FieldCoordinate>,
    pub swoop_direction: Option<Direction>,
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub player_id: Option<String>,
    pub outcome: Option<StepOutcome>,
}

pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(SwoopStepModifier));
        registry.register(SkillId::Swoop, sb);
    }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        let _has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::Swoop))
            .unwrap_or(false);
        false
    }
}

pub struct SwoopStepModifier;

impl StepModifierTrait for SwoopStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Swoop }

    fn priority(&self) -> i32 { 0 }

    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<SwoopStepHookState>()
            .expect("SwoopStepModifier: step_state must be SwoopStepHookState");

        let player_id = match state.player_id.clone() {
            Some(id) => id,
            None => return false,
        };
        let has_property = game.player(&player_id)
            .map(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION))
            .unwrap_or(false);
        if !state.using_swoop || !has_property {
            return false;
        }

        if state.re_rolled_action.as_deref() == Some("SWOOP_DIRECTION") {
            let should_skip = match state.re_roll_source.clone() {
                None => true,
                Some(ref source_name) => {
                    let source = ReRollSource::new(source_name.as_str());
                    !use_reroll(game, &source, &player_id)
                }
            };
            if should_skip {
                let dir = state.swoop_direction.unwrap_or(Direction::East);
                state.outcome = Some(
                    StepOutcome::next()
                        .publish(StepParameter::Direction(dir))
                        .publish(StepParameter::UsingSwoop(true))
                );
                return false;
            }
        }

        let coord_from = game.field_model.player_coordinate(&player_id)
            .unwrap_or(FieldCoordinate::new(0, 0));
        state.coordinate_from = Some(coord_from);

        let scatter_roll = rng.d6();

        let mechanic = ThrowInMechanic::new();
        let coord_to = state.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));

        let template_dir = if coord_from.x < coord_to.x {
            Direction::East
        } else if coord_from.x > coord_to.x {
            Direction::West
        } else if coord_from.y < coord_to.y {
            Direction::South
        } else {
            Direction::North
        };
        let scatter_direction = mechanic.interpret_throw_in_direction_roll_with_template(template_dir, scatter_roll);
        state.swoop_direction = Some(scatter_direction);

        let indicator_coordinate = coord_from.step(scatter_direction, 1);
        game.field_model.clear_move_squares();
        let out_of_bounds = !indicator_coordinate.is_on_pitch();
        if !out_of_bounds {
            game.field_model.add_move_square(MoveSquare::new(indicator_coordinate, 0, 0));
        }

        game.report_list.add(ReportSwoopDirection::new(
            Some(scatter_direction),
            scatter_roll,
            player_id.clone(),
            out_of_bounds,
        ));

        if state.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "SWOOP_DIRECTION", 0, false) {
                state.outcome = Some(StepOutcome::cont().with_prompt(prompt));
                return false;
            }
        }

        state.outcome = Some(
            StepOutcome::next()
                .publish(StepParameter::Direction(scatter_direction))
                .publish(StepParameter::UsingSwoop(true))
        );

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_swoop(game: &mut Game, id: &str, at: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Swoop)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, at);
    }

    fn make_hook_state(player_id: &str, coordinate_to: FieldCoordinate) -> SwoopStepHookState {
        SwoopStepHookState {
            using_swoop: true,
            coordinate_from: None,
            coordinate_to: Some(coordinate_to),
            swoop_direction: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id: Some(player_id.into()),
            outcome: None,
        }
    }

    #[test]
    fn name_is_swoop_behaviour() {
        assert_eq!(SwoopBehaviour::new().name(), "SwoopBehaviour");
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!SwoopBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false_no_player() {
        let b = SwoopBehaviour::new();
        let mut game = make_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SwoopBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn execute_step_hook_false_with_bb2025() {
        let b = SwoopBehaviour::new();
        let mut game = make_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn modifier_applies_to_swoop() {
        let m = SwoopStepModifier;
        assert!(m.applies_to(StepId::Swoop));
    }

    #[test]
    fn modifier_does_not_apply_to_block_roll() {
        let m = SwoopStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        SwoopBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Swoop).expect("Swoop must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn register_into_registers_swoop_skill() {
        let mut reg = SkillRegistry::empty();
        SwoopBehaviour::register_into(&mut reg);
        assert!(reg.get(SkillId::Swoop).is_some());
    }

    #[test]
    fn no_op_when_using_swoop_is_false() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        let mut hook = SwoopStepHookState {
            using_swoop: false,
            coordinate_from: None,
            coordinate_to: Some(FieldCoordinate::new(8, 5)),
            swoop_direction: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id: Some("p1".into()),
            outcome: None,
        };
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
        assert!(hook.outcome.is_none());
    }

    #[test]
    fn no_op_when_player_lacks_ttm_scatters_property() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p2".into(), name: "p2".into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(5, 5));
        let mut hook = make_hook_state("p2", FieldCoordinate::new(8, 5));
        hook.using_swoop = true;
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
        assert!(hook.outcome.is_none());
    }

    #[test]
    fn reroll_declined_branch_publishes_direction_and_next_step() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        let mut hook = SwoopStepHookState {
            using_swoop: true,
            coordinate_from: None,
            coordinate_to: Some(FieldCoordinate::new(8, 5)),
            swoop_direction: Some(Direction::East),
            re_rolled_action: Some("SWOOP_DIRECTION".into()),
            re_roll_source: None,
            player_id: Some("p1".into()),
            outcome: None,
        };
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
        let outcome = hook.outcome.expect("outcome must be set");
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingSwoop(true))));
    }

    #[test]
    fn normal_path_sets_swoop_direction_and_publishes_next_step() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        let mut hook = make_hook_state("p1", FieldCoordinate::new(8, 5));
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(42), &mut hook);
        assert!(!result);
        assert!(hook.swoop_direction.is_some());
        let outcome = hook.outcome.expect("outcome must be set");
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingSwoop(true))));
    }

    #[test]
    fn normal_path_clears_move_squares_before_adding() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(1, 1), 2, 3));
        let mut hook = make_hook_state("p1", FieldCoordinate::new(8, 5));
        m.handle_execute_step(&mut game, &mut GameRng::new(1), &mut hook);
        assert!(game.field_model.get_move_square(FieldCoordinate::new(1, 1)).is_none());
    }

    #[test]
    fn normal_path_adds_report() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        let before = game.report_list.size();
        let mut hook = make_hook_state("p1", FieldCoordinate::new(8, 5));
        m.handle_execute_step(&mut game, &mut GameRng::new(7), &mut hook);
        assert_eq!(game.report_list.size(), before + 1);
    }

    #[test]
    fn coordinate_from_is_set_from_field_model() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        let player_coord = FieldCoordinate::new(3, 7);
        add_player_with_swoop(&mut game, "p1", player_coord);
        let mut hook = make_hook_state("p1", FieldCoordinate::new(6, 7));
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert_eq!(hook.coordinate_from, Some(player_coord));
    }

    #[test]
    fn handle_execute_step_returns_false_always() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        add_player_with_swoop(&mut game, "p1", FieldCoordinate::new(5, 5));
        let mut hook = make_hook_state("p1", FieldCoordinate::new(5, 8));
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
    }

    #[test]
    fn no_op_when_player_id_is_none() {
        let m = SwoopStepModifier;
        let mut game = make_game();
        let mut hook = SwoopStepHookState {
            using_swoop: true,
            coordinate_from: None,
            coordinate_to: Some(FieldCoordinate::new(8, 5)),
            swoop_direction: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id: None,
            outcome: None,
        };
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
        assert!(hook.outcome.is_none());
    }
}
