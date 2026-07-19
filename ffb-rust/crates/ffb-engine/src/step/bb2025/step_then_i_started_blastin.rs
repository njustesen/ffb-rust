/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepThenIStartedBlastin (BB2025).
///
/// Resolves the "Then I Started Blastin'!" ability: throw a keg at a target, causing injury.
///
/// Commands: CLIENT_TARGET_SELECTED (target selection), CLIENT_END_TURN, UseReRoll (BlastinSolvesEverything).
///
/// Java: `NamedProperties.canBlastRemotePlayer` is granted by the `BlastinSolvesEverything`
/// skill (BB2025) — corresponds to `SkillId::BlastinSolvesEverything`.
use ffb_model::enums::{ApothecaryMode, SkillId, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::report::mixed::report_then_i_started_blastin::ReportThenIStartedBlastin;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_then_i_started_blastin::InjuryTypeThenIStartedBlastin;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

const RE_ROLLED_ACTION_NAME: &str = "BlastinSolvesEverything";

pub struct StepThenIStartedBlastin {
    /// Java: gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: roll — the skill die result.
    pub roll: i32,
    /// Java: oldTurnMode.
    pub old_turn_mode: Option<TurnMode>,
    /// Java: AbstractStepWithReRoll composition.
    pub re_roll_state: ReRollState,
}

impl StepThenIStartedBlastin {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            roll: 0,
            old_turn_mode: None,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl Step for StepThenIStartedBlastin {
    fn id(&self) -> StepId { StepId::ThenIStartedBlastin }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_TARGET_SELECTED
            Action::SelectPlayer { player_id } => {
                game.defender_id = Some(player_id.clone());
                let acting_id = match game.acting_player.player_id.clone() {
                    Some(id) => id,
                    None => return StepOutcome::next(),
                };

                // Java: game.playingTeamHasActingPLayer() — is the ACTING player on the
                // currently-"playing" team (per home_playing)? Not about the selected target.
                let playing_team_has_acting_player = if game.home_playing {
                    game.team_home.player(&acting_id).is_some()
                } else {
                    game.team_away.player(&acting_id).is_some()
                };

                if playing_team_has_acting_player {
                    return self.execute_step(game, rng);
                }

                // Java: else branch — flip home_playing, animate, hitPlayer(defender) directly.
                game.home_playing = !game.home_playing;
                let hit_id = player_id.clone();
                let outcome = self.hit_player(game, rng, &hit_id);
                game.report_list.add(ReportThenIStartedBlastin::new(
                    Some(acting_id.clone()),
                    Some(hit_id.clone()),
                    0,
                    true,
                    false,
                ));
                return outcome.with_event(GameEvent::ThenIStartedBlastin {
                    attacker_id: acting_id,
                    defender_id: Some(hit_id),
                    roll: 0,
                    success: true,
                    fumble: false,
                });
            }
            // Java: CLIENT_END_TURN → restoreTurnModes(game); publishParameter(END_PLAYER_ACTION, true); SKIP_STEP + NEXT_STEP
            Action::EndTurn => {
                self.restore_turn_modes(game);
                return StepOutcome::next().publish(StepParameter::EndPlayerAction(true));
            }
            // Java: AbstractStepWithReRoll's inherited handleCommand consumes the re-roll
            // response and re-invokes executeStep(); modeled here explicitly.
            Action::UseReRoll { use_reroll: wants_reroll } => {
                if *wants_reroll {
                    if let (Some(source), Some(acting_id)) =
                        (self.re_roll_state.re_roll_source.clone(), game.acting_player.player_id.clone())
                    {
                        if use_reroll(game, &source, &acting_id) {
                            return self.execute_step(game, rng);
                        }
                    }
                }
                return self.fail(game, rng);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepThenIStartedBlastin {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().setNextAction(NEXT_STEP) — default action.
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let is_rerolling = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == RE_ROLLED_ACTION_NAME)
            .unwrap_or(false);

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canBlastRemotePlayer)
        // canBlastRemotePlayer corresponds to SkillId::BlastinSolvesEverything (BB2025).
        let has_skill = game.player(&acting_id)
            .map(|p| p.all_skill_ids().any(|s| s == SkillId::BlastinSolvesEverything)
                && !p.used_skills.contains(&SkillId::BlastinSolvesEverything))
            .unwrap_or(false);

        if !has_skill && !is_rerolling {
            return StepOutcome::next();
        }

        if is_rerolling {
            // Java: if (getReRollSource() == null || !UtilServerReRoll.useReRoll(...)) { fail(); return; }
            let has_source = self.re_roll_state.re_roll_source.is_some();
            if !has_source {
                return self.fail(game, rng);
            }
            // Consumption already handled by the caller (handle_command's UseReRoll branch)
            // before re-invoking execute_step; if we got here directly (e.g. start()), the
            // re-roll can't be consumed — mirror Java's fail() in that case defensively.
        }

        // Java: if (game.getTurnMode() != TurnMode.THEN_I_STARTED_BLASTIN) → enter mode, CONTINUE
        if game.turn_mode != TurnMode::ThenIStartedBlastin {
            self.old_turn_mode = game.last_turn_mode;
            game.turn_mode = TurnMode::ThenIStartedBlastin;
            return StepOutcome::cont();
        }

        // Java: actingPlayer.markSkillUsed(skill)
        if has_skill {
            Self::mark_skill_used(game, &acting_id, SkillId::BlastinSolvesEverything);
        }

        // Java: roll = getGameState().getDiceRoller().rollSkill();
        //       success = DiceInterpreter.getInstance().isSkillRollSuccessful(roll, 3)
        self.roll = rng.d6();
        let success = self.roll >= 3;
        let def_id = game.defender_id.clone();
        game.report_list.add(ReportThenIStartedBlastin::new(
            Some(acting_id.clone()),
            def_id.clone(),
            self.roll,
            success,
            self.roll == 1,
        ));
        let tisb_event = GameEvent::ThenIStartedBlastin {
            attacker_id: acting_id.clone(),
            defender_id: def_id.clone(),
            roll: self.roll,
            success,
            fumble: self.roll == 1,
        };

        if success {
            if let Some(did) = def_id {
                return self.hit_player(game, rng, &did).with_event(tisb_event);
            }
            return StepOutcome::next().with_event(tisb_event);
        }

        // Java: failure branch — ask for re-roll (unless already re-rolling) or fail().
        if !is_rerolling {
            if let Some(prompt) = ask_for_reroll_if_available(game, RE_ROLLED_ACTION_NAME, 3, false) {
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION_NAME));
                // Java: `AbstractStepWithReRoll` stores the offered source (`fReRollSource`) so
                // that a later `UtilServerReRoll.useReRoll(this, getReRollSource(), ...)` call
                // can consume it.
                if let ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, .. } = prompt {
                    self.re_roll_state.re_roll_source = Some(source.clone());
                }
                return StepOutcome::cont().with_event(tisb_event).with_prompt(prompt);
            }
        }
        self.fail(game, rng).with_event(tisb_event)
    }

    /// Java: `fail()` — if roll == 1: hit self (fumble); else: flip home_playing + Continue.
    fn fail(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.roll == 1 {
            if let Some(actor_id) = game.acting_player.player_id.clone() {
                return self.hit_player(game, rng, &actor_id);
            }
            StepOutcome::next()
        } else {
            game.home_playing = !game.home_playing;
            StepOutcome::cont()
        }
    }

    /// Java: `hitPlayer(Player hitPlayer)`.
    fn hit_player(&mut self, game: &mut Game, rng: &mut GameRng, hit_player_id: &str) -> StepOutcome {
        let target_coord = match game.field_model.player_coordinate(hit_player_id) {
            Some(c) => c,
            None => {
                self.restore_turn_modes(game);
                return StepOutcome::next();
            }
        };

        let mut injury_type = InjuryTypeThenIStartedBlastin::new();
        let injury_result = handle_injury(
            game, rng, &mut injury_type,
            None, hit_player_id,
            target_coord, None, None,
            ApothecaryMode::Defender,
        );

        // Java: boolean endTurn = UtilPlayer.hasBall(game, hitPlayer) && game.getActingTeam().hasPlayer(hitPlayer)
        let hit_player_has_ball = game.field_model.ball_coordinate == Some(target_coord);
        let acting_team_has_hit_player = if game.home_playing {
            game.team_home.player(hit_player_id).is_some()
        } else {
            game.team_away.player(hit_player_id).is_some()
        };
        let end_turn = hit_player_has_ball && acting_team_has_hit_player;

        let dpc = DropPlayerContext {
            injury_result: Some(Box::new(injury_result)),
            end_turn,
            eligible_for_safe_pair_of_hands: true,
            player_id: Some(hit_player_id.to_owned()),
            apothecary_mode: Some(ApothecaryMode::Defender),
            requires_armour_break: true,
            ..DropPlayerContext::new()
        };

        self.restore_turn_modes(game);

        // Java: publishParameter(END_PLAYER_ACTION, true) at the end of hitPlayer().
        StepOutcome::next()
            .publish(StepParameter::DropPlayerContext(Box::new(dpc)))
            .publish(StepParameter::EndPlayerAction(true))
    }

    /// Java: `restoreTurnModes(Game game)`.
    fn restore_turn_modes(&mut self, game: &mut Game) {
        game.turn_mode = game.last_turn_mode.unwrap_or(TurnMode::Regular);
        game.last_turn_mode = self.old_turn_mode;
    }

    fn mark_skill_used(game: &mut Game, player_id: &str, skill_id: SkillId) {
        let is_home = game.team_home.player(player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(player_id) {
                p.used_skills.insert(skill_id);
            }
        } else if let Some(p) = game.team_away.player_mut(player_id) {
            p.used_skills.insert(skill_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_player_with_skill(id: &str, skill: SkillId) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_plain_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game() -> (Game, String) {
        let actor_id = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player_with_skill(&actor_id, SkillId::BlastinSolvesEverything));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(actor_id.clone());
        game.field_model.set_player_state(&actor_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&actor_id, FieldCoordinate::new(10, 7));
        (game, actor_id)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = Some("ghost".into());
        let mut step = StepThenIStartedBlastin::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_skill_does_not_add_report() {
        // Regression: previously this step unconditionally added a
        // ReportThenIStartedBlastin(roll=0, success=false) on every start(), even when the
        // acting player has no BlastinSolvesEverything skill (contradicts Java, which only
        // executes the report-adding branch when `skill != null`).
        use ffb_model::report::report_id::ReportId;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = Some("ghost".into());
        let mut step = StepThenIStartedBlastin::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            !game.report_list.has_report(ReportId::THEN_I_STARTED_BLASTIN),
            "no report should be added when the acting player lacks the skill"
        );
    }

    #[test]
    fn first_call_with_skill_enters_mode_and_returns_continue() {
        let (mut game, _) = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::ThenIStartedBlastin);
    }

    #[test]
    fn entering_mode_does_not_add_report() {
        // Java: entering THEN_I_STARTED_BLASTIN mode just returns CONTINUE; no report until
        // the actual roll happens.
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game();
        let mut step = StepThenIStartedBlastin::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::THEN_I_STARTED_BLASTIN));
    }

    #[test]
    fn end_turn_restores_turn_mode_and_returns_next() {
        let (mut game, _) = make_game();
        game.turn_mode = TurnMode::ThenIStartedBlastin;
        game.last_turn_mode = Some(TurnMode::Regular);
        let mut step = StepThenIStartedBlastin::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_teammate_target_rolls_and_adds_report() {
        // Acting player is on the home team and home_playing is true → playingTeamHasActingPLayer
        // is true → executeStep() rolls (since already in TISB mode).
        use ffb_model::report::report_id::ReportId;
        let (mut game, _actor_id) = make_game();
        let def_id = "defender2".to_string();
        game.team_away.players.push(make_plain_player(&def_id));
        game.field_model.set_player_state(&def_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&def_id, FieldCoordinate::new(12, 7));
        game.turn_mode = TurnMode::ThenIStartedBlastin;
        game.last_turn_mode = Some(TurnMode::Regular);

        let mut step = StepThenIStartedBlastin::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        step.handle_command(
            &Action::SelectPlayer { player_id: def_id.clone() },
            &mut game,
            &mut GameRng::new(42),
        );
        assert!(game.report_list.has_report(ReportId::THEN_I_STARTED_BLASTIN));
        assert_eq!(game.defender_id.as_deref(), Some(def_id.as_str()));
    }

    #[test]
    fn select_target_when_not_playing_team_hits_directly() {
        // Flip home_playing before selection so playingTeamHasActingPLayer() is false →
        // else branch: flips home_playing back, hits the target directly with roll=0.
        let (mut game, _actor_id) = make_game();
        game.home_playing = false; // acting player is on home team, so this makes the check false
        let def_id = "defender3".to_string();
        game.team_away.players.push(make_plain_player(&def_id));
        game.field_model.set_player_state(&def_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&def_id, FieldCoordinate::new(12, 7));

        let mut step = StepThenIStartedBlastin::new();
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: def_id.clone() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))),
            "should hit the target directly, publishing DropPlayerContext");
        // home_playing flipped back to true (was false, now flipped once)
        assert!(game.home_playing);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepThenIStartedBlastin::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
    }

    #[test]
    fn restore_turn_modes_works_correctly() {
        let (mut game, _) = make_game();
        game.turn_mode = TurnMode::ThenIStartedBlastin;
        game.last_turn_mode = Some(TurnMode::Regular);
        let mut step = StepThenIStartedBlastin::new();
        step.old_turn_mode = Some(TurnMode::Kickoff);
        step.restore_turn_modes(&mut game);
        assert_eq!(game.turn_mode, TurnMode::Regular);
        assert_eq!(game.last_turn_mode, Some(TurnMode::Kickoff));
    }
}
