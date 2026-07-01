use ffb_model::enums::{ApothecaryMode, PlayerAction, SkillId, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_then_i_started_blastin::InjuryTypeThenIStartedBlastin;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury;
use crate::step::util_server_re_roll::ask_for_reroll_if_available;

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepThenIStartedBlastin` (BB2020).
///
/// Resolves the "Then I Started Blastin'!" special ability (BB2020 star player skill).
/// The acting player can use this skill once per turn to target an opponent anywhere on the pitch.
///
/// Flow (mirrors Java `executeStep()`):
/// 1. Find unused `canBlastRemotePlayer` skill (or check if re-rolling THEN_I_STARTED_BLASTIN).
/// 2. If just starting (not in THEN_I_STARTED_BLASTIN turn mode):
///    - Mark action used; flip home_playing; enter mode; report; return Continue.
/// 3. On `CLIENT_TARGET_SELECTED`:
///    - If playing team has acting player → `executeStep()` (roll).
///    - Else → `hitPlayer()` directly (auto-hit).
/// 4. Roll d6 >= 3 → success → `hitPlayer()`.
/// 5. Fail → ask for re-roll or `fail()`.
/// 6. `fail()`: if roll == 1 → auto-hit self; else → flip home_playing + Continue.
/// 7. `hitPlayer()`: `handleInjury(InjuryTypeThenIStartedBlastin)` + publish `DropPlayerContext`.
///
/// BB2020 differences from BB2025 (if any are ported there in future):
/// - `markActionUsed`: ThrowTeamMate → pass_used; includes KickEmBlitz / KickTeamMate; no PUNT case.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepThenIStartedBlastin`.
pub struct StepThenIStartedBlastin {
    /// Java: gotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: roll
    pub roll: i32,
    /// Java: oldTurnMode
    pub old_turn_mode: Option<TurnMode>,
    /// Java: AbstractStepWithReRoll composition
    pub re_roll_state: ReRollState,
}

const RE_ROLLED_ACTION_NAME: &str = "ThenIStartedBlastin";

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

                let player_is_on_active_team = if game.home_playing {
                    game.team_home.player(player_id).is_some()
                } else {
                    game.team_away.player(player_id).is_some()
                };
                if player_is_on_active_team {
                    // Java: playingTeamHasActingPlayer → executeStep
                    return self.execute_step(game, rng);
                } else {
                    // Java: else → flip home_playing + animation + hitPlayer(defender)
                    game.home_playing = !game.home_playing;
                    let hit_id = player_id.clone();
                    let outcome = self.hit_player(game, rng, &hit_id);
                    // DEFERRED(tisb): ReportThenIStartedBlastin(actingPlayerId, defenderId, 0, true, false)
                    return outcome;
                }
            }
            // Java: CLIENT_END_TURN → restoreTurnModes; SKIP_STEP + NEXT_STEP
            Action::EndTurn => {
                self.restore_turn_modes(game);
                return StepOutcome::next();
            }
            // Java: UseReRoll response
            Action::UseReRoll { use_reroll } => {
                if *use_reroll {
                    // Use the re-roll: consume token
                    let td = game.turn_data_mut();
                    if td.rerolls > 0 {
                        td.rerolls -= 1;
                        td.reroll_used = true;
                    }
                    // Clear re-roll state so executeStep re-rolls
                    self.re_roll_state.re_rolled_action = None;
                    return self.execute_step(game, rng);
                } else {
                    // Declined re-roll → fail
                    let outcome = self.fail(game, rng);
                    return outcome;
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepThenIStartedBlastin {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Default action: NEXT_STEP (Java: getResult().setNextAction(NEXT_STEP))
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let acting_player = game.player(&acting_id).cloned();

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canBlastRemotePlayer)
        // canBlastRemotePlayer corresponds to SkillId::ThenIStartedBlastin
        let has_skill = acting_player.as_ref()
            .map(|p| p.all_skill_ids()
                .any(|s| s == SkillId::ThenIStartedBlastin && !p.used_skills.contains(&s)))
            .unwrap_or(false);

        let is_rerolling = self.re_roll_state.re_rolled_action.as_ref()
            .map(|a| a.name == RE_ROLLED_ACTION_NAME)
            .unwrap_or(false);

        if !has_skill && !is_rerolling {
            return StepOutcome::next();
        }

        // If rerolling: check re-roll source is available
        if is_rerolling {
            // Re-roll already triggered; proceed with the re-roll
            // (consume was already done in handle_command on UseReRoll)
        }

        // Java: if (game.getTurnMode() != TurnMode.THEN_I_STARTED_BLASTIN) → enter mode
        if game.turn_mode != TurnMode::ThenIStartedBlastin {
            self.old_turn_mode = game.last_turn_mode;
            game.turn_mode = TurnMode::ThenIStartedBlastin;
            // DEFERRED(tisb): ReportThenIStartedBlastin(actingPlayerId, null, 0, false, false)
            return StepOutcome::cont();
        }

        // Java: actingPlayer.markSkillUsed(skill)
        // canBlastRemotePlayer corresponds to SkillId::ThenIStartedBlastin
        if acting_player.as_ref()
            .map(|p| p.all_skill_ids().any(|s| s == SkillId::ThenIStartedBlastin))
            .unwrap_or(false)
        {
            Self::mark_skill_used(game, &acting_id, SkillId::ThenIStartedBlastin);
        }

        // Java: mark action used based on player action
        Self::mark_action_used(game);

        // Java: roll = rollSkill(); success = isSkillRollSuccessful(roll, 3) → roll >= 3
        self.roll = rng.d6();
        let success = self.roll >= 3;

        // DEFERRED(tisb): ReportThenIStartedBlastin(actingId, defenderId, roll, success, roll==1)

        if success {
            if let Some(def_id) = game.defender_id.clone() {
                return self.hit_player(game, rng, &def_id);
            }
            return StepOutcome::next();
        } else {
            // Failure: ask for re-roll or fail
            if !is_rerolling {
                if let Some(prompt) = ask_for_reroll_if_available(game, RE_ROLLED_ACTION_NAME, 3, false) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION_NAME));
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            return self.fail(game, rng);
        }
    }

    /// Java: `fail()` — if roll == 1: hit self; else: flip home_playing + Continue
    fn fail(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.roll == 1 {
            // Java: hitPlayer(actingPlayer) — fumbled keg hits self
            // DEFERRED(tisb): setAnimation(FUMBLED_KEG, throwerCoordinate)
            if let Some(actor_id) = game.acting_player.player_id.clone() {
                return self.hit_player(game, rng, &actor_id);
            }
            StepOutcome::next()
        } else {
            // Java: flip home_playing; return Continue (ask for target again)
            game.home_playing = !game.home_playing;
            // DEFERRED(tisb): setSound(QUESTION)
            StepOutcome::cont()
        }
    }

    /// Java: `hitPlayer(Player hitPlayer)`.
    /// Applies InjuryTypeThenIStartedBlastin, publishes DropPlayerContext, restores turn modes.
    fn hit_player(&mut self, game: &mut Game, rng: &mut GameRng, hit_player_id: &str) -> StepOutcome {
        let target_coord = match game.field_model.player_coordinate(hit_player_id) {
            Some(c) => c,
            None => {
                self.restore_turn_modes(game);
                return StepOutcome::next();
            }
        };

        // Java: InjuryResult injuryResult = UtilServerInjury.handleInjury(…, InjuryTypeThenIStartedBlastin, …)
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

        // Java: publishParameter(DROP_PLAYER_CONTEXT, new DropPlayerContext(injuryResult, endTurn, true, null, hitPlayer.getId(), DEFENDER, true))
        let dpc = DropPlayerContext {
            injury_result: Some(Box::new(injury_result)),
            end_turn,
            eligible_for_safe_pair_of_hands: true,
            player_id: Some(hit_player_id.to_owned()),
            apothecary_mode: Some(ApothecaryMode::Defender),
            requires_armour_break: true,
            ..DropPlayerContext::new()
        };

        // DEFERRED(tisb): setSound(EXPLODE)
        // DEFERRED(tisb): setAnimation(THEN_I_STARTED_BLASTIN, startCoord, targetCoord)

        self.restore_turn_modes(game);

        StepOutcome::next()
            .publish(StepParameter::DropPlayerContext(Box::new(dpc)))
    }

    /// Java: `restoreTurnModes(Game game)`.
    fn restore_turn_modes(&mut self, game: &mut Game) {
        game.turn_mode = game.last_turn_mode.unwrap_or(TurnMode::Regular);
        game.last_turn_mode = self.old_turn_mode;
    }

    /// Java: `markActionUsed` (BB2020 variant — no PUNT case).
    fn mark_action_used(game: &mut Game) {
        let action = game.acting_player.player_action;
        let turn = game.turn_data_mut();
        match action {
            Some(PlayerAction::BlitzMove | PlayerAction::KickEmBlitz) => turn.blitz_used = true,
            Some(PlayerAction::FoulMove) => {} // conditionally skip based on allowsAdditionalFoul — TODO
            Some(PlayerAction::HandOverMove) => turn.hand_over_used = true,
            Some(PlayerAction::PassMove | PlayerAction::ThrowTeamMateMove) => turn.pass_used = true,
            Some(PlayerAction::KickTeamMateMove) => turn.ktm_used = true,
            _ => {}
        }
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
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;

    fn make_player_with_skill(id: &str, skill: SkillId) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
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
        }
    }

    fn make_game() -> (Game, String) {
        let actor_id = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player_with_skill(&actor_id, SkillId::ThenIStartedBlastin));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.home_playing = true;
        game.acting_player.player_id = Some(actor_id.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&actor_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&actor_id, FieldCoordinate::new(10, 7));
        (game, actor_id)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let mut step = StepThenIStartedBlastin::new();
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.acting_player.player_id = Some("ghost".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn first_call_with_skill_enters_mode_and_returns_continue() {
        let (mut game, _) = make_game();
        let mut step = StepThenIStartedBlastin::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should enter THEN_I_STARTED_BLASTIN mode and wait for target selection
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::ThenIStartedBlastin);
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
    fn select_opponent_in_tisb_mode_publishes_drop_player_context() {
        let (mut game, _) = make_game();
        let def_id = "defender".to_string();
        game.team_away.players.push(make_plain_player(&def_id));
        game.field_model.set_player_state(&def_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&def_id, FieldCoordinate::new(15, 7));

        game.turn_mode = TurnMode::ThenIStartedBlastin;
        game.last_turn_mode = Some(TurnMode::Regular);

        let mut step = StepThenIStartedBlastin::new();
        // Simulate selecting an opponent on the away team (not acting team)
        // This triggers the auto-hit path (else branch in Java)
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: def_id.clone() },
            &mut game,
            &mut GameRng::new(0),
        );

        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))),
            "selecting opponent should publish DropPlayerContext via hitPlayer");
    }

    #[test]
    fn restore_turn_modes_works_correctly() {
        let (mut game, _) = make_game();
        game.turn_mode = TurnMode::ThenIStartedBlastin;
        game.last_turn_mode = Some(TurnMode::Regular);
        let mut step = StepThenIStartedBlastin::new();
        step.old_turn_mode = Some(TurnMode::Kickoff);
        step.restore_turn_modes(&mut game);
        // After restore: turn_mode = last_turn_mode (Regular), last_turn_mode = old_turn_mode (Kickoff)
        assert_eq!(game.turn_mode, TurnMode::Regular);
        assert_eq!(game.last_turn_mode, Some(TurnMode::Kickoff));
    }

    #[test]
    fn mark_action_used_blitz_move_sets_blitz_used() {
        let (mut game, _) = make_game();
        game.acting_player.player_action = Some(PlayerAction::BlitzMove);
        StepThenIStartedBlastin::mark_action_used(&mut game);
        assert!(game.turn_data_mut().blitz_used);
    }
}
