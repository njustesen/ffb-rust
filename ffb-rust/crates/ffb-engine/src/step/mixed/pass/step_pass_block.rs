/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepPassBlock`.
///
/// Handles the BB2020/BB2025 pass-interference window ("On The Ball",
/// `canMoveWhenOpponentPasses`): when a pass action is announced, the opposing
/// team's eligible players may each move up to 3 squares before the pass
/// resolves. Java switches the game into `TurnMode::PassBlock`, flips
/// `homePlaying` to the defending team, marks only the eligible pass blockers
/// ACTIVE (saving every on-pitch defender's state), and runs Select sequences
/// until no blocker is active — then restores the saved states, hands control
/// back to the thrower and continues the pass sequence.
///
/// Java keeps the SAME step instance on the stack (`pushCurrentStepOnStack()`);
/// the Rust stack re-creates steps from `SequenceStep` descriptors, so the
/// instance state (`fOldTurnMode`/`fOldPlayerStates`/`currentMove`/
/// `isGoingForIt`) travels via `StepParameter::PassBlockResume` on the
/// re-pushed descriptor instead.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END.
/// Incoming parameters: END_PLAYER_ACTION, END_TURN — consumed while the
/// pass-block mini-turn runs (Java consumes on `TurnMode == PASS_BLOCK`; the
/// Rust re-pushed continuation instance is exactly the one carrying
/// `PassBlockResume`, so it consumes and the fresh instance in the pass
/// sequence does not).
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStep`.
use ffb_model::enums::{PlayerAction, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_pass_block::ReportPassBlock;
use crate::action::Action;
use crate::step::framework::{
    PassBlockResumeState, SequenceStep, Step, StepId, StepOutcome, StepParameter,
};
use crate::step::generator::bb2025::select::{Select, SelectParams};

/// Java: `StepPassBlock` (mixed/pass, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepPassBlock {
    /// Java: `fGotoLabelOnEnd` (mandatory init param).
    pub goto_label_on_end: String,
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
    /// Java: `fOldTurnMode`/`fOldPlayerStates`/`currentMove`/`isGoingForIt` — present on the
    /// re-pushed continuation instance only (`Some` ⇔ Java `fOldTurnMode != null`).
    pub resume: Option<PassBlockResumeState>,
}

impl StepPassBlock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `pushCurrentStepOnStack()` equivalent: descriptor re-pushing this step with
    /// its label + resume state, so the continuation instance is reconstructed intact.
    fn self_seq(&self, resume: &PassBlockResumeState) -> Vec<SequenceStep> {
        vec![SequenceStep::with_params(StepId::PassBlock, vec![
            StepParameter::GotoLabelOnEnd(self.goto_label_on_end.clone()),
            StepParameter::PassBlockResume(resume.clone()),
        ])]
    }

    /// Java: `Select.pushSequence(new Select.SequenceParams(getGameState(), false))`.
    fn select_seq() -> Vec<SequenceStep> {
        Select::build_sequence(&SelectParams {
            update_persistence: false,
            is_blitz_move: false,
            ..Default::default()
        })
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if game.thrower_id.is_none() {
            // Java: if (game.getThrower() == null) return;  (no next-action → NEXT via result default)
            return StepOutcome::next();
        }

        // Java: no pass block for bombs or hand over or dump off (atm)
        if game.turn_mode.is_bomb_turn() {
            return StepOutcome::next();
        }
        if let Some(ta) = game.thrower_action {
            if ta == PlayerAction::DumpOff
                || ta == PlayerAction::HandOver
                || ta == PlayerAction::HandOverMove
            {
                return StepOutcome::next();
            }
        }

        let opposing_is_home = {
            let thrower_id = game.thrower_id.clone().unwrap_or_default();
            !game.team_home.players.iter().any(|p| p.id == thrower_id)
        };
        let opposing_team = if opposing_is_home { game.team_home.clone() } else { game.team_away.clone() };

        // Java: Set<Player> passBlockers = mechanic.findPassBlockers(game, opposingTeam, false);
        //       if (passBlockers.size() == 0 && fOldTurnMode == null) { NEXT_STEP; return; }
        let pass_blockers = Self::find_pass_blockers(game, &opposing_team, false);
        if pass_blockers.is_empty() && self.resume.is_none() {
            return StepOutcome::next();
        }

        if game.turn_mode == TurnMode::PassBlock {
            // ── Re-entry: the pass-block mini-turn is running ────────────────────────
            // Java: check if actingPlayer has dropped (failed dodge)
            if let Some(pid) = game.acting_player.player_id.clone() {
                if let Some(state) = game.field_model.player_state(&pid) {
                    if !state.has_tacklezones() {
                        crate::step::util_server_steps::change_player_action_to_none(game);
                        self.end_turn = true;
                        self.end_player_action = false;
                    }
                }
            }

            if self.end_player_action {
                if game.acting_player.has_acted {
                    crate::step::util_server_steps::change_player_action_to_none(game);
                    if Self::check_no_player_active(game, &pass_blockers) {
                        self.end_turn = true;
                    } else {
                        self.end_player_action = false;
                        let resume = self.resume.clone().expect("pass-block re-entry without resume state");
                        return StepOutcome::next()
                            .push_seq(self.self_seq(&resume))
                            .push_seq(Self::select_seq());
                    }
                } else {
                    crate::step::util_server_steps::change_player_action_to_none(game);
                    self.end_player_action = false;
                    let resume = self.resume.clone().expect("pass-block re-entry without resume state");
                    return StepOutcome::next()
                        .push_seq(self.self_seq(&resume))
                        .push_seq(Self::select_seq());
                }
            }

            if self.end_turn {
                let resume = self.resume.clone().expect("pass-block end without resume state");
                // Java: restore saved states for on-pitch defenders that still have tacklezones.
                for (pid, old_state) in &resume.old_player_states {
                    let on_pitch = game.field_model.player_coordinate(pid)
                        .map(|c| !c.is_box_coordinate())
                        .unwrap_or(false);
                    let has_tz = game.field_model.player_state(pid)
                        .map(|s| s.has_tacklezones())
                        .unwrap_or(false);
                    if on_pitch && has_tz {
                        game.field_model.set_player_state(pid, *old_state);
                    }
                }

                // Java: actingPlayer.setPlayer(thrower); setPlayerAction(throwerAction); setHasPassed(true)
                game.acting_player.player_id = game.thrower_id.clone();
                game.acting_player.player_action = game.thrower_action;
                game.acting_player.has_passed = true;
                if resume.current_move >= 0 {
                    game.acting_player.current_move = resume.current_move;
                    game.acting_player.goes_for_it = resume.going_for_it;
                }

                game.turn_mode = resume.old_turn_mode;
                if resume.old_turn_mode != TurnMode::DumpOff {
                    game.home_playing = !game.home_playing;
                }

                let thrower_coordinate = game.thrower_id.as_ref()
                    .and_then(|id| game.field_model.player_coordinate(id));
                if game.thrower_action == Some(PlayerAction::HailMaryPass) {
                    // Java: reset ball
                    game.field_model.ball_in_play = true;
                    game.field_model.ball_coordinate = thrower_coordinate;
                    game.field_model.ball_moving = false;
                } else if game.thrower_action == Some(PlayerAction::HailMaryBomb) {
                    game.field_model.bomb_coordinate = None;
                }
                // else: Java forces a rangeRuler redraw — client-only, no headless state.
            }

            StepOutcome::next()
        } else {
            // ── First entry: open the pass-block window if any blocker is available ──
            // Java: Set<Player> availablePassBlockers = mechanic.findPassBlockers(game, opposingTeam, true);
            let available = Self::find_pass_blockers(game, &opposing_team, true);
            if available.is_empty() {
                game.report_list.add(ReportPassBlock::new(opposing_team.id.clone(), false));
                return StepOutcome::next()
                    .with_event(ffb_model::events::GameEvent::PassBlock { player_id: None });
            }

            let mut old_player_states: Vec<(String, ffb_model::enums::PlayerState)> = Vec::new();
            let resume = PassBlockResumeState {
                old_turn_mode: game.turn_mode,
                old_player_states: Vec::new(), // filled below
                current_move: game.acting_player.current_move,
                going_for_it: game.acting_player.goes_for_it,
            };

            game.turn_mode = TurnMode::PassBlock;
            game.home_playing = !game.home_playing;
            // Java: game.getActingPlayer().setPlayerId(null) — raw id clear, NOT changeActingPlayer.
            game.acting_player.player_id = None;

            // Java: save every on-pitch defender's state, then activate exactly the available blockers.
            for player in &opposing_team.players {
                let on_pitch = game.field_model.player_coordinate(&player.id)
                    .map(|c| !c.is_box_coordinate())
                    .unwrap_or(false);
                if on_pitch {
                    if let Some(state) = game.field_model.player_state(&player.id) {
                        old_player_states.push((player.id.clone(), state));
                        game.field_model.set_player_state(
                            &player.id,
                            state.change_active(available.contains(&player.id)),
                        );
                    }
                }
            }
            let resume = PassBlockResumeState { old_player_states, ..resume };

            // Java: Hail Mary marks the pass coordinate with a faded ball / bomb.
            if game.thrower_action == Some(PlayerAction::HailMaryPass) {
                game.field_model.ball_in_play = false;
                game.field_model.ball_coordinate = game.pass_coordinate;
                game.field_model.ball_moving = true;
            }
            if game.thrower_action == Some(PlayerAction::HailMaryBomb) {
                game.field_model.bomb_coordinate = game.pass_coordinate;
            }

            // Java: game.setDialogParameter(new DialogPassBlockParameter()) — headless: the
            // pushed Select sequence emits the ActivatePlayer prompt that drives the window.
            StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::PassBlock { player_id: None })
                .push_seq(self.self_seq(&resume))
                .push_seq(Self::select_seq())
        }
    }

    /// Java: `mechanic.findPassBlockers(game, team, checkCanReach)` (mixed OnTheBallMechanic
    /// ignores `checkCanReach` — both calls use the same predicate).
    fn find_pass_blockers(game: &Game, opposing_team: &ffb_model::model::Team, check_can_reach: bool) -> std::collections::HashSet<String> {
        use ffb_mechanics::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;
        let mechanic = ffb_mechanics::mixed::on_the_ball_mechanic::OnTheBallMechanic::new();
        mechanic.find_pass_blockers(game, opposing_team, check_can_reach)
    }

    /// Java: `checkNoPlayerActive(Set<Player>)`.
    fn check_no_player_active(game: &Game, pass_blockers: &std::collections::HashSet<String>) -> bool {
        for pid in pass_blockers {
            if let Some(state) = game.field_model.player_state(pid) {
                if state.is_active() {
                    return false;
                }
            }
        }
        true
    }
}

impl Step for StepPassBlock {
    fn id(&self) -> StepId { StepId::PassBlock }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::PassBlockResume(state) => { self.resume = Some(state.clone()); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }

    // Java consumes END_TURN/END_PLAYER_ACTION only when game.getTurnMode() == PASS_BLOCK.
    // consumes_parameter has no game access, but the continuation instance (the one queued
    // beneath the mini-turn's Select sequence) is exactly the instance carrying the resume
    // state — and it exists if and only if the mode is PASS_BLOCK. The fresh instance inside
    // the ordinary pass sequence has `resume == None` and must NOT consume (it would eat the
    // END_TURN/END_PLAYER_ACTION meant for StepEndPassing below it).
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param, StepParameter::EndTurn(_) | StepParameter::EndPlayerAction(_))
            && self.resume.is_some()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PlayerAction, TurnMode};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, skills: Vec<ffb_model::model::skill_def::SkillWithValue>) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerGender, PlayerType};
        let player = Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills,
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
    }

    /// Java: `passBlockers.size() == 0 && fOldTurnMode == null` → plain NEXT_STEP with no
    /// report at all (the ReportPassBlock(false) branch needs declared-but-unavailable
    /// blockers, which the mixed OnTheBallMechanic — ignoring checkCanReach — never yields).
    #[test]
    fn no_report_when_no_pass_blockers_at_all() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", vec![]);
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(
            !game.report_list.has_report(ReportId::PASS_BLOCK),
            "Java early-exits before the report when no defender has the skill"
        );
    }

    #[test]
    fn no_pass_block_report_when_no_thrower() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            !game.report_list.has_report(ReportId::PASS_BLOCK),
            "should not add ReportPassBlock when there is no thrower"
        );
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bomb_turn_mode_skips_pass_block() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.turn_mode = TurnMode::BombHome;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn regular_pass_without_blockers_is_a_plain_next_step() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", vec![]);
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty(), "no pass-block window without eligible blockers");
    }

    #[test]
    fn hand_over_skips_pass_block() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::HandOver);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label() {
        let mut step = StepPassBlock::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("lbl".into()));
        assert_eq!(step.goto_label_on_end, "lbl");
    }

    fn game_with_on_the_ball_defender() -> Game {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        add_player(&mut game, true, "p1", vec![]);
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        game.home_playing = true;
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        // Defender with On The Ball (canMoveWhenOpponentPasses) + a plain defender.
        add_player(&mut game, false, "otb1", vec![SkillWithValue::new(SkillId::OnTheBall)]);
        add_player(&mut game, false, "plain1", vec![]);
        for id in ["otb1", "plain1"] {
            game.field_model.set_player_coordinate(id, FieldCoordinate::new(15, 7 + (id.len() as i32 % 3)));
            // Standing AND active — matches a mid-turn defender (turn start sets the ACTIVE bit).
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
        }
        game
    }

    /// Java: first entry with an available blocker → PASS_BLOCK mode, homePlaying flip,
    /// only the eligible blockers stay ACTIVE, self + Select sequences pushed.
    #[test]
    fn available_blocker_opens_pass_block_window() {
        let mut game = game_with_on_the_ball_defender();
        let mut step = StepPassBlock::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("endPassing".into()));
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2, "must push self-descriptor + Select sequence");
        assert_eq!(out.pushes[0][0].step_id, StepId::PassBlock);
        assert_eq!(game.turn_mode, TurnMode::PassBlock);
        assert!(!game.home_playing, "homePlaying must flip to the defending team");
        assert!(game.acting_player.player_id.is_none());
        assert!(game.field_model.player_state("otb1").unwrap().is_active(),
            "On The Ball defender stays active");
        assert!(!game.field_model.player_state("plain1").unwrap().is_active(),
            "non-blocker defenders are deactivated");
        // The re-pushed descriptor carries the resume state.
        assert!(out.pushes[0][0].params.iter().any(|p| matches!(p, StepParameter::PassBlockResume(_))));
    }

    /// Java: END_TURN in PASS_BLOCK mode → restore saved states, thrower becomes acting
    /// player again with hasPassed, turn mode + homePlaying restored.
    #[test]
    fn end_turn_restores_state_and_returns_control_to_thrower() {
        let mut game = game_with_on_the_ball_defender();
        // Enter the window first.
        let mut entry_step = StepPassBlock::new();
        entry_step.set_parameter(&StepParameter::GotoLabelOnEnd("endPassing".into()));
        let mut rng = GameRng::new(0);
        let out = entry_step.start(&mut game, &mut rng);
        let resume = out.pushes[0][0].params.iter().find_map(|p| match p {
            StepParameter::PassBlockResume(s) => Some(s.clone()),
            _ => None,
        }).expect("resume state on the re-pushed descriptor");

        // Continuation instance (as reconstructed from the descriptor) receives END_TURN.
        let mut cont = StepPassBlock::new();
        cont.set_parameter(&StepParameter::GotoLabelOnEnd("endPassing".into()));
        cont.set_parameter(&StepParameter::PassBlockResume(resume));
        cont.set_parameter(&StepParameter::EndTurn(true));
        assert!(cont.consumes_parameter(&StepParameter::EndTurn(true)),
            "continuation instance must consume END_TURN (Java: mode == PASS_BLOCK)");
        let out2 = cont.start(&mut game, &mut rng);
        assert_eq!(out2.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular, "old turn mode restored");
        assert!(game.home_playing, "homePlaying flipped back to the throwing team");
        assert_eq!(game.acting_player.player_id.as_deref(), Some("p1"));
        assert!(game.acting_player.has_passed);
        assert!(game.field_model.player_state("plain1").unwrap().is_active(),
            "saved (active) state restored for non-blocker defenders");
    }

    /// The fresh instance inside the ordinary pass sequence must NOT consume
    /// END_TURN/END_PLAYER_ACTION (Java only consumes when mode == PASS_BLOCK) —
    /// consuming would starve StepEndPassing below it.
    #[test]
    fn fresh_instance_does_not_consume_end_turn() {
        let step = StepPassBlock::new();
        assert!(!step.consumes_parameter(&StepParameter::EndTurn(true)));
        assert!(!step.consumes_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
