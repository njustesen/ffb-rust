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

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepMoveDodge.
///
/// BB2020 differs from BB2025 only in that DodgeModifierFactory uses BB2020 rules.
///
/// client-only: modifyingSkill dialog (Break Tackle / Diving Tackle) — headless skips skill use choice.
/// client-only: Arm-Bar multi-target path — headless skips arm-bar activation.
/// failDodge: SteadyFootingContext(InjuryTypeDropDodge) published — corrected from InjuryTypeFallDown.
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
    /// Java: usingModifyingSkill (Boolean tristate)
    pub using_modifying_skill: Option<bool>,
    /// Java: usingModifierIgnoringSkill (Boolean tristate)
    pub using_modifier_ignoring_skill: Option<bool>,
    /// Java: armBarPlayers
    pub arm_bar_players: Vec<String>,
    /// Java: armBarPlayerId
    pub arm_bar_player_id: Option<String>,
    /// Java: armBarChoice
    pub arm_bar_choice: bool,
    /// Java: dtRerollAsked
    pub dt_reroll_asked: bool,
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
            using_modifying_skill: None,
            using_modifier_ignoring_skill: None,
            arm_bar_players: Vec::new(),
            arm_bar_player_id: None,
            arm_bar_choice: false,
            dt_reroll_asked: false,
            re_roll_state: ReRollState::new(),
        }
    }
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
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            StepParameter::ArmBarPlayerId(v) => { self.arm_bar_player_id = v.clone(); true }
            StepParameter::DtRerollAsked(v) => { self.dt_reroll_asked = *v; true }
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
        let using_modifier_ignoring = self.using_modifier_ignoring_skill == Some(true);

        if already_rerolled && !using_modifier_ignoring {
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

        let factory = DodgeModifierFactory::for_rules(game.rules);
        let (minimum_roll, mod_names): (i32, Vec<String>) = if let Some(pid) = player_id.as_deref() {
            let acting = game.acting_player.clone();
            let src = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
            let tgt = self.coordinate_to.unwrap_or(FieldCoordinate::new(0, 0));
            let ctx = DodgeContext::new(game, &acting, src, tgt);
            let mods = factory.find_applicable(&ctx);
            let agility = game.player(pid).map(|p| p.agility as i32).unwrap_or(3);
            let min = DodgeModifierFactory::minimum_roll(agility, &mods);
            let names: Vec<String> = mods.iter().map(|m| m.get_report_string().to_string()).collect();
            (min, names)
        } else {
            (2, vec![])
        };
        let successful = DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll);

        // Java line 333-335: addReport(new ReportDodgeRoll(...))
        {
            use ffb_model::report::mixed::report_dodge_roll::ReportDodgeRoll;
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "DODGE").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            game.report_list.add(ReportDodgeRoll::new(
                player_id.clone(),
                successful,
                self.dodge_roll,
                minimum_roll,
                re_rolled,
                mod_names,
                None, // stat_based_roll_modifier: headless never applies modifier-ignoring skill
            ));
        }

        if successful {
            let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                .map(|a| a.name == "DODGE").unwrap_or(false)
                && self.re_roll_state.re_roll_source.is_some();
            StepOutcome::next()
                .publish(StepParameter::ReRollUsed(self.re_roll_used || re_rolled))
                .publish(StepParameter::UsingBreakTackle(self.using_break_tackle))
        } else {
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("DODGE"));

                let skill_source = find_skill_reroll_source(game, "DODGE");
                if let Some(source) = skill_source {
                    let pid = player_id.as_deref().unwrap_or("").to_owned();
                    use_reroll(game, &source, &pid);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.dodge_roll = 0;
                    return self.execute_step(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "DODGE", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.dodge_roll = 0;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }

            self.fail_dodge()
        }
    }

    fn fail_dodge(&self) -> StepOutcome {
        // Java: new InjuryTypeDropDodge(game.getDefender())
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
        Game::new(home, away, Rules::Bb2020)
    }

    fn add_player(game: &mut Game, id: &str) {
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
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn success_on_roll_two_or_above_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 4;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_on_roll_one_goes_to_failure_label() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn not_dodging_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "p1");
        game.acting_player.dodging = false; // guard fires
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1; // would fail if guard weren't hit
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_dodge_roll_accepted() {
        let mut step = StepMoveDodge::new("fail".into());
        assert!(step.set_parameter(&StepParameter::DodgeRoll(4)));
        assert_eq!(step.dodge_roll, 4);
    }

    #[test]
    fn success_emits_dodge_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 6;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::DODGE_ROLL));
    }

    #[test]
    fn failure_emits_dodge_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_player(&mut game, "p1");
        game.acting_player.dodging = true;
        let mut step = StepMoveDodge::new("fail".into());
        step.dodge_roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::DODGE_ROLL));
    }
}
