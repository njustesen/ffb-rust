/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepAutoGazeZoat (BB2025).
///
/// Applies the Zoat's automatic gaze to an adjacent opponent (within 3 squares).
///
/// Java `executeStep()`:
///   - skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canGazeAutomaticallyThreeSquaresAway)
///   - if skill present:
///     - if endTurn || endPlayerAction || actingPlayer.isStandingUp() -> GOTO_LABEL(goToLabelOnFailure)
///     - if playerId not chosen: find opposing players with tackle zones within 3 squares of the
///       acting player; none -> NEXT_STEP; else report skill use and show
///       DialogPlayerChoiceParameter(AUTO_GAZE_ZOAT), then CONTINUE.
///     - if playerId chosen: play HYPNO sound (client-only, dropped), set the target's PlayerState
///       to confused, mark the skill used, update move squares + dice decorations.
///   - CLIENT_PLAYER_CHOICE with an empty player id declines: report skill use (false), NEXT_STEP,
///     and restore oldPlayerState if the acting player has not yet acted.
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::{ServerUtilBlock, UtilServerPlayerMove};

pub struct StepAutoGazeZoat {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: playerId — set from CLIENT_PLAYER_CHOICE command.
    pub player_id: Option<String>,
    /// Java: oldPlayerState — OLD_PLAYER_STATE init parameter, restored on decline if the
    /// acting player has not yet acted.
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

impl StepAutoGazeZoat {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            player_id: None,
            old_player_state: None,
        }
    }
}

impl Default for StepAutoGazeZoat {
    fn default() -> Self { Self::new() }
}

impl Step for StepAutoGazeZoat {
    fn id(&self) -> StepId { StepId::AutoGazeZoat }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE — set playerId or decline (empty id → SKIP_STEP / NEXT_STEP)
        // Java: CLIENT_END_TURN → set endTurn + EXECUTE_STEP
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Declined: Java SKIP_STEP path — report skill not used, NEXT_STEP, and
                    // restore oldPlayerState if the acting player has not yet acted.
                    let acting_id = game.acting_player.player_id.clone();
                    let skill = acting_id.as_deref()
                        .and_then(|id| game.player(id))
                        .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY));
                    if let Some(skill) = skill {
                        game.report_list.add(ReportSkillUse::new(acting_id.clone(), skill, false, SkillUse::DISTRACT_OPPONENT));
                    }
                    if !game.acting_player.has_acted {
                        if let (Some(id), Some(state)) = (acting_id.as_deref(), self.old_player_state) {
                            game.field_model.set_player_state(id, state);
                        }
                    }
                    return StepOutcome::next();
                }
                self.player_id = Some(player_id.clone());
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)               => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)        => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnFailure(v)     => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::OldPlayerState(v)         => { self.old_player_state = Some(*v); true }
            _ => false,
        }
    }
}

impl StepAutoGazeZoat {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer,
        //           NamedProperties.canGazeAutomaticallyThreeSquaresAway)
        let Some(acting_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::next();
        };
        let skill = game.player(&acting_id)
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_GAZE_AUTOMATICALLY_THREE_SQUARES_AWAY));

        let Some(skill) = skill else {
            return StepOutcome::next();
        };

        // Java: if (endTurn || endPlayerAction || actingPlayer.isStandingUp())
        if self.end_turn || self.end_player_action || game.acting_player.standing_up {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        if self.player_id.is_none() {
            // Java: findPlayers(game, actingPlayer.getPlayer())
            let Some(coord) = game.field_model.player_coordinate(&acting_id) else {
                return StepOutcome::next();
            };
            let is_home = game.team_home.has_player(&acting_id);
            let other_team = if is_home { &game.team_away } else { &game.team_home };
            let eligible_players: Vec<String> = UtilPlayer::find_players_with_tackle_zones(game, other_team, coord, 3)
                .into_iter()
                .cloned()
                .collect();

            if eligible_players.is_empty() {
                return StepOutcome::next();
            }

            game.report_list.add(ReportSkillUse::new(Some(acting_id.clone()), skill, true, SkillUse::DISTRACT_OPPONENT));

            return StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                eligible_players,
                reason: "AUTO_GAZE_ZOAT".into(),
                descriptions: vec![],
            });
        }

        // Java: FieldModel fieldModel = game.getFieldModel(); Player<?> player = game.getPlayerById(playerId);
        //       if (StringTool.isProvided(playerId)) { ... }
        if let Some(pid) = self.player_id.clone() {
            // client-only: SoundId.HYPNO — sound playback is client-side only (dropped, per
            // established convention, e.g. step_hypnotic_gaze.rs).
            if let Some(old_state) = game.field_model.player_state(&pid) {
                game.field_model.set_player_state(&pid, old_state.change_confused(true));
            }
            game.mark_skill_used(&acting_id, skill);
            let jumping = game.acting_player.jumping;
            UtilServerPlayerMove::update_move_squares(game, jumping);
            ServerUtilBlock::update_dice_decorations(game);
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_zoat(game: &mut Game, id: &str, coord: FieldCoordinate) {
        let mut player = Player::default();
        player.id = id.into();
        player.starting_skills.push(SkillWithValue::new(ffb_model::enums::SkillId::ExcuseMeAreYouAZoat));
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.home_playing = true;
        game.acting_player.player_id = Some(id.into());
    }

    fn add_opponent(game: &mut Game, id: &str, coord: FieldCoordinate) {
        let mut player = Player::default();
        player.id = id.into();
        game.team_away.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
    }

    #[test]
    fn start_returns_next_step_when_no_gaze_skill() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepAutoGazeZoat::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_end_turn_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAutoGazeZoat::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_decline_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAutoGazeZoat::new();
        let action = Action::SelectPlayer { player_id: String::new() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("FAIL".into())));
        assert_eq!(step.goto_label_on_failure, "FAIL");
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepAutoGazeZoat::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn report_skill_use_not_added_by_no_skill() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepAutoGazeZoat::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::SKILL_USE));
    }

    // ── Bug fix regression tests ──────────────────────────────────────────
    // Previously this step was a stub that always returned NEXT_STEP regardless of the
    // NamedProperties.canGazeAutomaticallyThreeSquaresAway skill check — the entire Zoat
    // "Excuse Me, Are You a Zoat?" ability never triggered. All of its infra
    // (UtilCards::get_unused_skill_with_property, find_players_with_tackle_zones,
    // AgentPrompt::PlayerChoice, mark_skill_used, update_move_squares,
    // update_dice_decorations) already exists in the codebase, so the stub was stale.

    #[test]
    fn with_gaze_skill_and_eligible_opponent_shows_player_choice_dialog() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_zoat(&mut game, "zoat", FieldCoordinate::new(5, 5));
        add_opponent(&mut game, "opp", FieldCoordinate::new(6, 5));

        let mut step = StepAutoGazeZoat::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::PlayerChoice { .. })));
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }

    #[test]
    fn no_eligible_opponents_returns_next_step() {
        let mut game = make_game();
        add_zoat(&mut game, "zoat", FieldCoordinate::new(5, 5));
        // Opponent far away (outside 3 squares)
        add_opponent(&mut game, "opp", FieldCoordinate::new(20, 20));

        let mut step = StepAutoGazeZoat::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_with_gaze_skill_goes_to_label_on_failure() {
        let mut game = make_game();
        add_zoat(&mut game, "zoat", FieldCoordinate::new(5, 5));
        add_opponent(&mut game, "opp", FieldCoordinate::new(6, 5));

        let mut step = StepAutoGazeZoat::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("FAIL"));
    }

    #[test]
    fn player_chosen_sets_confused_state_and_marks_skill_used() {
        let mut game = make_game();
        add_zoat(&mut game, "zoat", FieldCoordinate::new(5, 5));
        add_opponent(&mut game, "opp", FieldCoordinate::new(6, 5));

        let mut step = StepAutoGazeZoat::new();
        step.player_id = Some("opp".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        let state = game.field_model.player_state("opp").unwrap();
        assert!(state.is_confused());
        assert!(game.player("zoat").unwrap().used_skills.contains(&ffb_model::enums::SkillId::ExcuseMeAreYouAZoat));
    }

    #[test]
    fn decline_reports_skill_use_false_and_restores_old_state() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        add_zoat(&mut game, "zoat", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("zoat", ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
        game.acting_player.has_acted = false;

        let mut step = StepAutoGazeZoat::new();
        step.old_player_state = Some(ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE));

        let action = Action::SelectPlayer { player_id: String::new() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
        let state = game.field_model.player_state("zoat").unwrap();
        assert_eq!(state, ffb_model::enums::PlayerState::new(ffb_model::enums::PS_PRONE));
    }
}
