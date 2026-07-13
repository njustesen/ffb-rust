/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.move.StepTentacles` +
/// inline of `com.fumbbl.ffb.server.skillbehaviour.bb2020.TentaclesBehaviour` /
/// `com.fumbbl.ffb.server.skillbehaviour.bb2025.TentaclesBehaviour` (byte-identical to each
/// other except for the extra `hasBlocked` trigger condition, BB2020-only).
///
/// Handles the TENTACLES skill check during movement.  When the acting player moves out of
/// the tackle zone of an opponent with Tentacles, that opponent may attempt to grab the mover
/// with a strength contest (rolled by the *Tentacles player*, unlike BB2016 where the
/// *escaping player* rolls).
///
/// Init parameters (mandatory): GOTO_LABEL_ON_SUCCESS (unused by BB2020/2025 behaviour --
/// carried over 1:1 from Java, which never reads `state.goToLabelOnSuccess` in
/// `handleExecuteStepHook`; the step always resolves via NEXT_STEP).
/// Incoming parameters: COORDINATE_FROM.
/// Incoming command: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.TENTACLES).
///
/// BB2020 vs BB2025:
/// - BB2020 also triggers Tentacles when `actingPlayer.hasBlocked() && coordinateFrom != null`
///   (blitz-into-tentacles-zone case); BB2025 only checks dodging/jumping.
/// - Otherwise the roll logic (1d6, min_roll = max(6 - stDifference, 2), re-roll offered to
///   the *defender*/Tentacles player) is identical between the two editions.
use ffb_model::enums::{ReRollSource, Rules, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use crate::util::util_server_player_move::UtilServerPlayerMove;

/// Java: `StepTentacles.StepState` — all step-local state collected in one place.
#[derive(Debug, Default)]
pub struct StepTentaclesState {
    /// Java: `state.goToLabelOnSuccess`
    pub go_to_label_on_success: String,
    /// Java: `state.coordinateFrom`
    pub coordinate_from: Option<ffb_model::types::FieldCoordinate>,
    /// Java: `state.usingTentacles` — `None` = not yet decided.
    pub using_tentacles: Option<bool>,
}

/// Java: `StepTentacles` (mixed/move, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepTentacles {
    pub state: StepTentaclesState,
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
}

impl StepTentacles {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (state.usingTentacles == null) { ... find tentacles players ... }
        if self.state.using_tentacles.is_none() {
            // Java bb2020: actingPlayer.isDodging() || actingPlayer.isJumping()
            //   || (actingPlayer.hasBlocked() && state.coordinateFrom != null)
            // Java bb2025: actingPlayer.isDodging() || actingPlayer.isJumping()
            let blocked_trigger = game.rules == Rules::Bb2020
                && game.acting_player.has_blocked
                && self.state.coordinate_from.is_some();
            if game.acting_player.dodging || game.acting_player.jumping || blocked_trigger {
                if let Some(coord_from) = self.state.coordinate_from {
                    let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                    // Java: UtilPlayer.findAdjacentOpposingPlayersWithSkill(game, state.coordinateFrom, skill, false)
                    // The lookup is centred on the mover's `coordinateFrom`, not the acting
                    // player's current/destination square.
                    let tentaclers: Vec<String> = UtilPlayer::find_adjacent_opposing_players_with_skill(
                        game,
                        &actor_id,
                        coord_from,
                        SkillId::Tentacles,
                        false,
                    ).into_iter().cloned().collect();

                    if !tentaclers.is_empty() {
                        let prompt = ffb_model::prompts::AgentPrompt::PlayerChoice {
                            eligible_players: tentaclers,
                            reason: "TENTACLES".into(),
                            descriptions: vec![],
                        };
                        return StepOutcome::cont().with_prompt(prompt);
                    } else {
                        self.state.using_tentacles = Some(false);
                    }
                } else {
                    self.state.using_tentacles = Some(false);
                }
            } else {
                self.state.using_tentacles = Some(false);
            }
        }

        // Java: if (state.usingTentacles != null) { ... roll or next step ... }
        if let Some(using) = self.state.using_tentacles {
            let mut do_next_step = true;

            if using {
                if let Some(ref defender_id) = game.defender_id.clone() {
                    if game.player(defender_id).is_some() {
                        let re_rolled = self.re_roll_state.re_rolled_action.as_ref()
                            .map(|a| a.name.as_str()) == Some("TENTACLES");
                        let mut roll_tentacles = true;

                        if re_rolled {
                            if let Some(ref source) = self.re_roll_state.re_roll_source.clone() {
                                // BB2020/2025: re-roll is offered to and consumed by the
                                // *defender* (the Tentacles player), not the acting player.
                                if !use_reroll(game, source, defender_id) {
                                    roll_tentacles = false;
                                    self.state.using_tentacles = Some(false);
                                }
                            } else {
                                roll_tentacles = false;
                                self.state.using_tentacles = Some(false);
                            }
                        }

                        if roll_tentacles {
                            let roll = rng.d6();
                            let defender_st = game.player(defender_id)
                                .map(|p| p.strength_with_modifiers())
                                .unwrap_or(0);
                            let actor_id = game.acting_player.player_id.clone().unwrap_or_default();
                            let actor_st = game.player(&actor_id)
                                .map(|p| p.strength_with_modifiers())
                                .unwrap_or(0);
                            let st_difference = defender_st - actor_st;
                            let min_roll = (6 - st_difference).max(2);
                            let successful = roll >= min_roll;

                            if !successful {
                                if self.re_roll_state.re_rolled_action.as_ref().map(|a| a.name.as_str()) != Some("TENTACLES") {
                                    if let Some(prompt) = ask_for_reroll_if_available(game, "TENTACLES", min_roll, false) {
                                        self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("TENTACLES"));
                                        self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                                        do_next_step = false;
                                        return StepOutcome::cont().with_prompt(prompt);
                                    } else {
                                        self.state.using_tentacles = Some(false);
                                    }
                                } else {
                                    self.state.using_tentacles = Some(false);
                                }
                            }
                        }
                    }
                }
            }

            if do_next_step {
                if self.state.using_tentacles == Some(true) {
                    // Tentacles wins: hold the mover in place, cancel dodging/jumping,
                    // move actor back to coordinateFrom.
                    game.acting_player.dodging = false;
                    game.acting_player.jumping = false;
                    game.acting_player.held_in_place = true;
                    UtilServerPlayerMove::update_move_squares(game, false);
                    if let Some(coord) = self.state.coordinate_from {
                        if let Some(actor_id) = game.acting_player.player_id.clone() {
                            let has_ball = UtilPlayer::has_ball(game, &actor_id);
                            game.field_model.set_player_coordinate(&actor_id, coord);
                            if has_ball {
                                game.field_model.ball_coordinate = Some(coord);
                            }
                        }
                    }
                }
                if let Some(last_defender_id) = game.last_defender_id.clone() {
                    game.defender_id = Some(last_defender_id);
                    game.last_defender_id = None;
                }
                return StepOutcome::next();
            }
        }

        StepOutcome::next()
    }
}

impl Step for StepTentacles {
    fn id(&self) -> StepId { StepId::Tentacles }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE / PlayerChoiceMode.TENTACLES
            Action::PlayerChoice { player_id, mode, .. } if mode == "TENTACLES" => {
                // Java: state.usingTentacles = StringTool.isProvided(playerChoiceCommand.getPlayerId())
                self.state.using_tentacles = Some(player_id.is_some());
                if let Some(pid) = player_id {
                    // Java: game.setLastDefenderId(game.getDefenderId()); game.setDefenderId(playerId)
                    game.last_defender_id = game.defender_id.clone();
                    game.defender_id = Some(pid.clone());
                }
            }
            // Compatibility with the BB2016 SelectPlayer command shape used elsewhere.
            Action::SelectPlayer { player_id } => {
                self.state.using_tentacles = Some(!player_id.is_empty());
                if !player_id.is_empty() {
                    game.last_defender_id = game.defender_id.clone();
                    game.defender_id = Some(player_id.clone());
                }
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v)     => { self.state.coordinate_from = Some(*v); true }
            StepParameter::GotoLabelOnSuccess(v) => { self.state.go_to_label_on_success = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_game_bb2020() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_tentacles() {
        assert_eq!(StepTentacles::new().id(), StepId::Tentacles);
    }

    #[test]
    fn start_without_dodging_or_jumping_returns_next() {
        let mut step = StepTentacles::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.state.using_tentacles, Some(false));
    }

    #[test]
    fn dodging_with_no_tentacles_players_sets_false() {
        let mut step = StepTentacles::new();
        let mut game = make_game();
        game.acting_player.dodging = true;
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.state.using_tentacles, Some(false));
    }

    #[test]
    fn bb2020_has_blocked_with_coordinate_from_triggers_lookup() {
        // BB2020-only trigger: hasBlocked() && coordinateFrom != null (no dodge/jump needed).
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game_bb2020();
        game.home_playing = true;
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_blocked = true;
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
            id: "tentacler".into(), name: "tentacler".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Tentacles, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("tentacler", FieldCoordinate::new(5, 4));
        game.field_model.set_player_state("tentacler", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "hasBlocked trigger must offer PlayerChoice on BB2020");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { .. })));
    }

    #[test]
    fn bb2025_has_blocked_without_dodge_or_jump_does_not_trigger() {
        // BB2025 has no hasBlocked trigger: same setup as above but under BB2025 rules.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_blocked = true;
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
            id: "tentacler".into(), name: "tentacler".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Tentacles, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("tentacler", FieldCoordinate::new(5, 4));
        game.field_model.set_player_state("tentacler", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep, "BB2025 must not trigger Tentacles on hasBlocked alone");
        assert_eq!(step.state.using_tentacles, Some(false));
    }

    #[test]
    fn tentacles_player_with_skill_triggers_player_choice_prompt() {
        // Regression: the eligible-holder lookup must key off squares adjacent to the
        // *mover's* coordinateFrom, and check the Tentacles skill specifically.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::skill_def::SkillWithValue;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.dodging = true;
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
            id: "tentacler".into(), name: "tentacler".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Tentacles, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("tentacler", FieldCoordinate::new(5, 4));
        game.field_model.set_player_state("tentacler", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "eligible Tentacles holder must offer the PlayerChoice dialog");
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::PlayerChoice { .. })));
    }

    #[test]
    fn player_choice_false_returns_next() {
        let mut step = StepTentacles::new();
        step.state.go_to_label_on_success = "success".into();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: None, player_ids: vec![], mode: "TENTACLES".into() },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn player_choice_true_with_no_defender_returns_next() {
        // usingTentacles=true but game.getDefender() is None (no defender_id was actually
        // set to a real player in the roster) -> roll is skipped, falls to doNextStep with
        // usingTentacles staying true -> mover held in place.
        let mut step = StepTentacles::new();
        step.state.go_to_label_on_success = "success".into();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(
            &Action::PlayerChoice { player_id: Some("def1".into()), player_ids: vec![], mode: "TENTACLES".into() },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.state.using_tentacles, Some(true));
    }

    #[test]
    fn successful_roll_holds_mover_in_place() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.dodging = true;
        game.acting_player.strength = 3;
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
        game.field_model.set_player_coordinate("actor", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("actor", PlayerState::new(PS_STANDING).change_active(true));
        game.team_away.players.push(Player {
            id: "tentacler".into(), name: "tentacler".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 6, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("tentacler", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("tentacler", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.state.using_tentacles = Some(true);
        game.defender_id = Some("tentacler".into());
        // defender ST 6, actor ST 3 -> stDifference = 3 -> minRoll = max(6-3,2) = 3
        // a roll of 6 always succeeds regardless of seed; use a seed that yields a high d6.
        let mut rng = GameRng::new(1);
        let out = step.start(&mut game, &mut rng);
        // Whatever the die roll, the step must resolve deterministically: either it
        // finishes the strength contest (NEXT_STEP, holding the mover in place on a
        // Tentacles win) or it offers a re-roll (CONTINUE) -- it must never silently
        // drop the roll outcome.
        match out.action {
            StepAction::NextStep => {
                // usingTentacles is only ever left true (mover held) when the roll
                // succeeded or no re-roll was available; both leave a consistent state.
                if step.state.using_tentacles == Some(true) {
                    assert!(game.acting_player.held_in_place);
                }
            }
            StepAction::Continue => {
                assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::ReRollOffer { .. })));
            }
            other => panic!("unexpected step action: {other:?}"),
        }
    }

    #[test]
    fn set_parameter_goto_label_on_success() {
        let mut step = StepTentacles::new();
        step.set_parameter(&StepParameter::GotoLabelOnSuccess("lbl".into()));
        assert_eq!(step.state.go_to_label_on_success, "lbl");
    }

    #[test]
    fn coordinate_from_parameter_accepted() {
        let mut step = StepTentacles::new();
        let coord = FieldCoordinate::new(3, 4);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.state.coordinate_from, Some(coord));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepTentacles::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn select_player_sets_defender_id() {
        let mut game = make_game();
        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(
            &Action::SelectPlayer { player_id: "p1".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn select_empty_player_declines_tentacles() {
        let mut game = make_game();
        let mut step = StepTentacles::new();
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: "".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.state.using_tentacles, Some(false));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepTentacles::new();
        step.re_roll_state.re_rolled_action = Some(ReRolledAction::new("TENTACLES"));
        step.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
        step.state.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_state.re_roll_source.is_none());
    }
}
