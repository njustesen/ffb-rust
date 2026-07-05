/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepFoulAppearance` (BB2020 + BB2025).
///
/// Handles the Foul Appearance skill in the block/attack sequence.
/// Needs `GOTO_LABEL_ON_FAILURE` initialisation parameter.
///
/// Java flow (via FoulAppearanceBehaviour.handleExecuteStepHook, BB2020 edition):
/// 1. Resolve defender from TargetSelectionState (if set) or game.defender_id.
/// 2. If defender has FoulAppearance AND attacker lacks forceRollBeforeBeingBlocked cancel:
///    a. If re-rolling: consume re-roll or → handleFailure (hasBlocked=true, turnStarted=true → goto)
///    b. Roll 1d6 (rollSkill); success = roll >= minimumRollResistingFoulAppearance (= 2)
///    c. On success: commitTargetSelection + NEXT_STEP
///    d. On failure: ask for re-roll, or → handleFailure
/// 3. Else: NEXT_STEP immediately
///
/// Java: `StepFoulAppearance extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::enums::{SkillId, ReRollSource, PlayerAction, PS_PRONE};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepFoulAppearance` (mixed, BB2020 + BB2025).
pub struct StepFoulAppearance {
    /// Java: `state.goToLabelOnFailure` (mandatory init param GOTO_LABEL_ON_FAILURE).
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Roll value (0 = not yet rolled)
    pub roll: i32,
}

impl StepFoulAppearance {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: resolve defender from TargetSelectionState (if selected+committed) or game.defender_id
        let defender_id = {
            let from_ts = game.field_model.target_selection_state.as_ref()
                .filter(|ts| ts.is_selected() && ts.is_committed())
                .and_then(|ts| ts.get_selected_player_id().cloned());
            from_ts.or_else(|| game.defender_id.clone())
        };
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
        //         if (source == null || !useReRoll) → handleFailure
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
            self.roll = 0;
        }

        // Java: roll = diceRoller.rollSkill() (1d6); minimumRoll = 2
        if self.roll == 0 {
            self.roll = rng.d6();
        }
        let minimum_roll = DiceInterpreter::minimum_roll_resisting_foul_appearance();
        let may_block = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);

        if may_block {
            // Java: step.commitTargetSelection() — calls targetSelectionState.commit() if not null
            if let Some(ref mut ts) = game.field_model.target_selection_state {
                ts.commit();
            }
            return StepOutcome::next();
        }

        // Failure — try re-roll if first failure
        if !already_rerolled {
            use ffb_model::model::re_rolled_action::ReRolledAction;
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("FOUL_APPEARANCE"));

            let pid = game.acting_player.player_id.clone();
            let skill_source = find_skill_reroll_source(game, "FOUL_APPEARANCE");
            if let Some(source) = skill_source {
                use_reroll(game, &source, pid.as_deref().unwrap_or(""));
                self.re_roll_state.re_roll_source = Some(source);
                self.roll = 0;
                return self.execute_step(game, rng);
            }

            // Java: if (reRolled || !askForReRollIfAvailable) → handleFailure
            // Note: BB2020 version only asks if NOT already reRolled
            if let Some(prompt) = ask_for_reroll_if_available(game, "FOUL_APPEARANCE", minimum_roll, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0;
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        self.fail_fa(game)
    }

    fn fail_fa(&mut self, game: &mut Game) -> StepOutcome {
        let player_action = game.acting_player.player_action;

        // Java: if (actingPlayer.isStandingUp() && (BLITZ_MOVE || blockAction || GAZE_MOVE || kickingDowned))
        //         setPlayerState(player, state.changeBase(PRONE).changeActive(false))
        if game.acting_player.standing_up {
            let set_prone = player_action.map(|pa|
                pa == PlayerAction::BlitzMove
                || pa.is_block_action()
                || pa == PlayerAction::GazeMove
                || pa.is_kicking_downed()
            ).unwrap_or(false);
            if set_prone {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    if let Some(state) = game.field_model.player_state(&pid) {
                        game.field_model.set_player_state(&pid, state.change_base(PS_PRONE).change_active(false));
                    }
                }
            }
        }

        game.acting_player.has_blocked = true;
        game.turn_data_mut().turn_started = true;

        // Java: targetSelectionState.failed(); if blitzing → blitzUsed = true
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.failed();
            if player_action.map(|pa| pa.is_blitzing()).unwrap_or(false) {
                game.turn_data_mut().blitz_used = true;
            }
        }

        // Java: if (GAZE || blockAction) → publishParameter(END_PLAYER_ACTION, true)
        let end_action = player_action.map(|pa|
            pa.is_gaze() || pa.is_block_action()
        ).unwrap_or(false);

        // Java: game.setDefenderId(null)
        game.defender_id = None;

        let label = self.goto_label_on_failure.clone();
        let out = StepOutcome::goto(&label);
        if end_action {
            out.publish(StepParameter::EndPlayerAction(true))
        } else {
            out
        }
    }
}

impl Default for StepFoulAppearance {
    fn default() -> Self { Self::new("") }
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, SkillId, PlayerType, PlayerGender, TurnMode, PlayerState};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, skills: Vec<SkillId>) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
});
    }

    #[test]
    fn id_is_foul_appearance() {
        assert_eq!(StepFoulAppearance::new("fail").id(), StepId::FoulAppearance);
    }

    #[test]
    fn no_defender_returns_next_step() {
        let mut step = StepFoulAppearance::new("fail");
        let mut game = make_game();
        game.defender_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn defender_without_foul_appearance_returns_next() {
        let mut game = make_game();
        add_player(&mut game, "def", vec![]);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn successful_roll_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        step.roll = 2; // success (>= 2)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn failed_roll_no_reroll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fa_fail"));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().turn_started);
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepFoulAppearance::new("old_label");
        let accepted = step.set_parameter(&StepParameter::GotoLabelOnFailure("new_label".into()));
        assert!(accepted);
        assert_eq!(step.goto_label_on_failure, "new_label");
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepFoulAppearance::new("fail");
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }

    #[test]
    fn failed_with_standing_up_blitz_move_sets_player_prone() {
        use ffb_model::enums::{PlayerAction, PS_PRONE, PS_STANDING};
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        game.acting_player.standing_up = true;
        game.acting_player.has_blocked = false;
        game.field_model.set_player_state("atk", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        // Player should be set to PRONE + inactive
        let state = game.field_model.player_state("atk").unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(!state.is_active());
    }

    #[test]
    fn failed_with_blitzing_action_sets_blitz_used_and_target_selection_failed() {
        use ffb_model::enums::PlayerAction;
        use ffb_model::model::target_selection_state::{TargetSelectionState, TargetSelectionStatus};
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.defender_id = Some("def".into());
        let ts = TargetSelectionState::new("def");
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data().blitz_used);
        assert_eq!(
            game.field_model.target_selection_state.as_ref().map(|ts| ts.status),
            Some(TargetSelectionStatus::FAILED)
        );
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn failed_with_block_action_publishes_end_player_action() {
        use ffb_model::enums::PlayerAction;
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        add_player(&mut game, "atk", vec![]);
        add_player(&mut game, "def", vec![SkillId::FoulAppearance]);
        game.acting_player.player_id = Some("atk".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.defender_id = Some("def".into());
        let mut step = StepFoulAppearance::new("fa_fail");
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }
}
