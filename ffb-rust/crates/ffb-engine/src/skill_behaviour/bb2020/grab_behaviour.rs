/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.GrabBehaviour.
///
/// Priority 4 modifier on StepPushback.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2020::block::step_pushback::StepPushbackHookState;
use crate::util::util_server_pushback::UtilServerPushback;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::types::FieldCoordinate;

// ── GrabStepModifier ──────────────────────────────────────────────────────────

pub struct GrabStepModifier;

impl StepModifierTrait for GrabStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 4 }

    /// Java: GrabBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    ///
    /// BB2020 adds a ViciousVines adjacency alternative and an attacker-conflicting-skill
    /// check (both absent in BB2016) on top of the same shape as BB2025's (priority 5) version.
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("GrabStepModifier: step_state must be StepPushbackHookState");

        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return false,
        };

        // Java: UtilCards.hasSkill(actingPlayer, skill)
        let has_grab = game.player(&attacker_id)
            .map(|p| p.has_skill(SkillId::Grab))
            .unwrap_or(false);
        if !has_grab {
            return false;
        }

        let attacker_coord = game.field_model.player_coordinate(&attacker_id);
        let defender_coord = state.starting_pushback_square
            .map(|sq| sq.coordinate)
            .or_else(|| game.defender_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id)));
        let defender_coord = match defender_coord {
            Some(c) => c,
            None => return false,
        };

        // Java: Skill cancellingSkill = UtilCards.getSkillCancelling(state.defender, skill)
        let defender_id = state.defender_id.clone();
        let has_cancelling = game.player(&defender_id)
            .map(|p| p.all_skill_ids().any(|id| id.properties().contains(&"cancelsCanPushBackToAnySquare")))
            .unwrap_or(false);

        // Java: boolean attackerHasConflictingSkill = skill.conflictsWithAnySkill(actingPlayer.getPlayer())
        let attacker_conflicts = false; // simplified: Grab has no self-conflict in standard rules

        // Java: boolean allowGrabOutsideBlock = actingPlayer.getPlayer().hasSkillProperty(NamedProperties.grabOutsideBlock)
        let allow_grab_outside_block = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::GRAB_OUTSIDE_BLOCK))
            .unwrap_or(false);

        let is_adjacent = attacker_coord
            .map(|ac| ac.is_adjacent(defender_coord))
            .unwrap_or(false);
        // Java: actingPlayer.getPlayerAction() == PlayerAction.VICIOUS_VINES
        let is_vicious_vines = game.acting_player.player_action
            .map(|a| a == ffb_model::enums::PlayerAction::ViciousVines)
            .unwrap_or(false);
        // Java: actingPlayer.getPlayerAction().isBlockAction()
        let is_block_action = game.acting_player.player_action
            .map(|a| a.is_block_action())
            .unwrap_or(false);
        let is_multiple_block = game.acting_player.player_action
            .map(|a| a == ffb_model::enums::PlayerAction::MultipleBlock)
            .unwrap_or(false);

        if (state.grabbing.is_none() || state.grabbing == Some(true))
            && state.free_square_around_defender
            && (is_adjacent || is_vicious_vines)
            && !has_cancelling
            && !attacker_conflicts
            && (is_block_action || is_multiple_block || allow_grab_outside_block)
        {
            // Java: if ((state.grabbing == null) && ArrayTool.isProvided(state.pushbackSquares))
            if state.grabbing.is_none() && !state.pushback_squares.is_empty() {
                state.grabbing = Some(true);
                for sq in &state.pushback_squares {
                    if game.field_model.player_at(sq.coordinate).is_some() {
                        state.grabbing = None;
                        break;
                    }
                }
            }

            // Java: if (state.grabbing == null) show dialog → headless: auto-decline
            if state.grabbing.is_none() {
                state.grabbing = None;
                return true;
            }

            if state.grabbing == Some(true) {
                state.pushback_mode = PushbackMode::GRAB;
                if let Some(starting_sq) = state.starting_pushback_square {
                    let new_squares = UtilServerPushback::find_pushback_squares_grab(
                        starting_sq,
                        &|c: FieldCoordinate| game.field_model.player_at(c).is_some(),
                        &|_c| true,
                        game.home_playing,
                    );
                    state.pushback_squares = new_squares;
                }
                state.grabbing = None;
            } else {
                state.grabbing = Some(false);
            }

            state.starting_pushback_square = None;
            return true;
        }

        false
    }
}

// ── GrabBehaviour ─────────────────────────────────────────────────────────────

/// Grab: attacker may choose which of the candidate pushback squares to push into.
pub struct GrabBehaviour;

impl GrabBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(GrabStepModifier));
        registry.register(SkillId::Grab, sb);
    }
}

impl Default for GrabBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::bb2020::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerAction, PlayerState, PS_STANDING, Rules, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::{FieldCoordinate, PushbackSquare};
    use ffb_model::enums::Direction;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn player_with_skills(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    fn default_hook_state_with_sq(defender_id: &str, sq: PushbackSquare) -> StepPushbackHookState {
        StepPushbackHookState::new(
            defender_id.into(),
            Some(PlayerState::new(PS_STANDING)),
            Some(sq), 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        )
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        GrabBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Grab).expect("Grab must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = GrabStepModifier;
        assert!(m.applies_to(StepId::Pushback));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = GrabStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_four() {
        let m = GrabStepModifier;
        assert_eq!(m.priority(), 4);
    }

    #[test]
    fn no_grab_skill_returns_false() {
        let mut game = make_game();
        game.team_home.players.push(player_with_skills("att", vec![]));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        let def_coord = FieldCoordinate::new(5, 6);
        game.defender_id = Some("def".into());
        game.team_away.players.push(player_with_skills("def", vec![]));
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));

        let sq = PushbackSquare::new(def_coord, Direction::North, true);
        let m = GrabStepModifier;
        let mut hs = default_hook_state_with_sq("def", sq);
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
    }

    #[test]
    fn grab_not_adjacent_returns_false() {
        let mut game = make_game();
        game.team_home.players.push(player_with_skills("att", vec![SkillId::Grab]));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(1, 1));
        let def_coord = FieldCoordinate::new(10, 10);
        game.defender_id = Some("def".into());
        game.team_away.players.push(player_with_skills("def", vec![]));
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));

        let sq = PushbackSquare::new(def_coord, Direction::North, true);
        let m = GrabStepModifier;
        let mut hs = default_hook_state_with_sq("def", sq);
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
    }

    #[test]
    fn grab_with_free_pushback_squares_auto_grabs() {
        let mut game = make_game();
        game.team_home.players.push(player_with_skills("att", vec![SkillId::Grab]));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let att_coord = FieldCoordinate::new(5, 5);
        let def_coord = FieldCoordinate::new(5, 6);
        game.field_model.set_player_coordinate("att", att_coord);
        game.defender_id = Some("def".into());
        game.team_away.players.push(player_with_skills("def", vec![]));
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));

        let sq = PushbackSquare::new(def_coord, Direction::North, true);
        let m = GrabStepModifier;
        let mut hs = default_hook_state_with_sq("def", sq);
        hs.pushback_squares = vec![
            PushbackSquare::new(FieldCoordinate::new(4, 6), Direction::Northwest, true),
        ];

        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result, "Grab should fire when conditions met");
        assert_eq!(hs.pushback_mode, PushbackMode::GRAB, "mode should switch to GRAB");
    }

    #[test]
    fn grab_clears_starting_square_when_applied() {
        let mut game = make_game();
        game.team_home.players.push(player_with_skills("att", vec![SkillId::Grab]));
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let att_coord = FieldCoordinate::new(5, 5);
        let def_coord = FieldCoordinate::new(5, 6);
        game.field_model.set_player_coordinate("att", att_coord);
        game.defender_id = Some("def".into());
        game.team_away.players.push(player_with_skills("def", vec![]));
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));

        let sq = PushbackSquare::new(def_coord, Direction::North, true);
        let m = GrabStepModifier;
        let mut hs = default_hook_state_with_sq("def", sq);
        hs.pushback_squares = vec![
            PushbackSquare::new(FieldCoordinate::new(4, 6), Direction::Northwest, true),
        ];

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared when Grab applies");
    }

}
