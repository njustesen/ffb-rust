use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_event::ReportEvent;

// ── MonstrousMouthStepModifier ────────────────────────────────────────────────

/// BB2025-only: an unrelated mechanic from BB2016/BB2020's MonstrousMouth (which grants a Catch
/// re-roll, see the `bb2016`/`bb2020` monstrous_mouth_behaviour.rs Catch-twin implementations).
/// Here MonstrousMouth forces a push and blocks ball-strip when the defender was chomped.
pub struct MonstrousMouthStepModifier;

impl StepModifierTrait for MonstrousMouthStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 1 }

    /// Java: MonstrousMouthBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    ///
    /// ```text
    /// if (playerState.isChomped()) {
    ///   state.doPush = true;
    ///   state.pushbackStack.clear();
    ///   step.publishParameter(STARTING_PUSHBACK_SQUARE, null);
    ///   step.publishParameter(FOLLOWUP_CHOICE, false);
    ///   step.publishParameter(BALL_KNOCKED_LOSE, false);
    ///   step.publishParameter(CATCH_SCATTER_THROW_IN_MODE, null);
    ///   if (state.defender has ball) addReport("Strip ball is prevented as the player is chomped.");
    ///   return true;
    /// }
    /// return false;
    /// ```
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("MonstrousMouthStepModifier: step_state must be StepPushbackHookState");

        // Java: playerState.isChomped() — read from state.defender's current field-model state.
        let is_chomped = game.field_model.player_state(&state.defender_id)
            .map(|s| s.is_chomped())
            .unwrap_or(false);
        if !is_chomped {
            return false;
        }

        state.do_push = true;
        // Java also clears state.pushbackStack (the client-chosen-coordinate stack), which lives
        // on the StepPushback struct itself, not in the hook state — matches the same limitation
        // already accepted by the shipped StandFirmStepModifier.
        state.pushback_squares.clear();
        state.starting_pushback_square = None;

        // Java: if (defender has ball) addReport("Strip ball is prevented as the player is chomped.")
        let defender_coord = game.field_model.player_coordinate(&state.defender_id);
        let defender_has_ball = defender_coord.is_some() && game.field_model.ball_coordinate == defender_coord;
        if defender_has_ball {
            game.report_list.add(ReportEvent::new(Some(
                "Strip ball is prevented as the player is chomped.".to_string(),
            )));
        }

        true
    }
}

// ── MonstrousMouthBehaviour ───────────────────────────────────────────────────

/// Monstrous Mouth (BB2025): forces a push and blocks ball-strip against a chomped defender.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2025.MonstrousMouthBehaviour`.
pub struct MonstrousMouthBehaviour;

impl MonstrousMouthBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(MonstrousMouthStepModifier));
        registry.register(SkillId::MonstrousMouth, sb);
    }
}

impl Default for MonstrousMouthBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, Rules};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn default_hook_state(defender_id: &str) -> StepPushbackHookState {
        StepPushbackHookState::new(
            defender_id.into(), None, None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        )
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        MonstrousMouthBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::MonstrousMouth).expect("MonstrousMouth must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = MonstrousMouthStepModifier;
        assert!(m.applies_to(StepId::Pushback));
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_one() {
        assert_eq!(MonstrousMouthStepModifier.priority(), 1);
    }

    #[test]
    fn not_chomped_returns_false() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = MonstrousMouthStepModifier;
        let mut hs = default_hook_state("def1");
        assert!(!m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs));
    }

    #[test]
    fn chomped_defender_forces_push() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        let chomped = PlayerState::new(PS_STANDING).change_chomped(true);
        game.field_model.set_player_state("def1", chomped);

        let m = MonstrousMouthStepModifier;
        let mut hs = default_hook_state("def1");
        hs.starting_pushback_square = Some(ffb_model::types::PushbackSquare::new(
            FieldCoordinate::new(5, 5), ffb_model::enums::Direction::North, true,
        ));
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result);
        assert!(hs.do_push);
        assert!(hs.starting_pushback_square.is_none());
    }

    #[test]
    fn chomped_defender_with_ball_prevents_strip_and_reports() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("def1", coord);
        game.field_model.ball_coordinate = Some(coord);
        let chomped = PlayerState::new(PS_STANDING).change_chomped(true);
        game.field_model.set_player_state("def1", chomped);

        let m = MonstrousMouthStepModifier;
        let mut hs = default_hook_state("def1");
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::EVENT));
    }

    fn test_game() -> ffb_model::model::game::Game {
        let home = ffb_model::model::team::Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = home.clone();
        ffb_model::model::game::Game::new(home, away, ffb_model::enums::Rules::Bb2025)
    }

}
