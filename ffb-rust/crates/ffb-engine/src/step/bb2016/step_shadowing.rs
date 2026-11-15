use ffb_model::enums::{ReRollSource, SkillId, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available_for, use_reroll};
use ffb_model::report::bb2016::report_tentacles_shadowing_roll::ReportTentaclesShadowingRoll;
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepShadowing +
/// inline of com.fumbbl.ffb.server.skillbehaviour.bb2016.ShadowingBehaviour.
///
/// Handles the Shadowing skill (BB2016 edition).
///
/// BB2016 differences vs BB2020:
/// - `doShadowing` also excludes `TurnMode::PassBlock`
/// - 2d6 escape roll via `DiceInterpreter::is_shadowing_escape_successful`
/// - Re-roll offered to *acting player* (not defender)
/// - Re-roll action label: "SHADOWING_ESCAPE"
/// - No `shadowerWasPreviousDefender` field
///
/// Expects: COORDINATE_FROM, DEFENDER_POSITION, USING_DIVING_TACKLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowingStatus {
    None,
    Success,
    Failure,
}

pub struct StepShadowing {
    /// Java: state.status
    pub status: ShadowingStatus,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            status: ShadowingStatus::None,
            defender_position: None,
            coordinate_from: None,
            using_diving_tackle: false,
            using_shadowing: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    fn restore_defender(&self, game: &mut Game) {
        // Java: if (state.defenderPosition != null) { defender = getPlayer(defenderPosition); setDefenderId(..) }
        if let Some(def_pos) = self.defender_position {
            let defender_at_pos = game.field_model.player_at(def_pos).cloned();
            game.defender_id = defender_at_pos;
        }
    }

    /// Java `bb2016.ShadowingBehaviour.handleExecuteStepHook`, line for line. BB2016 Shadowing is
    /// an ESCAPE roll by the DODGER: 2d6 against `8 + shadowerMA - dodgerMA`, re-rolled (if at
    /// all) by the ACTING player under `SHADOWING_ESCAPE`. `usingShadowing` stays `true` unless
    /// the escape succeeds, so a failed escape that is not (or cannot be) re-rolled means the
    /// shadower follows.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut do_next_step = true;
        let do_shadowing = !self.using_diving_tackle
            && game.turn_mode != TurnMode::KickoffReturn
            && game.turn_mode != TurnMode::PassBlock;
        let actor_id = game.acting_player.player_id.clone().unwrap_or_default();

        if do_shadowing && self.coordinate_from.is_some() && self.using_shadowing.is_none() {
            let coord_from = self.coordinate_from.unwrap();
            let mut shadowers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_skill(
                game, &actor_id, coord_from, SkillId::Shadowing, true,
            ).into_iter().cloned().collect();
            // UtilPlayer.filterThrower
            if let Some(ref tid) = game.thrower_id.clone() {
                shadowers.retain(|id| id != tid);
            }
            // UtilPlayer.filterAttackerAndDefender during DUMP_OFF
            if game.turn_mode == TurnMode::DumpOff {
                if let Some(ref aid) = game.acting_player.player_id.clone() {
                    shadowers.retain(|id| id != aid);
                }
                if let Some(ref did) = game.defender_id.clone() {
                    shadowers.retain(|id| id != did);
                }
            }
            if !shadowers.is_empty() {
                return StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::PlayerChoice {
                    eligible_players: shadowers,
                    reason: "SHADOWING".into(),
                    descriptions: vec![],
                });
            }
            self.using_shadowing = Some(false);
        }

        if do_shadowing && self.coordinate_from.is_some() && self.using_shadowing.is_some() {
            do_next_step = true;
            let defender = game.defender_id.clone().filter(|d| game.player(d).is_some());
            if self.using_shadowing == Some(true) {
                if let Some(defender_id) = defender {
                    let mut roll_shadowing = true;
                    let is_escape_reroll = self.re_rolled_action.as_deref() == Some("SHADOWING_ESCAPE");
                    if is_escape_reroll {
                        match self.re_roll_source.clone() {
                            Some(source_str) => {
                                let source = ReRollSource::new(source_str.as_str());
                                if !use_reroll(game, &source, &actor_id, rng) {
                                    roll_shadowing = false;
                                }
                            }
                            None => roll_shadowing = false,
                        }
                    }
                    if roll_shadowing {
                        // DiceRoller.rollShadowingEscape() = rollDice(2, 6)
                        let roll_escape = [rng.d6(), rng.d6()];
                        let defender_ma = game.player(&defender_id).map(|p| p.movement_with_modifiers()).unwrap_or(0);
                        let actor_ma = game.player(&actor_id).map(|p| p.movement_with_modifiers()).unwrap_or(0);
                        let successful = DiceInterpreter::is_shadowing_escape_successful(&roll_escape, defender_ma, actor_ma);
                        let minimum_roll = DiceInterpreter::minimum_roll_shadowing_escape(defender_ma, actor_ma);
                        let re_rolled = is_escape_reroll && self.re_roll_source.is_some();
                        game.report_list.add(ReportTentaclesShadowingRoll::new(
                            "Shadowing".into(),
                            defender_id.clone(),
                            roll_escape.to_vec(),
                            successful,
                            minimum_roll,
                            re_rolled,
                        ));
                        if successful {
                            self.using_shadowing = Some(false);
                        } else if !is_escape_reroll {
                            // askForReRollIfAvailable(gameState, actingPlayer.getPlayer(), SHADOWING_ESCAPE, ..)
                            if let Some(prompt) = ask_for_reroll_if_available_for(
                                game, Some(actor_id.as_str()), "SHADOWING_ESCAPE", minimum_roll, false)
                            {
                                self.re_rolled_action = Some("SHADOWING_ESCAPE".into());
                                if let ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, .. } = prompt {
                                    self.re_roll_source = Some(source.name.clone());
                                }
                                return StepOutcome::cont().with_prompt(prompt);
                            }
                        }
                    }
                }
            }
            if do_next_step && self.using_shadowing == Some(true) {
                // updatePlayerAndBallPosition(defender, coordinateFrom) + updateMoveSquares
                if let (Some(defender_id), Some(coord)) = (game.defender_id.clone(), self.coordinate_from) {
                    let has_ball = UtilPlayer::has_ball(game, &defender_id);
                    game.field_model.set_player_coordinate(&defender_id, coord);
                    if has_ball {
                        game.field_model.ball_coordinate = Some(coord);
                    }
                    UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
                }
            }
        }

        if do_next_step {
            self.restore_defender(game);
        }
        StepOutcome::next()
    }
}

impl Default for StepShadowing {
    fn default() -> Self { Self::new() }
}

impl Step for StepShadowing {
    fn id(&self) -> StepId { StepId::Shadowing }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.SHADOWING):
            //   usingShadowing = isProvided(playerId); game.setDefenderId(playerId)
            Action::PlayerChoice { player_id, mode, .. } if mode == "SHADOWING" => {
                self.using_shadowing = Some(player_id.is_some());
                game.defender_id = player_id.clone();
            }
            Action::SelectPlayer {player_id } => {
                self.using_shadowing = Some(!player_id.is_empty());
                game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
            }
            // Re-roll declined: clear source so execute_step detects decline
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(c)     => { self.coordinate_from = *c; true }
            StepParameter::DefenderPosition(c)   => { self.defender_position = Some(*c); true }
            StepParameter::UsingDivingTackle(v)  => { self.using_diving_tackle = *v; true }
            StepParameter::UsingShadowing(v)     => { self.using_shadowing = *v; true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(5, 3);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(Some(coord))));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn defender_position_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(7, 4);
        assert!(step.set_parameter(&StepParameter::DefenderPosition(coord)));
        assert_eq!(step.defender_position, Some(coord));
    }

    #[test]
    fn using_diving_tackle_disables_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_diving_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // With usingDivingTackle=true, doShadowing=false → NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kickoff_return_mode_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn pass_block_mode_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_coordinate_from_returns_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn shadower_with_shadowing_skill_triggers_player_choice_prompt() {
        // Regression test: the eligible-shadower lookup must key off the Shadowing skill
        // itself, not the DivingTackle-only `canAttemptToTackleDodgingPlayer` property.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("actor".into());
        game.team_home.players.push(Player {
            id: "actor".into(), name: "actor".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("actor", PlayerState::new(PS_STANDING).change_active(true));
        game.team_away.players.push(Player {
            id: "shadower".into(), name: "shadower".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Shadowing, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("shadower", FieldCoordinate::new(5, 4));
        game.field_model.set_player_state("shadower", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "eligible shadower must offer the PlayerChoice dialog");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { .. })));
    }

    /// BB2016 Shadowing is the DODGER's 2d6 escape against `8 + shadowerMA - dodgerMA`, reported
    /// as `tentaclesShadowingRoll`; a failed escape with no re-roll leaves `usingShadowing` true,
    /// so the shadower follows into the vacated square (`bb2016/ShadowingBehaviour`).
    #[test]
    fn accepted_shadowing_rolls_a_two_dice_escape_and_reports_it() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerState, PS_STANDING, SkillId};
        use ffb_model::model::skill_def::SkillWithValue;
        for seed in 0..40u64 {
            let mut game = make_game();
            game.home_playing = true;
            game.acting_player.player_id = Some("actor".into());
            game.team_home.players.push(Player {
                id: "actor".into(), name: "actor".into(), nr: 1, movement: 6, strength: 3,
                agility: 3, passing: 4, armour: 9, ..Default::default()
            });
            game.field_model.set_player_coordinate("actor", FieldCoordinate::new(6, 5));
            game.field_model.set_player_state("actor", PlayerState::new(PS_STANDING).change_active(true));
            game.team_away.players.push(Player {
                id: "shadower".into(), name: "shadower".into(), nr: 1, movement: 6, strength: 3,
                agility: 3, passing: 4, armour: 9,
                starting_skills: vec![SkillWithValue { skill_id: SkillId::Shadowing, value: None }],
                ..Default::default()
            });
            game.field_model.set_player_coordinate("shadower", FieldCoordinate::new(5, 4));
            game.field_model.set_player_state("shadower", PlayerState::new(PS_STANDING).change_active(true));

            let mut step = StepShadowing::new();
            step.coordinate_from = Some(FieldCoordinate::new(5, 5));
            let mut rng = GameRng::new(seed);
            assert_eq!(step.start(&mut game, &mut rng).action, StepAction::Continue);
            let out = step.handle_command(
                &Action::PlayerChoice { player_id: Some("shadower".into()), player_ids: vec![], mode: "SHADOWING".into() },
                &mut game, &mut rng);
            assert_eq!(out.action, StepAction::NextStep);
            let reports: Vec<serde_json::Value> = game.report_list.get_reports().iter()
                .filter_map(|r| r.to_json()).collect();
            assert_eq!(reports.len(), 1, "exactly one escape roll");
            let r = &reports[0];
            assert_eq!(r["reportId"], "tentaclesShadowingRoll");
            assert_eq!(r["skill"], "Shadowing");
            assert_eq!(r["minimumRoll"], 8);
            let dice = r["tentacleRoll"].as_array().unwrap();
            assert_eq!(dice.len(), 2);
            let total: i64 = dice.iter().map(|d| d.as_i64().unwrap()).sum();
            assert_eq!(r["successful"].as_bool().unwrap(), total >= 8);
            let followed = game.field_model.player_coordinate("shadower") == Some(FieldCoordinate::new(5, 5));
            assert_eq!(followed, total < 8, "seed {seed}: shadower follows exactly when the escape fails");
        }
    }

    #[test]
    fn no_shadowers_sets_using_shadowing_false_and_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        // No opponents with Shadowing skill → usingShadowing = false → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn using_shadowing_false_returns_next_step() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_shadowing = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_player_sets_using_shadowing_true() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer {player_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        // using_shadowing set to true, then rolls dice (no defender on pitch → next_step)
        assert!(game.defender_id.as_deref() == Some("p1"));
    }

    #[test]
    fn select_empty_player_declines_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.handle_command(
            &Action::SelectPlayer {player_id: "".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.using_shadowing, Some(false));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.re_rolled_action = Some("SHADOWING_ESCAPE".into());
        step.re_roll_source = Some("TRR".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn set_parameter_using_shadowing_accepted() {
        let mut step = StepShadowing::new();
        assert!(step.set_parameter(&StepParameter::UsingShadowing(Some(true))));
        assert_eq!(step.using_shadowing, Some(true));
    }
}
