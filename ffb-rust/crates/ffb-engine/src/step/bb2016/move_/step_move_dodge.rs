use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::drop_player_context::SteadyFootingContext;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::dodge_context::DodgeContext;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepMoveDodge.
///
/// Resolves a dodge roll when leaving a tackle zone in BB2016.
///
/// BB2016 differences from BB2025:
/// - `actingPlayer.isDodging()` guard: if !dodging → NEXT_STEP
/// - Uses `InjuryTypeDropDodge` on failure (not InjuryTypeFallDown)
/// - Has STAND_FIRM_NO_DROP_ON_FAILED_DODGE game option check
/// - Has fUsingBreakTackle / fUsingDivingTackle fields (same as BB2025)
/// - Has fReRollUsed field
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
/// Expects: COORDINATE_FROM, COORDINATE_TO, DODGE_ROLL, USING_BREAK_TACKLE,
///          USING_DIVING_TACKLE, RE_ROLL_USED published by preceding steps.
///
/// standFirmNoDropOnFailedDodge game option → wired (NEXT_STEP + EndPlayerAction instead of failDodge).
/// DEFERRED(usingBreakTackle): Break-Tackle dialog (canAddStrengthToDodge) not yet ported.
/// DEFERRED(divingTackle): Diving Tackle pre-roll dialog not yet ported.
/// DEFERRED(armBar): Arm-Bar choice dialog not yet ported.
pub struct StepMoveDodge {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: fDodgeRoll
    pub dodge_roll: i32,
    /// Java: fUsingDivingTackle (Boolean tristate)
    pub using_diving_tackle: Option<bool>,
    /// Java: fUsingBreakTackle
    pub using_break_tackle: bool,
    /// Java: fReRollUsed
    pub re_roll_used: bool,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepMoveDodge {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            coordinate_from: None,
            coordinate_to: None,
            dodge_roll: 0,
            using_diving_tackle: None,
            using_break_tackle: false,
            re_roll_used: false,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepMoveDodge {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepMoveDodge {
    fn id(&self) -> StepId { StepId::MoveDodge }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            // DEFERRED(BreakTackle): canAddStrengthToDodge not yet ported — skill use dialog skipped
            // DEFERRED(ArmBar): CLIENT_PLAYER_CHOICE ARM_BAR mode not yet ported
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::DodgeRoll(v) => { self.dodge_roll = *v; true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            StepParameter::UsingBreakTackle(v) => { self.using_break_tackle = *v; true }
            StepParameter::ReRollUsed(v) => { self.re_roll_used = *v; true }
            _ => false,
        }
    }
}

impl StepMoveDodge {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!actingPlayer.isDodging()) { setNextAction(NEXT_STEP); return; }
        if !game.acting_player.dodging {
            return StepOutcome::next();
        }

        let player_id = game.acting_player.player_id.clone();

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "DODGE").unwrap_or(false);

        if already_rerolled {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if consumed {
                self.re_roll_used = true;
            } else {
                return self.fail_dodge();
            }
        }

        if self.dodge_roll == 0 {
            self.dodge_roll = rng.d6();
        }

        let minimum_roll = {
            let factory = DodgeModifierFactory::for_rules(game.rules);
            if let Some(pid) = player_id.as_deref() {
                let acting = game.acting_player.clone();
                let src = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
                let tgt = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
                let ctx = DodgeContext::new(game, &acting, src, tgt);
                let mods = factory.find_applicable(&ctx);
                let agility = game.player(pid).map(|p| p.agility as i32).unwrap_or(3);
                DodgeModifierFactory::minimum_roll(agility, &mods)
            } else {
                2
            }
        };
        let successful = DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll);

        if successful {
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "DODGE").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            return StepOutcome::next()
                .publish(StepParameter::ReRollUsed(self.re_roll_used || re_rolled))
                .publish(StepParameter::UsingBreakTackle(self.using_break_tackle));
        }

        // First failure: try re-roll
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("DODGE"));

            // Skill re-roll (Dodge property canRerollDodge) — auto-used
            let skill_source = find_skill_reroll_source(game, "DODGE");
            if let Some(source) = skill_source {
                let pid = player_id.as_deref().unwrap_or("").to_owned();
                use_reroll(game, &source, &pid);
                self.re_roll_state.re_roll_source = Some(source);
                self.dodge_roll = 0;
                return self.execute_step(game, rng);
            }

            // TRR offer
            if let Some(prompt) = ask_for_reroll_if_available(game, "DODGE", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.dodge_roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        // Java: if (UtilGameOption.isOptionEnabled(game, GameOptionId.STAND_FIRM_NO_DROP_ON_FAILED_DODGE))
        if game.options.is_enabled("standFirmNoDropOnFailedDodge") {
            return StepOutcome::next()
                .publish(StepParameter::EndPlayerAction(true));
        }
        self.fail_dodge()
    }

    fn fail_dodge(&self) -> StepOutcome {
        // Java: BB2016 uses InjuryTypeDropDodge (not InjuryTypeFallDown)
        // DEFERRED(ArmBar): Arm-Bar dialog can override injuryType to InjuryTypeArmBar
        let ctx = SteadyFootingContext::from_injury_type_name("InjuryTypeDropDodge".into());
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
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

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some(id.into());
        game.acting_player.dodging = true;
    }

    #[test]
    fn success_on_roll_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn failure_publishes_steady_footing_context() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))));
    }

    #[test]
    fn success_publishes_re_roll_used_false() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 3;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ReRollUsed(false))));
    }

    #[test]
    fn success_publishes_using_break_tackle() {
        let mut game = make_game();
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 5;
        step.using_break_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::UsingBreakTackle(true))));
    }

    #[test]
    fn not_dodging_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.dodging = false;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1; // would fail if guard weren't hit
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_with_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.dodge_roll = 5;
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepMoveDodge::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_dodge_roll_accepted() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(step.set_parameter(&StepParameter::DodgeRoll(4)));
        assert_eq!(step.dodge_roll, 4);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn stand_firm_no_drop_option_returns_next_step_on_failure() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.options.set("standFirmNoDropOnFailedDodge", "true");
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1; // guaranteed fail
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn stand_firm_no_drop_option_disabled_still_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.options.set("standFirmNoDropOnFailedDodge", "false");
        add_player(&mut game, "p1");
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }
}
