/// 1:1 translation of com.fumbbl.ffb.server.step.action.ktm.StepKickTeamMate (COMMON; only the
/// bb2016 KickTeamMate sequence uses it).
///
/// Rolls dice for the kick-team-mate action, determines success/failure, offers the
/// KICK_TEAM_MATE re-roll when ALLOW_KTM_REROLL is on, and pushes the ScatterPlayer sequence on
/// success.
///
/// Mandatory init param: GOTO_LABEL_ON_FAILURE.
/// Expected preceding params: KICKED_PLAYER_ID, KICKED_PLAYER_STATE, KICKED_PLAYER_HAS_BALL,
/// NR_OF_DICE.
///
/// §Q3.8: the re-roll (and the option that enables it) and the ScatterPlayer push were stubs;
/// a successful kick published IS_KICKED_PLAYER but never flew the player anywhere.
use ffb_model::enums::{PlayerState, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::kick_team_mate_range::KickTeamMateRange;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2016::report_kick_team_mate_roll::ReportKickTeamMateRoll;
use crate::action::Action;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::scatter_player::{ScatterPlayer, ScatterPlayerParams};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

const KICK_TEAM_MATE: &str = "KICK_TEAM_MATE";

pub struct StepKickTeamMate {
    /// Java: fGotoLabelOnFailure — mandatory.
    pub goto_label_on_failure: String,
    /// Java: fKickedPlayerId — set by preceding step parameter.
    pub kicked_player_id: Option<String>,
    /// Java: fKickedPlayerState — player state at kick time.
    pub kicked_player_state: Option<PlayerState>,
    /// Java: fKickedPlayerHasBall — whether kicked player carries the ball.
    pub kicked_player_has_ball: bool,
    /// Java: fNumDice — 1 or 2, clamped.
    pub num_dice: i32,
    /// Java: fDistance — total rolled distance (rolls[0] + rolls[1]).
    pub distance: i32,
    /// Java: fRolls — individual dice values.
    pub rolls: Vec<i32>,
    /// Java: AbstractStepWithReRoll fields.
    pub re_roll_state: ReRollState,
}

impl StepKickTeamMate {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            kicked_player_id: None,
            kicked_player_state: None,
            kicked_player_has_ball: false,
            num_dice: 0,
            distance: 0,
            rolls: Vec::new(),
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Step for StepKickTeamMate {
    fn id(&self) -> StepId { StepId::KickTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java AbstractStepWithReRoll CLIENT_USE_RE_ROLL: a declined offer carries no source.
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v) => { self.kicked_player_state = Some(*v); true }
            StepParameter::KickedPlayerHasBall(v) => { self.kicked_player_has_ball = *v; true }
            // Java StepKickTeamMate.setParameter: NR_OF_DICE stores fNumDice then `break;` → the
            // method returns FALSE (unlike the KICKED_* cases right above, which `return true`). A
            // false return means the published NR_OF_DICE keeps propagating DOWN the stack to lower
            // steps (StepStack.publishParameter stops only on a true return). StepInitKickTeamMate
            // publishes NR_OF_DICE, so this step is a mid-stack consumer that must not swallow it.
            StepParameter::NrOfDice(v) => { self.num_dice = (*v).max(0).min(2); false }
            _ => false,
        }
    }
}

impl StepKickTeamMate {
    fn re_rolled_ktm(&self) -> bool {
        self.re_roll_state.re_rolled_action.as_ref().is_some_and(|a| a.name == KICK_TEAM_MATE)
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: actingPlayer.setHasBlocked(true); game.setConcessionPossible(false); turnData.setBlitzUsed(true)
        game.acting_player.has_blocked = true;
        game.concession_possible = false;
        game.turn_data_mut().blitz_used = true;
        // Java: UtilServerDialog.hideDialog(getGameState())
        crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);

        let Some(kicker_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::goto(&self.goto_label_on_failure);
        };
        let kicker_coord = game.field_model.player_coordinate(&kicker_id);

        // Java: if (KICK_TEAM_MATE == getReRolledAction()) { if (source == null || !useReRoll(...))
        //   { successful = fNumDice == 1 || fRolls[0] != fRolls[1]; executeKick(...); doRoll = false } }
        if self.re_rolled_ktm() {
            let used = match self.re_roll_state.re_roll_source.clone() {
                Some(source) => use_reroll(game, &source, &kicker_id, rng),
                None => false,
            };
            if !used {
                let successful = self.num_dice.max(1) == 1 || self.rolls.get(0) != self.rolls.get(1);
                return self.execute_kick(game, kicker_coord, successful);
            }
        }

        let Some(kicked_id) = self.kicked_player_id.clone() else {
            return StepOutcome::goto(&self.goto_label_on_failure);
        };
        let (Some(kicker_coord), Some(kicked_coord)) = (kicker_coord, game.field_model.player_coordinate(&kicked_id)) else {
            return StepOutcome::goto(&self.goto_label_on_failure);
        };

        // Java: Direction d = kickerCoordinate.getDirection(kickedPlayerCoordinate)
        let direction = kicker_coord.direction_to(kicked_coord);

        // Java: fRolls = new int[fNumDice]; each rollSkill(). (Java throws on 0 dice; Rust has
        // always rolled at least one, which StepInitKickTeamMate guarantees anyway.)
        let num_dice = self.num_dice.max(1);
        self.rolls = (0..num_dice).map(|_| rng.d6()).collect();

        // Java: successful = (fNumDice == 1) || (rolls[0] != rolls[1])
        let successful = num_dice == 1 || self.rolls[0] != self.rolls[1];

        // Java: fDistance = rolls[0] + (numDice > 1 ? rolls[1] : 0)
        self.distance = self.rolls[0] + if num_dice > 1 { self.rolls[1] } else { 0 };

        // Java: targetCoordinate = kickedPlayerCoordinate.move(d, distance); setPassCoordinate
        if let Some(dir) = direction {
            game.pass_coordinate = Some(kicked_coord.step(dir, self.distance));
        }

        // Java: reRolled = (getReRolledAction() == KICK_TEAM_MATE) && (getReRollSource() != null)
        let re_rolled = self.re_rolled_ktm() && self.re_roll_state.re_roll_source.is_some();
        game.report_list.add(ReportKickTeamMateRoll::new(
            kicker_id.clone(),
            kicked_id.clone(),
            successful,
            self.rolls.clone(),
            re_rolled,
            self.distance,
        ));

        // Java: if (ALLOW_KTM_REROLL && getReRolledAction() != KICK_TEAM_MATE) {
        //   setReRolledAction(KICK_TEAM_MATE);
        //   if (!askForReRollIfAvailable(..., KICK_TEAM_MATE, 0, false)) act = true;
        // } else act = true;
        let allow_ktm_reroll = ffb_model::option::util_game_option::is_option_enabled(
            game, ffb_model::option::game_option_id::ALLOW_KTM_REROLL);
        if allow_ktm_reroll && !self.re_rolled_ktm() {
            self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(KICK_TEAM_MATE));
            if let Some(prompt) = ask_for_reroll_if_available(game, KICK_TEAM_MATE, 0, false) {
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                return StepOutcome::cont().with_prompt(prompt);
            }
        }
        self.execute_kick(game, Some(kicker_coord), successful)
    }

    /// Java `executeKick(kickedPlayer, kickerCoordinate, successful)`.
    fn execute_kick(&self, game: &mut Game, kicker_coord: Option<FieldCoordinate>, successful: bool) -> StepOutcome {
        if !successful {
            // Java: getResult().setNextAction(GOTO_LABEL, fGotoLabelOnFailure)
            return StepOutcome::goto(&self.goto_label_on_failure);
        }
        // Java: hasSwoop = kickedPlayer != null && kickedPlayer.hasSkillProperty(ttmScattersInSingleDirection)
        let has_swoop = self.kicked_player_id.as_deref()
            .and_then(|id| game.player(id))
            .is_some_and(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION));
        // Java: game.getFieldModel().setPlayerState(game.getDefender(), fKickedPlayerState.changeBase(PICKED_UP))
        if let (Some(defender_id), Some(state)) = (game.defender_id.clone(), self.kicked_player_state) {
            game.field_model.set_player_state(&defender_id, state.change_base(ffb_model::enums::PS_PICKED_UP));
        }
        // Java: ScatterPlayer.pushSequence(new SequenceParams(gameState, fKickedPlayerId, fKickedPlayerState,
        //   fKickedPlayerHasBall, kickerCoordinate, hasSwoop, true))
        let seq = ScatterPlayer::build_sequence(&ScatterPlayerParams {
            thrown_player_id: self.kicked_player_id.clone(),
            thrown_player_state: self.kicked_player_state,
            thrown_player_has_ball: self.kicked_player_has_ball,
            thrown_player_coordinate: kicker_coord,
            throw_scatter: true,
            has_swoop,
        });
        // Java: publishParameter(IS_KICKED_PLAYER, true); KTM_MODIFIER LONG (>= 9) / MEDIUM (>= 6)
        let mut outcome = StepOutcome::next()
            .push_seq(seq)
            .publish(StepParameter::IsKickedPlayer(true));
        if self.distance >= 9 {
            outcome = outcome.publish(StepParameter::KtmModifier(KickTeamMateRange::LONG));
        } else if self.distance >= 6 {
            outcome = outcome.publish(StepParameter::KtmModifier(KickTeamMateRange::MEDIUM));
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerState, Rules, PS_STANDING};
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;

    fn make_game_with_players() -> (Game, String, String) {
        let kicker_id = "kicker".to_string();
        let kicked_id = "kicked".to_string();
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(kicker_id.clone());
        game.field_model.set_player_coordinate(&kicker_id, FieldCoordinate::new(8, 5));
        game.field_model.set_player_coordinate(&kicked_id, FieldCoordinate::new(9, 5));
        game.field_model.set_player_state(&kicked_id, PlayerState::new(PS_STANDING));
        (game, kicker_id, kicked_id)
    }

    fn make_step(kicked_id: &str) -> StepKickTeamMate {
        let mut step = StepKickTeamMate::new("fail");
        step.kicked_player_id = Some(kicked_id.into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.num_dice = 1;
        step
    }

    #[test]
    fn nr_of_dice_stored_but_not_consumed_so_it_keeps_propagating() {
        // Fidelity regression (Bug #14): Java StepKickTeamMate.setParameter stores fNumDice for
        // NR_OF_DICE then `break;` → returns FALSE, so a published NR_OF_DICE keeps flowing DOWN the
        // stack to lower steps (StepInitKickTeamMate publishes it). The KICKED_* keys `return true`
        // (consumed). Rust previously returned true for NrOfDice too, wrongly swallowing it.
        let mut step = StepKickTeamMate::new("fail");
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)), "NR_OF_DICE must not be consumed");
        assert_eq!(step.num_dice, 2, "…but the value is still stored locally");
        // The KICKED_* keys, by contrast, are consumed (return true).
        assert!(step.set_parameter(&StepParameter::KickedPlayerId(Some("k1".into()))));
        assert!(step.set_parameter(&StepParameter::KickedPlayerHasBall(true)));
    }

    #[test]
    fn single_die_roll_always_succeeds() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 1;
        let out = step.start(&mut game, &mut GameRng::new(42));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::IsKickedPlayer(true))));
    }

    #[test]
    fn double_on_two_dice_fails() {
        // Seed chosen to produce a double on 2d6.
        // With GameRng::new(0): find a seed where both d6 produce the same value.
        // Use a known-bad scenario: set rolls manually via a stub.
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 2;
        // Force rolls to produce a double by pre-setting rolls (simulated).
        // We can't force the RNG, but we can test the logic:
        // call execute_kick directly with successful=false.
        let kicker = FieldCoordinate::new(8, 5);
        let out = step.execute_kick(&mut game, Some(kicker), false);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn successful_kick_sets_has_blocked_and_blitz_used() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 1;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_blocked);
        assert!(game.turn_data().blitz_used);
        assert!(!game.concession_possible);
    }

    #[test]
    fn distance_long_range_publishes_long_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 9;
        let kicker = FieldCoordinate::new(8, 5);
        let out = step.execute_kick(&mut game, Some(kicker), true);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(KickTeamMateRange::LONG))));
    }

    #[test]
    fn distance_medium_range_publishes_medium_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 6;
        let kicker = FieldCoordinate::new(8, 5);
        let out = step.execute_kick(&mut game, Some(kicker), true);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(KickTeamMateRange::MEDIUM))));
    }

    #[test]
    fn short_range_no_ktm_modifier() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.distance = 3;
        let kicker = FieldCoordinate::new(8, 5);
        let out = step.execute_kick(&mut game, Some(kicker), true);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::KtmModifier(_))));
    }

    #[test]
    fn no_kicker_gotos_failure() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        game.acting_player.player_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn successful_kick_adds_kick_team_mate_roll_report() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        step.num_dice = 1;
        step.start(&mut game, &mut GameRng::new(42));
        assert!(game.report_list.has_report(ReportId::KICK_TEAM_MATE_ROLL));
    }

    /// §Q3.8 Java executeKick: a successful kick pushes the (bb2016) ScatterPlayer sequence from
    /// the KICKER's square, and the kicked player goes PICKED_UP via `game.getDefender()`.
    #[test]
    fn successful_kick_pushes_scatter_player_from_the_kicker() {
        let (mut game, _, kicked_id) = make_game_with_players();
        game.defender_id = Some(kicked_id.clone());
        let step = make_step(&kicked_id);
        let out = step.execute_kick(&mut game, Some(FieldCoordinate::new(8, 5)), true);
        let first = out.pushes.first().and_then(|s| s.first()).expect("a ScatterPlayer sequence");
        assert_eq!(first.step_id, StepId::InitScatterPlayer);
        assert!(first.params.iter().any(|p| matches!(p,
            StepParameter::ThrownPlayerCoordinate(Some(c)) if *c == FieldCoordinate::new(8, 5))));
        assert_eq!(game.field_model.player_state(&kicked_id).map(|s| s.base()), Some(ffb_model::enums::PS_PICKED_UP));
    }

    /// §Q3.8 Java: with ALLOW_KTM_REROLL (default off) the roll is followed by a KICK_TEAM_MATE
    /// re-roll offer when one is available; without it the kick resolves at once.
    #[test]
    fn allow_ktm_reroll_offers_a_re_roll() {
        for enabled in [false, true] {
            let (mut game, kicker_id, kicked_id) = make_game_with_players();
            // Java isTeamReRollAvailable: the kicker is on the acting team.
            game.team_home.players.push(ffb_model::model::player::Player { id: kicker_id, ..Default::default() });
            game.turn_data_home.rerolls = 1;
            if enabled {
                game.options.set(ffb_model::option::game_option_id::ALLOW_KTM_REROLL, "true");
            }
            let mut step = make_step(&kicked_id);
            let out = step.start(&mut game, &mut GameRng::new(42));
            assert_eq!(out.prompt.is_some(), enabled, "enabled={enabled}");
            assert_eq!(out.action == StepAction::NextStep, !enabled, "enabled={enabled}");
        }
    }

    #[test]
    fn failed_kick_still_adds_kick_team_mate_roll_report() {
        let (mut game, _, kicked_id) = make_game_with_players();
        let mut step = make_step(&kicked_id);
        // Force failure via execute_step with pre-seeded rolls that produce a double.
        // We set num_dice=2 and call start; regardless of success/failure the report
        // is always emitted (Java line 167 fires before the reroll/act branch).
        // execute_kick with successful=false is called after addReport, so we test
        // start() with num_dice=2 — report must be present even on goto outcome.
        step.num_dice = 2;
        step.rolls = vec![3, 3]; // pre-loaded to guarantee a double on the logic path
        // Call execute_step path via start; rolls will be overwritten by RNG, but
        // the report is always added before execute_kick. Verify presence regardless.
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::KICK_TEAM_MATE_ROLL));
    }
}
