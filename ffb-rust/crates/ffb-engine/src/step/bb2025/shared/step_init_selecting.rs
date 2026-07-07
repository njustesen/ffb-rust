use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_staller_detected::ReportStallerDetected;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, PlayerActionChoice};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps;

/// Initialises the player-selection phase: waits for `ActivatePlayer` or `EndTurn` commands,
/// then dispatches to the appropriate action sequence via GOTO_LABEL or NEXT_STEP.
///
/// Java executeStep routing:
///   end_turn → GotoLabel(end) + publish EndTurn + CheckForgo
///   end_player_action → GotoLabel(end) + publish EndPlayerAction
///   dispatch_player_action set + acting player present:
///     publish DispatchPlayerAction
///     if standing_up && !force_goto → NextStep (proceed to JumpUp/StandUp)
///     else → GotoLabel(end)
///   otherwise → Continue (waiting for command)
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.shared.StepInitSelecting`.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: forceGotoOnDispatch
    pub force_goto_on_dispatch: bool,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            force_goto_on_dispatch: false,
        }
    }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java start() only updates persistence — it does NOT call executeStep().
        // Emit the activation prompt so the agent knows which players are available.
        let team = if game.home_playing { &game.team_home } else { &game.team_away };
        let eligible: Vec<_> = team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
            .map(|p| (p.id.clone(), vec![PlayerAction::Move]))
            .collect();
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            Action::ActivatePlayer { player_id, player_action, block_defender_id } => {
                let pa = pac_to_player_action(*player_action);
                util_server_steps::change_player_action(game, player_id, pa, false);
                if let Some(def_id) = block_defender_id {
                    game.defender_id = Some(def_id.clone());
                }
                // Block/Blitz variants: go directly to label (forceGotoOnDispatch)
                self.force_goto_on_dispatch = matches!(
                    player_action,
                    PlayerActionChoice::Block | PlayerActionChoice::Blitz
                );
                self.dispatch_player_action = Some(pa);
                // Java: checkForStaller() called after CLIENT_ACTIVATE_PLAYER
                Self::check_for_staller(game);
            }
            // Java: CLIENT_USE_SKILL — selected skills that are resolved immediately (SKIP_STEP).
            Action::UseSkill { skill_id, use_skill: true } => {
                let acting_player_id = game.acting_player.player_id.clone();
                // Collect skill property booleans before any mutable borrow of game.
                let (gain_hail_mary, avoid_dodging, add_block_die) = {
                    let p = acting_player_id.as_deref().and_then(|id| game.player(id));
                    p.map(|player| (
                        player.has_skill_property(NamedProperties::CAN_GAIN_HAIL_MARY) && player.has_skill(*skill_id),
                        player.has_skill_property(NamedProperties::CAN_AVOID_DODGING) && player.has_skill(*skill_id),
                        player.has_skill_property(NamedProperties::CAN_ADD_BLOCK_DIE) && player.has_skill(*skill_id),
                    )).unwrap_or((false, false, false))
                };
                if gain_hail_mary {
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, GAIN_HAIL_MARY))
                    game.report_list.add(ReportSkillUse::new(
                        acting_player_id.clone(),
                        *skill_id,
                        true,
                        SkillUse::GAIN_HAIL_MARY,
                    ));
                } else if avoid_dodging {
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, AVOID_DODGING))
                    game.report_list.add(ReportSkillUse::new(
                        acting_player_id.clone(),
                        *skill_id,
                        true,
                        SkillUse::AVOID_DODGING,
                    ));
                } else if add_block_die {
                    // Java: getResult().addReport(new ReportSkillUse(skill, true, ADD_BLOCK_DIE)) — no player_id
                    game.report_list.add(ReportSkillUse::new(
                        None,
                        *skill_id,
                        true,
                        SkillUse::ADD_BLOCK_DIE,
                    ));
                }
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    /// Java: `checkForStaller()` — if game is already marked stalling (game.stalling==true),
    /// emit `ReportStallerDetected` for the acting player (unless they are forgone).
    fn check_for_staller(game: &mut Game) {
        if game.stalling {
            let player_id = game.acting_player.player_id.clone();
            if let Some(pid) = player_id {
                let forgo = game.acting_player.forgone;
                if !forgo {
                    // Java: if (actingPlayer.getPlayerAction() != PlayerAction.FORGO)
                    game.report_list.add(ReportStallerDetected::new(Some(pid)));
                }
            }
        }
    }

    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = &self.goto_label_on_end;

        if self.end_turn {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        if let Some(dispatch) = self.dispatch_player_action {
            if game.acting_player.player_id.is_some() {
                let standing_up = game.acting_player.standing_up;
                let outcome = StepOutcome::next()
                    .publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                if standing_up && !self.force_goto_on_dispatch {
                    return outcome;
                } else {
                    return StepOutcome::goto(label)
                        .publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                }
            }
        }
        // Waiting: build activation prompt
        let team = if game.home_playing { &game.team_home } else { &game.team_away };
        let eligible: Vec<_> = team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
            .map(|p| (p.id.clone(), vec![PlayerAction::Move]))
            .collect();
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }
}

/// Maps `PlayerActionChoice` (Rust engine action) to `PlayerAction` (model enum).
fn pac_to_player_action(pac: PlayerActionChoice) -> PlayerAction {
    match pac {
        PlayerActionChoice::Move => PlayerAction::Move,
        PlayerActionChoice::Blitz => PlayerAction::Blitz,
        PlayerActionChoice::Block => PlayerAction::Block,
        PlayerActionChoice::Stab => PlayerAction::Stab,
        PlayerActionChoice::Foul => PlayerAction::Foul,
        PlayerActionChoice::Pass => PlayerAction::Pass,
        PlayerActionChoice::HandOff => PlayerAction::HandOver,
        PlayerActionChoice::StandUp => PlayerAction::StandUp,
        PlayerActionChoice::StandUpBlitz => PlayerAction::StandUpBlitz,
        PlayerActionChoice::ThrowTeamMate => PlayerAction::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PlayerAction::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PlayerAction::Gaze,
        PlayerActionChoice::ThrowBomb => PlayerAction::ThrowBomb,
        PlayerActionChoice::Swoop => PlayerAction::Swoop,
        PlayerActionChoice::Punt => PlayerAction::Punt,
        PlayerActionChoice::BreatheFire => PlayerAction::BreatheFire,
        PlayerActionChoice::ProjectileVomit => PlayerAction::ProjectileVomit,
        PlayerActionChoice::SecureTheBall => PlayerAction::SecureTheBall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_cont() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_emits_activate_player_prompt() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(AgentPrompt::ActivatePlayer { .. })));
    }

    #[test]
    fn end_turn_returns_goto_label_with_end_turn_param() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn end_player_action_returns_goto_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        step.end_player_action = true;
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn activate_player_move_sets_dispatch_and_returns_goto_label() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))));
    }

    #[test]
    fn activate_player_block_sets_force_goto_on_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Block,
            block_defender_id: Some("def".into()),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.force_goto_on_dispatch);
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn use_skill_hail_mary_adds_report() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::report::report_id::ReportId;
        use std::collections::HashSet;
        let mut game = make_game();
        // Add a player with HailMaryPass (canGainHailMary property).
        game.team_home.players.push(Player {
            id: "hm1".into(), name: "HM".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: ffb_model::enums::SkillId::HailMaryPass, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.home_playing = true;
        game.acting_player.player_id = Some("hm1".into());
        let mut step = StepInitSelecting::new("end".into());
        step.handle_command(
            &Action::UseSkill { skill_id: ffb_model::enums::SkillId::HailMaryPass, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "expected SKILL_USE report for HailMaryPass");
    }

    #[test]
    fn staller_detected_report_added_when_stalling() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        use std::collections::HashSet;
        let mut game = make_game();
        game.stalling = true;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.forgone = false;
        game.team_home.players.push(Player {
            id: "p1".into(), name: "P1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("p1", ffb_model::types::FieldCoordinate::new(5, 5));
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::STALLER_DETECTED),
            "expected STALLER_DETECTED report when game.stalling is true");
    }

    #[test]
    fn pac_to_player_action_all_variants() {
        assert_eq!(pac_to_player_action(PlayerActionChoice::Move), PlayerAction::Move);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Block), PlayerAction::Block);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Foul), PlayerAction::Foul);
        assert_eq!(pac_to_player_action(PlayerActionChoice::HandOff), PlayerAction::HandOver);
    }
}
