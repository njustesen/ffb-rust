/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepFoulAppearance` (BB2016).
///
/// Resolves the Foul Appearance negatrait check against an attacker.
///
/// Java flow (via FoulAppearanceBehaviour.handleExecuteStepHook):
/// 1. If defender has FoulAppearance AND attacker lacks forceRollBeforeBeingBlocked cancel:
///    a. If re-rolling: consume re-roll or → goToLabelOnFailure (hasBlocked=true, turnStarted=true)
///    b. Roll 1d6 (rollSkill); success = roll >= minimumRollResistingFoulAppearance (= 2)
///    c. On failure: ask for re-roll, or → goToLabelOnFailure
/// 2. Else: NEXT_STEP immediately
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepFoulAppearance`.
use ffb_model::model::game::Game;
use ffb_model::enums::SkillId;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::ReRollSource;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepFoulAppearance {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Java: roll (stored for re-roll path)
    pub roll: i32,
}

impl StepFoulAppearance {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }
}

impl Default for StepFoulAppearance {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepFoulAppearance {
    fn id(&self) -> StepId { StepId::FoulAppearance }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => self.execute_step(game, rng),
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
            _ => false,
        }
    }
}

impl StepFoulAppearance {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: FoulAppearanceBehaviour.handleExecuteStepHook checks:
        // 1. getDefender() != null && UtilCards.hasSkill(defender, FoulAppearance)
        // 2. !UtilCards.hasSkillToCancelProperty(actingPlayer, forceRollBeforeBeingBlocked)
        let defender_id = game.defender_id.clone();
        let defender_has_fa = defender_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::FoulAppearance))
            .unwrap_or(false);
        let attacker_cancels = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::FORCE_ROLL_BEFORE_BEING_BLOCKED))
            .unwrap_or(false);

        if !defender_has_fa || attacker_cancels {
            return StepOutcome::next();
        }

        // Java: if (FOUL_APPEARANCE == reRolledAction)
        //         if (source == null || !useReRoll) → hasBlocked=true, turnStarted=true → goToLabel
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "FOUL_APPEARANCE").unwrap_or(false);

        if already_rerolled {
            let pid = game.acting_player.player_id.clone();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, pid.as_deref().unwrap_or("")))
                .unwrap_or(false);
            if !consumed {
                return self.fail_fa(game);
            }
            self.roll = 0; // fresh roll after re-roll consumed
        }

        // Java: roll = diceRoller.rollSkill() (1d6); minimumRoll = minimumRollResistingFoulAppearance() = 2
        if self.roll == 0 {
            self.roll = rng.d6();
        }
        let minimum_roll = DiceInterpreter::minimum_roll_resisting_foul_appearance();
        let may_block = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        if may_block {
            return StepOutcome::next();
        }

        // Failure — try re-roll if this is the first failure
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("FOUL_APPEARANCE"));

            // Skill re-roll first (e.g. Pro)
            let pid = game.acting_player.player_id.clone();
            let skill_source = find_skill_reroll_source(game, "FOUL_APPEARANCE");
            if let Some(source) = skill_source {
                use_reroll(game, &source, pid.as_deref().unwrap_or(""));
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                return self.execute_step(game, rng);
            }

            // TRR offer
            if let Some(prompt) = ask_for_reroll_if_available(game, "FOUL_APPEARANCE", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_fa(game)
    }

    fn fail_fa(&mut self, game: &mut Game) -> StepOutcome {
        // Java: actingPlayer.setHasBlocked(true); game.getTurnData().setTurnStarted(true)
        game.acting_player.has_blocked = true;
        game.turn_data_mut().turn_started = true;
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, SkillId, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, skills: Vec<SkillId>) -> Player {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(p.clone());
        p
    }

    #[test]
    fn id_is_foul_appearance() {
        assert_eq!(StepFoulAppearance::new("fail".into()).id(), StepId::FoulAppearance);
    }

    #[test]
    fn no_defender_skips_roll_and_returns_next() {
        // Java: if (defender == null) → NEXT_STEP
        let mut game = make_game();
        game.defender_id = None;
        let mut step = StepFoulAppearance::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn defender_without_foul_appearance_returns_next() {
        let mut game = make_game();
        add_player(&mut game, "def", vec![]);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn defender_with_fa_successful_roll_returns_next() {
        // roll = 2 → may_block = true → NEXT_STEP
        let mut game = make_game();
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail".into());
        step.roll = 2; // roll >= 2 → success
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn defender_with_fa_failed_roll_no_reroll_goes_to_label() {
        // roll = 1 → failure, no TRR → goto fail label; hasBlocked=true, turnStarted=true
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail".into());
        step.roll = 1; // guaranteed failure
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fa_fail"));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().turn_started);
    }

    #[test]
    fn defender_with_fa_failed_roll_with_trr_offers_reroll() {
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepFoulAppearance::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepFoulAppearance::new("old".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
