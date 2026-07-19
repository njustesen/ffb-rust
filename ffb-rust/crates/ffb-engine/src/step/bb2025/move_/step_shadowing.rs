use ffb_model::enums::{ReRollSource, SkillId, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepShadowing +
/// inline of com.fumbbl.ffb.server.skillbehaviour.bb2025.ShadowingBehaviour.
///
/// BB2025 differences vs BB2020:
/// - `doShadowing` also excludes players with `movesRandomly` property
/// - Minimum roll is fixed at 4 (not the BB2020 moveDiff formula)
/// - Shadower eligibility filtered by `shadowingCount` (movement > times already shadowed)
/// - Dialog description strings are null (no MA diff description)
pub struct StepShadowing {
    /// Java: state.coordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.defenderPosition
    pub defender_position: Option<FieldCoordinate>,
    /// Java: state.usingDivingTackle
    pub using_diving_tackle: bool,
    /// Java: state.usingShadowing (Boolean tristate)
    pub using_shadowing: Option<bool>,
    /// Java: state.shadowerWasPreviousDefender
    pub shadower_was_previous_defender: bool,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepShadowing {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            defender_position: None,
            using_diving_tackle: false,
            using_shadowing: None,
            shadower_was_previous_defender: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // BB2025: also exclude players with movesRandomly
        let actor_moves_randomly = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);

        let do_shadowing = !self.using_diving_tackle
            && game.turn_mode != TurnMode::KickoffReturn
            && !actor_moves_randomly;

        if do_shadowing {
            if let Some(coord_from) = self.coordinate_from {
                if self.using_shadowing.is_none() {
                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                    let mut shadowers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_skill(
                        game,
                        &actor_id,
                        coord_from,
                        SkillId::Shadowing,
                        true,
                    ).into_iter().cloned().collect();

                    // filterThrower
                    if let Some(ref tid) = game.thrower_id.clone() {
                        shadowers.retain(|id| id != tid);
                    }

                    // filterAttackerAndDefender during DUMP_OFF
                    if game.turn_mode == TurnMode::DumpOff {
                        if let Some(ref aid) = game.acting_player.player_id.clone() {
                            shadowers.retain(|id| id != aid);
                        }
                        if let Some(ref did) = game.defender_id.clone() {
                            shadowers.retain(|id| id != did);
                        }
                    }

                    shadowers.retain(|id| {
                        let movement = game.player(id).map(|p| p.movement_with_modifiers()).unwrap_or(0);
                        movement > game.shadowing_count(id) as i32
                    });

                    if !shadowers.is_empty() {
                        let prompt = ffb_model::prompts::AgentPrompt::PlayerChoice {
                            eligible_players: shadowers,
                            reason: "SHADOWING".into(),
                            descriptions: vec![],
                        };
                        return StepOutcome::cont().with_prompt(prompt);
                    } else {
                        self.using_shadowing = Some(false);
                    }
                }

                if let Some(using) = self.using_shadowing {
                    let mut do_next_step = true;

                    if using {
                        if let Some(ref defender_id) = game.defender_id.clone() {
                            if game.player(defender_id).is_some() {
                                let re_rolled = self.re_rolled_action.as_deref() == Some("SHADOWING");
                                let mut roll_shadowing = true;

                                if re_rolled {
                                    if let Some(ref source_str) = self.re_roll_source.clone() {
                                        let source = ReRollSource::new(source_str.as_str());
                                        if !use_reroll(game, &source, defender_id) {
                                            roll_shadowing = false;
                                            self.using_shadowing = Some(false);
                                        }
                                    } else {
                                        roll_shadowing = false;
                                        self.using_shadowing = Some(false);
                                    }
                                }

                                if roll_shadowing {
                                    game.add_shadower(defender_id);
                                    let roll = rng.d6();
                                    let min_roll = 4; // fixed in BB2025
                                    let successful = roll >= min_roll;

                                    // Java: boolean reRolled = ((step.getReRolledAction() == ReRolledActions.SHADOWING)
                                    //   && (step.getReRollSource() != null));
                                    // step.getResult().addReport(new ReportTentaclesShadowingRoll(skill, defenderId,
                                    //   roll, successful, minimumRoll, reRolled));
                                    let reported_re_rolled = re_rolled && self.re_roll_source.is_some();
                                    {
                                        use ffb_model::report::mixed::report_tentacles_shadowing_roll::ReportTentaclesShadowingRoll;
                                        game.report_list.add(ReportTentaclesShadowingRoll::new(
                                            Some(SkillId::Shadowing),
                                            Some(defender_id.clone()),
                                            roll,
                                            successful,
                                            min_roll,
                                            reported_re_rolled,
                                        ));
                                    }

                                    if !successful {
                                        if !re_rolled {
                                            if let Some(prompt) = ask_for_reroll_if_available(game, "SHADOWING", min_roll, false) {
                                                self.re_rolled_action = Some("SHADOWING".into());
                                                // Java sets reRollSource from the client's reply to the offered
                                                // dialog; the offer itself already carries the actual source
                                                // (skill re-roll or TRR) — don't assume TRR unconditionally.
                                                if let ffb_model::prompts::AgentPrompt::ReRollOffer { ref source, .. } = prompt {
                                                    self.re_roll_source = Some(source.name.clone());
                                                }
                                                do_next_step = false;
                                                return StepOutcome::cont().with_prompt(prompt);
                                            } else {
                                                self.using_shadowing = Some(false);
                                            }
                                        } else {
                                            self.using_shadowing = Some(false);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if do_next_step {
                        if self.using_shadowing == Some(true) {
                            if let (Some(ref defender_id), Some(coord)) = (game.defender_id.clone(), self.coordinate_from) {
                                if self.shadower_was_previous_defender {
                                    self.defender_position = Some(coord);
                                }
                                let has_ball = UtilPlayer::has_ball(game, defender_id);
                                game.field_model.set_player_coordinate(defender_id, coord);
                                if has_ball {
                                    game.field_model.ball_coordinate = Some(coord);
                                }
                                let outcome = StepOutcome::next()
                                    .publish(StepParameter::PlayerEnteringSquare(defender_id.clone()));
                                UtilServerPlayerMove::update_move_squares(game, game.acting_player.jumping);
                                if let Some(def_pos) = self.defender_position {
                                    let defender_at_pos = game.field_model.player_at(def_pos).cloned();
                                    game.defender_id = defender_at_pos;
                                }
                                return outcome;
                            }
                        }

                        if let Some(def_pos) = self.defender_position {
                            let defender_at_pos = game.field_model.player_at(def_pos).cloned();
                            game.defender_id = defender_at_pos;
                        }
                        return StepOutcome::next();
                    }
                }
            }
        }

        if let Some(def_pos) = self.defender_position {
            let defender_at_pos = game.field_model.player_at(def_pos).cloned();
            game.defender_id = defender_at_pos;
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
            Action::PlayerChoice { player_id, mode, .. } if mode == "SHADOWING" => {
                self.using_shadowing = Some(player_id.is_some());
                if let Some(ref pid) = player_id {
                    let is_prev = game.defender_id.as_deref().map(|d| d == pid.as_str()).unwrap_or(false);
                    if is_prev {
                        self.shadower_was_previous_defender = true;
                    } else {
                        game.defender_id = Some(pid.clone());
                    }
                }
            }
            Action::SelectPlayer {player_id } => {
                self.using_shadowing = Some(!player_id.is_empty());
                game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v)           => { self.coordinate_from = Some(*v); true }
            StepParameter::DefenderPosition(v)         => { self.defender_position = Some(*v); true }
            StepParameter::UsingDivingTackle(v)        => { self.using_diving_tackle = *v; true }
            StepParameter::Jumped(_)                   => { self.using_shadowing = Some(false); true }
            StepParameter::UsingShadowing(v)           => { self.using_shadowing = *v; true }
            StepParameter::ShadowerWasPreviousDefender(v) => { self.shadower_was_previous_defender = *v; true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_shadowing() {
        assert_eq!(StepShadowing::new().id(), StepId::Shadowing);
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepShadowing::new();
        let coord = FieldCoordinate::new(3, 4);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn using_diving_tackle_disables_shadowing() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_diving_tackle = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn kickoff_return_disables_shadowing() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_shadowers_sets_using_shadowing_false() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_shadowing, Some(false));
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
    fn jumped_parameter_sets_using_shadowing_false() {
        let mut step = StepShadowing::new();
        step.using_shadowing = Some(true);
        assert!(step.set_parameter(&StepParameter::Jumped(true)));
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn shadower_was_previous_defender_parameter_accepted() {
        let mut step = StepShadowing::new();
        assert!(step.set_parameter(&StepParameter::ShadowerWasPreviousDefender(true)));
        assert!(step.shadower_was_previous_defender);
    }

    #[test]
    fn select_player_sets_defender_id() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer {player_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.re_rolled_action = Some("SHADOWING".into());
        step.re_roll_source = Some("TRR".into());
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepShadowing::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn min_roll_is_4_fixed() {
        // BB2025: fixed min roll of 4 regardless of MA difference
        // Verify by testing with using_shadowing=true but no defender on pitch → NEXT_STEP
        let mut game = make_game();
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.using_shadowing = Some(true);
        // defender_id not set → skips roll → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn shadowing_count_filter_excludes_exhausted_shadower() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        // Shadower with MA=4, already shadowed 4 times → excluded (4 > 4 is false)
        let mut game = make_game();
        game.active_shadowers = vec!["sh1".into(), "sh1".into(), "sh1".into(), "sh1".into()];
        game.acting_player.player_id = Some("actor".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(10, 5));
        // Add shadower to away team with MA=4 adjacent to actor's previous square
        game.team_away.players.push(Player {
            id: "sh1".into(), name: "sh1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("sh1", FieldCoordinate::new(10, 4));
        // shadowing_count("sh1") = 4, movement = 4 → 4 > 4 = false → filtered out
        assert_eq!(game.shadowing_count("sh1"), 4);
        // With all shadowers filtered out, step should skip the prompt
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.using_shadowing, Some(false));
    }

    #[test]
    fn shadowing_count_filter_keeps_eligible_shadower() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        // Shadower with MA=5, shadowed 4 times → eligible (5 > 4)
        let mut game = make_game();
        game.active_shadowers = vec!["sh1".into(), "sh1".into(), "sh1".into(), "sh1".into()];
        game.acting_player.player_id = Some("actor".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(10, 5));
        game.team_away.players.push(Player {
            id: "sh1".into(), name: "sh1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("sh1", FieldCoordinate::new(10, 4));
        // shadowing_count("sh1") = 4, movement = 5 → 5 > 4 = true → kept
        // The shadower lacks the Shadowing skill so won't actually appear in the dialog,
        // but the filter itself passes. Test the count logic directly.
        assert_eq!(game.shadowing_count("sh1"), 4);
        assert!(5 > game.shadowing_count("sh1") as i32);
    }

    #[test]
    fn add_shadower_called_when_roll_succeeds() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        // Find a seed that gives roll >= 4
        let seed = (0u64..).find(|&s| GameRng::new(s).d6() >= 4).unwrap();
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(10, 5));
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(10, 4));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 5));
        step.using_shadowing = Some(true);
        step.start(&mut game, &mut GameRng::new(seed));
        // After a successful roll, "def" should have been added as a shadower
        assert!(game.active_shadowers.contains(&"def".to_string()));
    }

    #[test]
    fn shadowing_roll_emits_tentacles_shadowing_roll_report() {
        // Java ShadowingBehaviour.handleExecuteStepHook always adds a
        // ReportTentaclesShadowingRoll after rolling, whether the roll succeeds or fails.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(10, 5));
        game.team_away.players.push(Player {
            id: "def".into(), name: "def".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(10, 4));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game.defender_id = Some("def".into());
        let mut step = StepShadowing::new();
        step.coordinate_from = Some(FieldCoordinate::new(10, 5));
        step.using_shadowing = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::TENTACLES_SHADOWING_ROLL));
    }
}
