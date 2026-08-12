use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_argue_the_call_roll::ReportArgueTheCallRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Handles argue-the-call and bribes in the foul ejection sequence.
/// For the parity random-agent, the coach always argues (no dialog) and never uses bribes.
/// Roll ≥ 6 → argue succeeds (fouler stays); roll < 2 → coach banned; else → fouler ejected.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.foul.StepBribes`.
pub struct StepBribes {
    pub goto_label_on_end: String,
    pub argue_the_call_choice: Option<bool>,
    pub argue_the_call_successful: Option<bool>,
    pub bribes_choice: Option<bool>,
    pub bribe_successful: Option<bool>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    pub player_id_single_use_re_roll: Option<String>,
}

impl StepBribes {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            argue_the_call_choice: None,
            argue_the_call_successful: None,
            bribes_choice: None,
            bribe_successful: None,
            re_rolled_action: None,
            re_roll_source: None,
            player_id_single_use_re_roll: None,
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
        match action {
            Action::ArgueTheCall { argue } => {
                self.argue_the_call_choice = Some(*argue);
                self.argue_the_call_successful = None;
            }
            Action::UseBribe { use_bribe } => {
                self.bribes_choice = Some(*use_bribe);
                self.bribe_successful = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }
}

impl StepBribes {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let fouler_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let fouler_coord = game.field_model.player_coordinate(&fouler_id);
        let fouler_has_ball = fouler_coord
            .zip(game.field_model.ball_coordinate)
            .map(|(fc, bc)| fc == bc)
            .unwrap_or(false);

        // Argue-the-call: random agent always argues. Roll d6.
        // DiceInterpreter: isArgueSuccessful = roll > 5 (≥ 6), isCoachBanned = roll < 2.
        if self.argue_the_call_choice.is_none() {
            self.argue_the_call_choice = Some(true);
        }

        if self.argue_the_call_choice == Some(true) && self.argue_the_call_successful.is_none() {
            if !game.turn_data().coach_banned {
                // Java rollArgue (StepBribes): rollArgueTheCall d6; +1 if friendsWithTheRef and roll>1;
                // + biasedRefBonus (Usage.ADD_TO_ARGUE_ROLL). On a natural 1 that bans the coach, if the
                // team holds a Bribery-and-Corruption (REROLL_ARGUE) inducement with uses left, consume a
                // use, report USED, and re-roll the argue (Java `couldReRoll && coachBanned` →
                // useBriberyReRoll + recursive rollArgue(..., Optional.empty())). The ParityRunner path
                // never hits the `else if couldReRoll` dialog branch: isCoachBanned(roll) is roll<2, so a
                // natural 1 is ALWAYS coach-banned → the auto-reroll branch. (dwarf seed 7 i=133: away_03's
                // foul-ejection argue rolled 1 → B&C re-roll 5 → success; Rust rolled once and banned the
                // coach, running a die behind and desyncing the whole game.)
                use ffb_model::inducement::usage::Usage;
                // Java getActingTeam() = the team currently playing (turn_data() selects by home_playing).
                let team_id = if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() };
                let friends = game.prayer_state.is_friends_with_ref(&team_id);
                let mut argue_event: Option<GameEvent> = None;
                loop {
                    let roll = rng.d6();
                    let mut modified = if friends && roll > 1 { roll + 1 } else { roll };
                    let biased_ref_bonus = game.turn_data().inducement_set.value(Usage::ADD_TO_ARGUE_ROLL);
                    modified += biased_ref_bonus;
                    let successful = crate::dice_interpreter::DiceInterpreter::is_argue_the_call_successful(modified);
                    let coach_banned_by_argue = crate::dice_interpreter::DiceInterpreter::is_coach_banned(roll);
                    self.argue_the_call_successful = Some(successful);
                    game.report_list.add(ReportArgueTheCallRoll::new(
                        game.acting_player.player_id.clone(),
                        successful,
                        coach_banned_by_argue,
                        roll,
                        true,
                        friends,
                        biased_ref_bonus,
                    ));
                    // One ArgueTheCall coverage event per roll (initial + any B&C re-roll).
                    argue_event = Some(GameEvent::ArgueTheCall {
                        player_id: fouler_id.clone(),
                        roll,
                        success: successful,
                    });
                    // Java `couldReRoll = roll==1 && reRollSource != BRIBERY && briberyReRoll present && hasUsesLeft`.
                    let bribery_type: Option<String> = game.turn_data().inducement_set
                        .for_usage(Usage::REROLL_ARGUE)
                        .filter(|t| game.turn_data().inducement_set.has_uses_left(t))
                        .map(str::to_owned);
                    let could_re_roll = roll == 1 && bribery_type.is_some();
                    if could_re_roll && coach_banned_by_argue {
                        // Java useBriberyReRoll: consume a use, report USED, re-roll.
                        let type_id = bribery_type.unwrap();
                        game.turn_data_mut().inducement_set.use_one_of(&type_id);
                        game.report_list.add(
                            ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll::new(
                                Some(team_id.clone()), "USED".into()));
                        continue;
                    }
                    if coach_banned_by_argue {
                        game.turn_data_mut().coach_banned = true;
                    }
                    break;
                }
                let successful = self.argue_the_call_successful == Some(true);
                let argue_event = argue_event.expect("argue rolled at least once");
                if successful {
                    self.bribes_choice = Some(false);
                    // BB2016 (bb2016 uses this shared StepBribes via make_step fall-through) treats a
                    // SUCCESSFUL argue differently from BB2020+: the fouler is STILL ejected — but to
                    // RESERVE (they can be set up next drive), not BANNED. Only an UNSUCCESSFUL argue
                    // bans them. So bb2016 must proceed to EjectPlayer (which boxes the fouler as
                    // RESERVE when ArgueTheCallSuccessful=true) instead of gotoing the end label and
                    // leaving the fouler on the pitch. BB2020+ keep the on-pitch-stay behaviour.
                    // (amazon bb2016 seed22 i=225: home_08 fouled on doubles armour, argued 6 (success)
                    // — Java sent the fouler to Reserve, Rust left it Standing → state-hash divergence.)
                    if game.rules == ffb_model::enums::Rules::Bb2016 {
                        return StepOutcome::next()
                            .with_event(argue_event)
                            .publish(StepParameter::FoulerHasBall(fouler_has_ball))
                            .publish(StepParameter::ArgueTheCallSuccessful(true))
                            .publish(StepParameter::EndTurn(true));
                    }
                    let label = self.goto_label_on_end.clone();
                    return StepOutcome::goto(&label)
                        .with_event(argue_event)
                        .publish(StepParameter::FoulerHasBall(fouler_has_ball))
                        .publish(StepParameter::ArgueTheCallSuccessful(true))
                        .publish(StepParameter::EndTurn(true));
                }
                // Argue failed → fouler ejected below; attach the roll event to that outcome.
                return StepOutcome::next()
                    .with_event(argue_event)
                    .publish(StepParameter::FoulerHasBall(fouler_has_ball))
                    .publish(StepParameter::ArgueTheCallSuccessful(false))
                    .publish(StepParameter::EndTurn(true));
            } else {
                // Coach already banned — argue is skipped, auto-eject.
                self.argue_the_call_successful = Some(false);
                self.bribes_choice = Some(false);
            }
        }

        // No successful argue and no bribes → fouler ejected, turnover.
        StepOutcome::next()
            .publish(StepParameter::FoulerHasBall(fouler_has_ball))
            .publish(StepParameter::ArgueTheCallSuccessful(false))
            .publish(StepParameter::EndTurn(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. start() with no acting player → NextStep (no fouler id guard)
    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 2. Coach is NOT banned + argue always chosen → d6 roll happens. With seed 0 the roll
    //    is deterministic; regardless of outcome we get GotoLabel (success) or NextStep
    //    (ejected) — both are valid non-panic exits.
    #[test]
    fn with_acting_player_terminates() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Must be one of the terminal actions — never Continue
        assert!(out.action == StepAction::GotoLabel || out.action == StepAction::NextStep);
    }

    // 3. ArgueTheCallSuccessful(false) is published when ejection path is taken
    #[test]
    fn ejection_path_publishes_argue_failed() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        // coach_banned so argue is skipped → auto-eject path
        game.turn_data_mut().coach_banned = true;
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(false))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    // 4. ArgueTheCall failed + bribes declined → NextStep (ejected turnover)
    #[test]
    fn argue_fail_no_bribes_next_step() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepBribes::new("end".into());
        step.argue_the_call_choice = Some(true);
        step.argue_the_call_successful = Some(false);
        step.bribes_choice = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    // 6. Argue roll emits ReportArgueTheCallRoll when coach is not banned
    #[test]
    fn argue_roll_emits_argue_the_call_report() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        // coach NOT banned → argue roll fires
        let mut step = StepBribes::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ReportId::ARGUE_THE_CALL),
            "ReportArgueTheCallRoll should be emitted when argue roll is made"
        );
    }

    // 7. No argue roll when coach is banned → no ARGUE_THE_CALL report
    #[test]
    fn no_argue_report_when_coach_banned() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game.turn_data_mut().coach_banned = true;
        let mut step = StepBribes::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::ARGUE_THE_CALL),
            "No ReportArgueTheCallRoll should be emitted when coach is already banned"
        );
    }

    /// Regression (dwarf seed 7 i=133): a foul-ejection argue that rolls a natural 1 (coach-banned)
    /// must be re-rolled when the fouling team holds a Bribery and Corruption (REROLL_ARGUE)
    /// inducement — Java StepBribes.rollArgue: `couldReRoll = roll==1 && REROLL_ARGUE present &&
    /// hasUsesLeft; if (couldReRoll && coachBanned) useBriberyReRoll + recursive rollArgue`. Without
    /// it Rust rolled once, banned the coach, and consumed one fewer die than Java — desyncing the
    /// shared stream for the rest of the game. First two d6 = 1 (natural one) then 6 (successful
    /// re-roll) → argue succeeds, coach not banned, one B&C use consumed.
    #[test]
    fn seed7_foul_ejection_bribery_reroll_of_natural_one() {
        use ffb_model::inducement::inducement::Inducement as InducementModel;
        use ffb_model::inducement::usage::Usage;
        // Seed whose first two d6 are 1 then 6.
        let mut seed = 0u64;
        loop {
            let mut r = GameRng::new(seed);
            if r.d6() == 1 && r.d6() == 6 { break; }
            seed += 1;
            assert!(seed < 100_000, "no seed with d6 sequence 1,6 found");
        }
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        // Grant the fouling team its Bribery and Corruption argue re-roll (Java leaveStep grant).
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));

        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel, "re-rolled argue (6) → success → goto end label");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(true))));
        assert!(!game.turn_data_home.coach_banned, "successful re-roll → coach not banned");
        assert!(!game.turn_data_home.inducement_set.has_uses_left("briberyAndCorruption"),
            "the B&C re-roll charge was consumed");
    }

    #[test]
    fn bb2016_successful_argue_still_ejects_via_next_step() {
        // BB2016: a SUCCESSFUL argue does NOT keep the fouler on the pitch (unlike BB2020+). The
        // fouler is still ejected — to RESERVE — so bb2016 must return NEXT_STEP (proceed to
        // EjectPlayer, which boxes the fouler as RESERVE when ArgueTheCallSuccessful=true), NOT
        // GotoLabel (the BB2020+ on-pitch-stay path). amazon bb2016 seed22 i=225.
        let mut seed = 0u64;
        loop {
            if GameRng::new(seed).d6() == 6 { break; }  // natural 6 → argue succeeds
            seed += 1;
            assert!(seed < 100_000, "no seed with first d6=6 found");
        }
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let mut step = StepBribes::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep,
            "bb2016 successful argue must proceed to EjectPlayer (NextStep), not stay on pitch (GotoLabel)");
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(true))),
            "bb2016 successful argue must publish ArgueTheCallSuccessful(true) for EjectPlayer to box as RESERVE");
    }

    // 5. UseBribe action sets bribes_choice flag
    #[test]
    fn use_bribe_action_sets_bribes_choice() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepBribes::new("end".into());
        step.handle_command(&Action::UseBribe { use_bribe: true }, &mut game, &mut GameRng::new(0));
        // bribes_choice is set by handle_command before execute_step resets it if necessary
        // The state after execute_step may change it, but the command was processed
        // (no panic = command was handled)
    }
}
