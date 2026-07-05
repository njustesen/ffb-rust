/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepSelectBlitzTarget` (BB2020).
///
/// Prompts the active coach to select a target for the blitz action, or handles special-skill
/// sequences (Treacherous, RaidingParty, etc.) and WisdomOfTheWhiteDwarf.
///
/// DEFERRED items:
///  - EndPlayerAction / EndTurn cancel path (clear stack + EndPlayerAction sequence) not translated.
///  - TargetSelectionState (skip / cancel) not translated.
///  - ReportSelectBlitzTarget: now wired as GameEvent::SelectBlitzTarget.
///  - usedSkill skill-enhancement sequences not translated (Treacherous, RaidingParty,
///    BalefulHex, LookIntoMyEyes, BlackInk, ThenIStartedBlastin, CatchOfTheDay, AutoGazeZoat).
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepSelectBlitzTarget`.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::target_selection_state::TargetSelectionState;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::TurnMode;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepSelectBlitzTarget {
    pub goto_label_on_end: String,
    pub selected_player_id: Option<String>,
    pub confirmed: bool,
    pub end_player_action: bool,
    pub end_turn: bool,
    /// Skill name used (from UseSkill action that triggered a TODO-generator).
    pub used_skill: Option<String>,
}

impl StepSelectBlitzTarget {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            selected_player_id: None,
            confirmed: false,
            end_player_action: false,
            end_turn: false,
            used_skill: None,
        }
    }
}

impl Default for StepSelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

impl Step for StepSelectBlitzTarget {
    fn id(&self) -> StepId { StepId::SelectBlitzTarget }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.selected_player_id = Some(player_id.clone());
            }
            Action::EndTurn => {
                // Only end turn if it's from the active team (simplified: always accept).
                self.end_turn = true;
            }
            Action::UseSkill { skill_id, use_skill } if *use_skill => {
                let skill_name = format!("{skill_id:?}");
                // DEFERRED(select_blitz_target): skill-enhancement generators not translated.
                // Treacherous, RaidingParty, BalefulHex, LookIntoMyEyes, BlackInk,
                // ThenIStartedBlastin, CatchOfTheDay, AutoGazeZoat all push sequences
                // here and return StepAction::Repeat. For now fall through to execute_step.
                self.used_skill = Some(skill_name);
            }
            Action::UseSkill { .. } => {
                // use_skill = false: declined, just execute step
            }
            Action::Acknowledge => {
                self.confirmed = true;
            }
            // CLIENT_USE_TEAM_MATES_WISDOM → push WisdomOfTheWhiteDwarf sequence + Repeat
            // DEFERRED(select_blitz_target): No dedicated Action variant for this yet.
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepSelectBlitzTarget {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Case 1: End player action or end turn triggered
        if self.end_player_action || self.end_turn {
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
            // DEFERRED(select_blitz_target): clear stack and push EndPlayerAction sequence not translated.
            return StepOutcome::next();
        }

        let acting_player_id = game.acting_player.player_id.clone();

        // Case 2: No player selected yet
        if self.selected_player_id.is_none() {
            if self.has_standing_opponents(game) {
                game.turn_mode = TurnMode::SelectBlitzTarget;
                // DEFERRED(select_blitz_target): dialog prompt not translated.
                return StepOutcome::cont();
            } else {
                // Java: setTargetSelectionState(new TargetSelectionState(null).skip()) — no targets available
                // TargetSelectionState has no skip() method; use cancel() as the closest equivalent.
                let mut ts = TargetSelectionState::default();
                ts.cancel();
                game.field_model.target_selection_state = Some(ts);
                return StepOutcome::next();
            }
        }

        // Case 3: A player was selected
        let selected_id = self.selected_player_id.clone().unwrap();
        let acting_id = acting_player_id.unwrap_or_default();

        if selected_id == acting_id {
            // Selected the acting player itself
            let has_acted = game.acting_player.has_acted;
            if has_acted && !self.confirmed {
                // DEFERRED(select_blitz_target): confirm dialog not translated.
                return StepOutcome::cont();
            } else {
                // Cancel blitz target: restore last turn mode and goto end label
                game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
                // Java: setTargetSelectionState(new TargetSelectionState(actingPlayerId).cancel())
                let mut ts = TargetSelectionState::default();
                ts.cancel();
                game.field_model.target_selection_state = Some(ts);
                if self.goto_label_on_end.is_empty() {
                    return StepOutcome::next();
                }
                return StepOutcome::goto(&self.goto_label_on_end);
            }
        }

        // Check if selected player is on inactive (opposing) team
        let is_on_inactive_team = {
            let inactive = game.inactive_team();
            inactive.player(&selected_id).is_some()
        };

        if is_on_inactive_team {
            // Selected an opponent → valid blitz target
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
            // Java: getFieldModel().getPlayerState(targetPlayer).addSelectedBlitzTarget()
            if let Some(state) = game.field_model.player_state(&selected_id) {
                game.field_model.set_player_state(&selected_id, state.add_selected_blitz_target());
            }
            // Java: new TargetSelectionState(selectedPlayerId) → conditional commit → select()
            let mut ts = TargetSelectionState::new(selected_id.clone());
            if game.acting_player.has_acted {
                ts.commit();
            }
            ts.select();
            game.field_model.target_selection_state = Some(ts);
            // DEFERRED(select_blitz_target): usedSkill skill-enhancements not translated.
            let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
            StepOutcome::next().with_event(GameEvent::SelectBlitzTarget {
                attacker_id,
                defender_id: selected_id,
            })
        } else {
            // Selected own teammate (not acting player): restore last turn mode
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
            StepOutcome::next()
        }
    }

    /// Java: hasStandingOpponents — true if any opponent is on the field and can be blocked.
    fn has_standing_opponents(&self, game: &Game) -> bool {
        let inactive = game.inactive_team();
        inactive.players.iter().any(|player| {
            // Check if player is on the field (x: 1..=26, y: 1..=15)
            if let Some(coord) = game.field_model.player_coordinate(&player.id) {
                if coord.x >= 1 && coord.x <= 26 && coord.y >= 1 && coord.y <= 15 {
                    // Check if player state can be blocked
                    if let Some(state) = game.field_model.player_state(&player.id) {
                        return state.can_be_blocked();
                    }
                }
            }
            false
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PS_KNOCKED_OUT};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_no_opponents_returns_next() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No opponents on field -> NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_with_opponent_standing_returns_continue() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        let mut game = make_game();
        game.home_playing = true;

        // Add a standing opponent (away team player)
        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));

        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Has standing opponent -> Continue (waiting for selection)
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::SelectBlitzTarget);
    }

    #[test]
    fn end_turn_returns_next() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_player_action_returns_next() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn selecting_opponent_returns_next() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());
        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some("away_p99".into());
        // away_p99 not in home team, but game.inactive_team() is away
        // We won't find them in away.players but that's OK - the is_on_inactive_team check
        // will be false in that case, falls through to "selected own teammate" path
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepSelectBlitzTarget::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("MY_LABEL".into())));
        assert_eq!(step.goto_label_on_end, "MY_LABEL");
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepSelectBlitzTarget::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepSelectBlitzTarget::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn handle_end_turn_action_sets_end_turn() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.end_turn);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_acknowledge_sets_confirmed() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some("actor".into()); // selected self
        step.confirmed = false;
        game.acting_player.has_acted = true;
        // First call without confirm -> Continue
        let out1 = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(step.confirmed);
        // After confirm, goto_label_on_end is empty -> NextStep
        assert_eq!(out1.action, StepAction::NextStep);
    }

    #[test]
    fn selecting_self_with_no_action_yet_returns_goto() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_acted = false;
        let mut step = StepSelectBlitzTarget::new();
        step.goto_label_on_end = "END".into();
        step.selected_player_id = Some("actor".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // has_acted=false and confirmed=false -> goto
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn knocked_out_opponent_does_not_trigger_continue() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        let mut game = make_game();
        game.home_playing = true;

        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        // KO state cannot be blocked
        game.field_model.set_player_state(&pid, PlayerState::new(PS_KNOCKED_OUT));

        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No blockable opponent -> NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_opponents_sets_target_selection_state_canceled() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        step.start(&mut game, &mut GameRng::new(0));
        // No opponents → cancel path → target_selection_state should be Some(canceled)
        let ts = game.field_model.target_selection_state.as_ref().unwrap();
        assert!(ts.is_canceled());
    }

    #[test]
    fn selecting_opponent_with_acted_sets_target_selection_state_selected_and_committed() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::target_selection_state::TargetSelectionState;

        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());
        game.acting_player.has_acted = true;

        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));

        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some(pid.clone());
        step.start(&mut game, &mut GameRng::new(0));

        let ts = game.field_model.target_selection_state.as_ref().unwrap();
        assert!(ts.is_selected());
        assert!(ts.is_committed());
        assert_eq!(ts.get_selected_player_id().map(|id| id.as_str()), Some(pid.as_str()));
    }

    #[test]
    fn selecting_opponent_without_acted_does_not_commit_target_selection() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::target_selection_state::TargetSelectionState;

        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());
        game.acting_player.has_acted = false;

        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));

        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some(pid.clone());
        step.start(&mut game, &mut GameRng::new(0));

        let ts = game.field_model.target_selection_state.as_ref().unwrap();
        assert!(ts.is_selected());
        assert!(!ts.is_committed());
    }

    #[test]
    fn selecting_opponent_adds_selected_blitz_target_bit() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};

        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("home_p1".into());

        let pid = "away_p1".to_string();
        game.team_away.players.push(Player {
            id: pid.clone(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING));

        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some(pid.clone());
        step.start(&mut game, &mut GameRng::new(0));

        let state = game.field_model.player_state(&pid).unwrap();
        assert!(state.is_selected_blitz_target());
    }

    #[test]
    fn selecting_self_sets_target_selection_state_canceled() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.has_acted = false;
        let mut step = StepSelectBlitzTarget::new();
        step.goto_label_on_end = "END".into();
        step.selected_player_id = Some("actor".into());
        step.start(&mut game, &mut GameRng::new(0));
        let ts = game.field_model.target_selection_state.as_ref().unwrap();
        assert!(ts.is_canceled());
    }

    #[test]
    fn selecting_opponent_emits_select_blitz_target_event() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("attacker".into());
        game.team_away.players.push(Player {
            id: "defender".into(), name: "Defender".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate("defender", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("defender", PlayerState::new(PS_STANDING));
        let mut step = StepSelectBlitzTarget::new();
        step.selected_player_id = Some("defender".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::SelectBlitzTarget {
            defender_id, ..
        } if defender_id == "defender")));
    }
}
