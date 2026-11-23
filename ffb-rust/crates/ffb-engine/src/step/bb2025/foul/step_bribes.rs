use ffb_model::dialog::dialog_argue_the_call_parameter::DialogArgueTheCallParameter;
use ffb_model::dialog::dialog_bribery_and_corruption_parameter::DialogBriberyAndCorruptionParameter;
use ffb_model::dialog::dialog_bribes_parameter::DialogBribesParameter;
use ffb_model::dialog::dialog_parameter_factory::DialogParameter;
use ffb_model::enums::Rules;
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_argue_the_call_roll::ReportArgueTheCallRoll;
use ffb_model::report::report_bribes_roll::ReportBribesRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::util_server_dialog::UtilServerDialog;

/// 1:1 port of Java `StepBribes` -- argue the call and bribes after a foul ejection. One Rust step
/// serves all three editions (`make_step` has no per-edition route for BRIBES):
/// `bb2020/bb2025.foul.StepBribes` (argue first, Bribery-and-Corruption argue re-roll, then
/// bribes; END_TURN published) and `bb2016.foul.StepBribes` (bribes first, then argue). The
/// coach answers `DialogArgueTheCallParameter` with CLIENT_ARGUE_THE_CALL (`Action::ArgueTheCall`),
/// `DialogBribesParameter` with CLIENT_USE_INDUCEMENT (`Action::UseInducement`) and
/// `DialogBriberyAndCorruptionParameter` with CLIENT_USE_RE_ROLL (`Action::UseReRoll`).
pub struct StepBribes {
    pub goto_label_on_end: String,
    pub argue_the_call_choice: Option<bool>,
    pub argue_the_call_successful: Option<bool>,
    pub bribes_choice: Option<bool>,
    pub bribe_successful: Option<bool>,
    /// AbstractStepWithReRoll `getReRolledAction() == ARGUE_THE_CALL` (the B&C dialog was answered).
    pub re_rolled_argue: bool,
    /// AbstractStepWithReRoll `getReRollSource() == BRIBERY_AND_CORRUPTION`.
    pub re_roll_source_bribery: bool,
    /// Java publishes FOULER_HAS_BALL when the argue is first asked; kept for the step's exit.
    fouler_has_ball: Option<bool>,
    /// The dialog this execution left open (None once the step moved on).
    prompt: Option<AgentPrompt>,
    events: Vec<GameEvent>,
}

impl StepBribes {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            argue_the_call_choice: None,
            argue_the_call_successful: None,
            bribes_choice: None,
            bribe_successful: None,
            re_rolled_argue: false,
            re_roll_source_bribery: false,
            fouler_has_ball: None,
            prompt: None,
            events: Vec::new(),
        }
    }
}

impl Step for StepBribes {
    fn id(&self) -> StepId { StepId::Bribes }

    // Java: init(StepParameterSet) reads GOTO_LABEL_ON_END (mandatory init param). Without
    // accepting it the label stayed empty and the success path did goto(""), silently
    // draining the whole step stack and ending the game mid-foul.
    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let acting = game.acting_player.player_id.clone().unwrap_or_default();
        match action {
            // CLIENT_ARGUE_THE_CALL: `fArgueTheCallChoice = command.hasPlayerId(actingPlayerId)`.
            Action::ArgueTheCall { player_ids } => {
                self.argue_the_call_choice = Some(player_ids.contains(&acting));
                self.argue_the_call_successful = None;
            }
            // CLIENT_USE_INDUCEMENT: only an AVOID_BAN inducement answers the bribes dialog.
            Action::UseInducement { inducement_type, player_ids, .. } => {
                let avoid_ban = inducement_type.as_deref()
                    .and_then(|t| game.turn_data().inducement_set.get(t))
                    .is_some_and(|i| i.has_usage(Usage::AVOID_BAN));
                if avoid_ban {
                    self.bribes_choice = Some(player_ids.contains(&acting));
                    self.bribe_successful = None;
                }
            }
            // AbstractStepWithReRoll CLIENT_USE_RE_ROLL from the Bribery-and-Corruption dialog.
            Action::UseReRoll { use_reroll } => {
                self.re_rolled_argue = true;
                self.re_roll_source_bribery = *use_reroll;
            }
            // Java: UNHANDLED_COMMAND -- the step keeps waiting with its dialog.
            _ => return self.waiting(),
        }
        self.execute_step(game, rng)
    }
}

impl StepBribes {
    fn waiting(&self) -> StepOutcome {
        match &self.prompt {
            Some(p) => StepOutcome::cont().with_prompt(p.clone()),
            None => StepOutcome::cont(),
        }
    }

    fn acting_team_id(game: &Game) -> String {
        if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        UtilServerDialog::hide_dialog(game);
        self.prompt = None;
        let outcome = if game.rules == Rules::Bb2016 {
            self.execute_step_bb2016(game, rng)
        } else {
            self.execute_step_bb2020(game, rng)
        };
        let mut outcome = match outcome {
            Some(o) => o,
            None => self.waiting(),
        };
        for ev in self.events.drain(..) {
            outcome = outcome.with_event(ev);
        }
        outcome
    }

    /// A successful bribe: GOTO the end label (FOULER_HAS_BALL is already published in Java).
    fn goto_end(&self) -> StepOutcome {
        let label = self.goto_label_on_end.clone();
        let out = StepOutcome::goto(&label);
        match self.fouler_has_ball {
            Some(b) => out.publish(StepParameter::FoulerHasBall(b)),
            None => out,
        }
    }

    /// The exit both editions share once argue and bribes are decided.
    fn finish(&self, successful: bool, end_turn: bool, goto_on_success: bool) -> StepOutcome {
        let mut out = if successful && goto_on_success {
            let label = self.goto_label_on_end.clone();
            StepOutcome::goto(&label)
        } else {
            StepOutcome::next()
        };
        if let Some(has_ball) = self.fouler_has_ball {
            out = out.publish(StepParameter::FoulerHasBall(has_ball));
        }
        out = out.publish(StepParameter::ArgueTheCallSuccessful(successful));
        if end_turn {
            out = out.publish(StepParameter::EndTurn(true));
        }
        out
    }

    fn fouler_has_ball(game: &Game) -> bool {
        let fouler = game.acting_player.player_id.as_deref();
        let coord = fouler.and_then(|id| game.field_model.player_coordinate(id));
        coord.is_some() && coord == game.field_model.ball_coordinate
    }

    /// Java bb2020 / bb2025 `StepBribes.executeStep`.
    fn execute_step_bb2020(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<StepOutcome> {
        let Some(fouler_id) = game.acting_player.player_id.clone() else { return Some(StepOutcome::next()) };
        if self.bribes_choice.is_none() {
            let team_id = Self::acting_team_id(game);
            let friends = game.prayer_state.is_friends_with_ref(&team_id);
            if self.argue_the_call_choice.is_none() {
                self.fouler_has_ball = Some(Self::fouler_has_ball(game));
                let biased_ref_bonus = game.turn_data().inducement_set.value(Usage::ADD_TO_ARGUE_ROLL);
                self.ask_for_argue_the_call(game, &fouler_id, friends, biased_ref_bonus);
            }
            if self.argue_the_call_choice == Some(true) && (!self.re_rolled_argue || self.re_roll_source_bribery) {
                let bribery = game.turn_data().inducement_set.for_usage(Usage::REROLL_ARGUE).map(str::to_owned);
                if self.re_roll_source_bribery {
                    if let Some(t) = bribery.as_deref() {
                        if game.turn_data().inducement_set.has_uses_left(t) {
                            Self::use_bribery_re_roll(game, t, &team_id);
                        }
                    }
                }
                if self.roll_argue(game, rng, &fouler_id, &team_id, friends, bribery) {
                    return None;
                }
            }
            if !self.re_roll_source_bribery && self.re_rolled_argue {
                // Java: getReRollSource() == null && getReRolledAction() == ARGUE_THE_CALL
                self.bribes_choice = Some(false);
                game.turn_data_mut().coach_banned = true;
            }
        }
        if self.argue_the_call_choice.is_some() && self.argue_the_call_successful != Some(true) {
            if self.bribes_choice.is_none() {
                self.ask_for_bribes(game, &fouler_id);
            }
            if self.bribes_choice == Some(true) && self.bribe_successful.is_none() {
                self.roll_bribes(game, rng, &fouler_id, true);
            }
            if self.bribe_successful == Some(true) {
                return Some(self.goto_end());
            }
        }
        if self.bribes_choice.is_some() && self.argue_the_call_choice.is_some() {
            let successful = self.argue_the_call_successful == Some(true);
            return Some(self.finish(successful, true, true));
        }
        None
    }

    /// Java bb2016 `StepBribes.executeStep`: bribes first, then argue; a successful argue still
    /// ejects the fouler (EjectPlayer boxes him as RESERVE) and no END_TURN is published here.
    fn execute_step_bb2016(&mut self, game: &mut Game, rng: &mut GameRng) -> Option<StepOutcome> {
        let Some(fouler_id) = game.acting_player.player_id.clone() else { return Some(StepOutcome::next()) };
        if self.bribes_choice.is_none() {
            self.ask_for_bribes(game, &fouler_id);
        }
        if self.bribes_choice == Some(true) && self.bribe_successful.is_none() {
            self.roll_bribes(game, rng, &fouler_id, false);
        }
        if self.bribe_successful == Some(true) {
            return Some(self.goto_end());
        }
        if self.bribes_choice.is_some() && self.argue_the_call_choice.is_none() {
            self.fouler_has_ball = Some(Self::fouler_has_ball(game));
            self.argue_the_call_choice = Some(false);
            if is_option_enabled(game, game_option_id::ARGUE_THE_CALL) && !game.turn_data().coach_banned {
                let team_id = Self::acting_team_id(game);
                self.show_argue_dialog(game, &team_id, &fouler_id, false, false, 0);
            }
        }
        if self.bribes_choice.is_some() && self.argue_the_call_choice == Some(true) {
            let roll = rng.d6();
            let successful = DiceInterpreter::is_argue_the_call_successful(roll);
            let coach_banned = DiceInterpreter::is_coach_banned(roll);
            self.argue_the_call_successful = Some(successful);
            game.report_list.add(
                ffb_model::report::bb2016::report_argue_the_call_roll::ReportArgueTheCallRoll::new(
                    fouler_id.clone(), successful, coach_banned, roll));
            self.events.push(GameEvent::ArgueTheCall { player_id: fouler_id.clone(), roll, success: successful });
            if coach_banned {
                game.turn_data_mut().coach_banned = true;
            }
        }
        if self.bribes_choice.is_some() && self.argue_the_call_choice.is_some() {
            let successful = self.argue_the_call_successful == Some(true);
            // The success path proceeds to EjectPlayer too (which boxes the fouler as RESERVE).
            return Some(self.finish(successful, true, false));
        }
        None
    }

    /// Java `askForArgueTheCall` (bb2020/bb2025): the dialog only for a non-casualty fouler, with
    /// the ARGUE_THE_CALL option on and the coach not banned; otherwise the choice is `false`.
    fn ask_for_argue_the_call(&mut self, game: &mut Game, fouler_id: &str, friends: bool, biased_ref_bonus: i32) {
        self.argue_the_call_choice = Some(false);
        let was_cased = game.field_model.player_state(fouler_id).is_some_and(|s| s.is_casualty());
        if is_option_enabled(game, game_option_id::ARGUE_THE_CALL) && !game.turn_data().coach_banned && !was_cased {
            let team_id = Self::acting_team_id(game);
            self.show_argue_dialog(game, &team_id, fouler_id, true, friends, biased_ref_bonus);
        }
    }

    fn show_argue_dialog(&mut self, game: &mut Game, team_id: &str, fouler_id: &str, stay_on_pitch: bool, friends: bool, biased_refs: i32) {
        UtilServerDialog::show_dialog(game, DialogParameter::ArgueTheCall(DialogArgueTheCallParameter {
            team_id: Some(team_id.to_string()),
            player_ids: vec![fouler_id.to_string()],
            stay_on_pitch,
            friends_with_the_ref: friends,
            biased_refs,
        }), false);
        self.argue_the_call_choice = None;
        self.prompt = Some(AgentPrompt::ArgueTheCall { team_id: team_id.to_string(), player_ids: vec![fouler_id.to_string()] });
    }

    /// Java `askForBribes`: the dialog only when the acting team holds an AVOID_BAN inducement
    /// with a use left; otherwise the choice is `false`.
    fn ask_for_bribes(&mut self, game: &mut Game, fouler_id: &str) {
        self.bribes_choice = Some(false);
        let set = &game.turn_data().inducement_set;
        let bribe_type = set.get_inducements().into_iter()
            .find(|i| i.has_usage(Usage::AVOID_BAN) && set.has_uses_left(&i.type_id))
            .map(|i| i.type_id);
        if let Some(inducement_type) = bribe_type {
            let team_id = Self::acting_team_id(game);
            let mut dialog = DialogBribesParameter::new_with(team_id.clone(), 1);
            dialog.player_ids.push(fouler_id.to_string());
            UtilServerDialog::show_dialog(game, DialogParameter::Bribes(dialog), false);
            self.bribes_choice = None;
            self.prompt = Some(AgentPrompt::Bribes {
                team_id,
                inducement_type,
                player_ids: vec![fouler_id.to_string()],
                max_nr_of_bribes: 1,
            });
        }
    }

    /// Java: use one AVOID_BAN inducement, `rollBribes`, report; after a failed bribe ask again
    /// (bb2016 always, bb2020/bb2025 unless ONLY_ONE_BRIBE_PER_SEND_OFF).
    fn roll_bribes(&mut self, game: &mut Game, rng: &mut GameRng, fouler_id: &str, check_one_bribe_option: bool) {
        let bribe_type = game.turn_data().inducement_set.for_usage(Usage::AVOID_BAN).map(str::to_owned);
        if let Some(t) = bribe_type {
            if game.turn_data_mut().inducement_set.use_one_of(&t) {
                let roll = rng.d6();
                let successful = DiceInterpreter::is_bribes_successful(roll);
                self.bribe_successful = Some(successful);
                game.report_list.add(ReportBribesRoll::new(fouler_id.to_string(), successful, roll));
                let only_one = check_one_bribe_option
                    && is_option_enabled(game, game_option_id::ONLY_ONE_BRIBE_PER_SEND_OFF);
                if !only_one && !successful {
                    self.ask_for_bribes(game, fouler_id);
                }
            }
        }
    }

    /// Java `useBriberyReRoll`: consume a use, report USED.
    fn use_bribery_re_roll(game: &mut Game, type_id: &str, team_id: &str) {
        game.turn_data_mut().inducement_set.use_one_of(type_id);
        game.report_list.add(
            ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll::new(
                Some(team_id.to_string()), "USED".into()));
    }

    /// Java `rollArgue` (bb2020/bb2025): returns true when the B&C dialog was shown.
    fn roll_argue(&mut self, game: &mut Game, rng: &mut GameRng, fouler_id: &str, team_id: &str, friends: bool, mut bribery: Option<String>) -> bool {
        loop {
            let roll = rng.d6();
            let mut modified = if friends && roll > 1 { roll + 1 } else { roll };
            let biased_ref_bonus = game.turn_data().inducement_set.value(Usage::ADD_TO_ARGUE_ROLL);
            modified += biased_ref_bonus;
            let successful = DiceInterpreter::is_argue_the_call_successful(modified);
            self.argue_the_call_successful = Some(successful);
            if successful {
                self.bribes_choice = Some(false);
            }
            // bb2020: `isCoachBanned(modifiedRoll)`; bb2025: `isCoachBanned(roll)`.
            let coach_banned = DiceInterpreter::is_coach_banned(if game.rules == Rules::Bb2020 { modified } else { roll });
            game.report_list.add(ReportArgueTheCallRoll::new(
                Some(fouler_id.to_string()), successful, coach_banned, roll, true, friends, biased_ref_bonus));
            self.events.push(GameEvent::ArgueTheCall { player_id: fouler_id.to_string(), roll, success: successful });
            let could_re_roll = roll == 1 && !self.re_roll_source_bribery
                && bribery.as_deref().is_some_and(|t| game.turn_data().inducement_set.has_uses_left(t));
            if could_re_roll && coach_banned {
                Self::use_bribery_re_roll(game, bribery.as_deref().unwrap(), team_id);
                bribery = None; // Java: rollArgue(..., Optional.empty())
                continue;
            } else if could_re_roll {
                UtilServerDialog::show_dialog(game, DialogParameter::BriberyAndCorruption(
                    DialogBriberyAndCorruptionParameter { team_id: Some(team_id.to_string()) }), false);
                self.prompt = Some(AgentPrompt::BriberyAndCorruption { team_id: team_id.to_string() });
                return true;
            } else if coach_banned {
                if game.rules == Rules::Bb2020 {
                    self.bribes_choice = Some(false);
                }
                game.turn_data_mut().coach_banned = true;
            }
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::inducement::inducement::Inducement as InducementModel;
    use ffb_model::types::FieldCoordinate;

    fn make_game_with(rules: Rules) -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, rules);
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game
    }

    fn make_game() -> Game { make_game_with(Rules::Bb2025) }

    fn argue(step: &mut StepBribes, game: &mut Game, rng: &mut GameRng, yes: bool) -> StepOutcome {
        let player_ids = if yes { vec!["p1".to_string()] } else { vec![] };
        step.handle_command(&Action::ArgueTheCall { player_ids }, game, rng)
    }

    fn seed_with_d6s(want: &[i32]) -> u64 {
        let mut seed = 0u64;
        loop {
            let mut r = GameRng::new(seed);
            if want.iter().all(|w| r.d6() == *w) { return seed; }
            seed += 1;
            assert!(seed < 200_000, "no seed with d6 sequence {want:?}");
        }
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        game.acting_player.player_id = None;
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// Java askForArgueTheCall: the coach is asked (DialogArgueTheCallParameter) and the step waits.
    #[test]
    fn start_asks_the_coach_to_argue_and_waits() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ArgueTheCall { ref player_ids, .. }) if player_ids == &vec!["p1".to_string()]));
        assert!(matches!(game.dialog_parameter, Some(DialogParameter::ArgueTheCall(_))));
        assert_eq!(rng.call_count, 0, "no die before the coach answers");
        assert!(!game.report_list.has_report(ReportId::ARGUE_THE_CALL));
    }

    #[test]
    fn arguing_rolls_and_ends_the_step() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let out = argue(&mut step, &mut game, &mut rng, true);
        assert!(out.action == StepAction::GotoLabel || out.action == StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::ARGUE_THE_CALL));
        assert!(game.dialog_parameter.is_none(), "hideDialog on the answer");
    }

    #[test]
    fn declining_to_argue_ejects_without_a_roll() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let out = argue(&mut step, &mut game, &mut rng, false);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(rng.call_count, 0);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(false))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn coach_banned_skips_the_argue_and_ejects() {
        let mut game = make_game();
        game.turn_data_mut().coach_banned = true;
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(false))));
        assert!(!game.report_list.has_report(ReportId::ARGUE_THE_CALL));
    }

    /// Regression (dwarf seed 7 i=133): a natural 1 with a Bribery and Corruption (REROLL_ARGUE)
    /// inducement is re-rolled at once (Java `couldReRoll && coachBanned`).
    #[test]
    fn seed7_foul_ejection_bribery_reroll_of_natural_one() {
        let seed = seed_with_d6s(&[1, 6]);
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        let out = argue(&mut step, &mut game, &mut rng, true);
        assert_eq!(out.action, StepAction::GotoLabel, "re-rolled argue (6) → success → goto end label");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(true))));
        assert!(!game.turn_data_home.coach_banned, "successful re-roll → coach not banned");
        assert!(!game.turn_data_home.inducement_set.has_uses_left("briberyAndCorruption"));
    }

    /// Java askForBribes after a failed argue: a team with a bribe is asked; declining ejects.
    #[test]
    fn failed_argue_offers_the_bribe_and_declining_ejects() {
        let seed = seed_with_d6s(&[3]);
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(InducementModel::new("bribes", 1, vec![Usage::AVOID_BAN]));
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        let out = argue(&mut step, &mut game, &mut rng, true);
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::Bribes { .. })));
        assert!(matches!(game.dialog_parameter, Some(DialogParameter::Bribes(_))));
        let out = step.handle_command(&Action::UseInducement {
            inducement_type: Some("bribes".into()), card_id: None, player_ids: vec![] }, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.turn_data_home.inducement_set.has_uses_left("bribes"), "declined: no bribe used");
    }

    #[test]
    fn a_successful_bribe_keeps_the_fouler() {
        let seed = seed_with_d6s(&[3, 6]);
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(InducementModel::new("bribes", 1, vec![Usage::AVOID_BAN]));
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        argue(&mut step, &mut game, &mut rng, true);
        let out = step.handle_command(&Action::UseInducement {
            inducement_type: Some("bribes".into()), card_id: None, player_ids: vec!["p1".into()] }, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.report_list.has_report(ReportId::BRIBES_ROLL));
        assert!(!game.turn_data_home.inducement_set.has_uses_left("bribes"));
    }

    /// BB2016: a SUCCESSFUL argue still ejects the fouler (to RESERVE via EjectPlayer), so the
    /// step proceeds with NEXT_STEP. amazon bb2016 seed22 i=225.
    #[test]
    fn bb2016_successful_argue_still_ejects_via_next_step() {
        let seed = seed_with_d6s(&[6]);
        let mut game = make_game_with(Rules::Bb2016);
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(seed);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.prompt, Some(AgentPrompt::ArgueTheCall { .. })));
        let out = argue(&mut step, &mut game, &mut rng, true);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(true))));
    }

    #[test]
    fn an_unrelated_command_keeps_the_dialog_waiting() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ArgueTheCall { .. })));
        assert_eq!(rng.call_count, 0);
    }
}
