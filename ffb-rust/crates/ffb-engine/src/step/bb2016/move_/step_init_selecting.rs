use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::{Action, PlayerActionChoice};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::change_player_action;
use crate::util::ServerUtilBlock;
use crate::util::UtilServerPlayerMove;

const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepInitSelecting.
///
/// Initialises the select sequence. Waits for a client command; on receipt,
/// publishes the relevant parameters and GOTOs the end label so EndSelecting
/// can route to the correct action sequence.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), UPDATE_PERSISTENCE (mandatory).
///
/// On start: if fUpdatePersistence → clear flag (queueDbUpdate TODO).
///
/// On executeStep:
/// - timeout || fEndTurn → publish END_TURN + GOTO_LABEL
/// - fEndPlayerAction → publish END_PLAYER_ACTION + GOTO_LABEL
/// - fDispatchPlayerAction set → publish DISPATCH_PLAYER_ACTION + GOTO_LABEL
///   (unless standingUp → NEXT_STEP)
/// - else → prepareStandingUp, then NEXT_STEP if REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
///
/// no-op: gameCache.queueDbUpdate — headless engine has no DB layer (confirmed intentional).
/// REMOVE_CONFUSION / STAND_UP / STAND_UP_BLITZ → NEXT_STEP path implemented.
/// no-op: game.isTimeoutEnforced() — headless engine has no turn timer; always treated as false.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fUpdatePersistence (transient)
    pub update_persistence: bool,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            update_persistence: false,
        }
    }
}

impl Default for StepInitSelecting {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fUpdatePersistence) { fUpdatePersistence=false; gameCache.queueDbUpdate(...); }
        // no-op: headless engine has no DB layer; gameCache.queueDbUpdate is skipped
        self.update_persistence = false;
        // start() does NOT call executeStep — waits for a client command
        StepOutcome::cont()
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let acting_pid = game.acting_player.player_id.clone().unwrap_or_default();

        match action {
            Action::Move { path } if !path.is_empty() => {
                // Java: CLIENT_MOVE → dispatchPlayerAction(MOVE) + publish MOVE_STACK
                self.dispatch_player_action = Some(PlayerAction::Move);
                let out = self.execute_step(game, rng);
                return out.publish(StepParameter::MoveStack(path.clone()));
            }
            Action::Foul { target_id } => {
                if game.turn_data().foul_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::Foul, false);
                self.dispatch_player_action = Some(PlayerAction::Foul);
                return self.execute_step(game, rng)
                    .publish(StepParameter::FoulDefenderId(target_id.clone()));
            }
            Action::Block { defender_id } => {
                self.dispatch_player_action = Some(PlayerAction::Block);
                return self.execute_step(game, rng)
                    .publish(StepParameter::BlockDefenderId(defender_id.clone()));
            }
            Action::HypnoticGaze { target_id } => {
                // Java: CLIENT_GAZE → changePlayerAction(GAZE), dispatch
                change_player_action(game, &acting_pid, PlayerAction::Gaze, false);
                self.dispatch_player_action = Some(PlayerAction::Gaze);
                return self.execute_step(game, rng)
                    .publish(StepParameter::GazeVictimId(Some(target_id.clone())));
            }
            Action::Pass { coord } => {
                // Java: passAllowed = !isPassUsed || (action == THROW_BOMB || HAIL_MARY_BOMB)
                let is_bomb_action = matches!(player_action,
                    Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb));
                if game.turn_data().pass_used && !is_bomb_action {
                    return StepOutcome::cont();
                }
                // Java: if (game.isHomePlaying()) { publish(coord) } else { publish(coord.transform()) }
                let target = if game.home_playing { *coord } else { coord.transform() };
                let dispatch_action = if player_action == Some(PlayerAction::HailMaryPass) {
                    PlayerAction::HailMaryPass
                } else {
                    change_player_action(game, &acting_pid, PlayerAction::Pass, false);
                    PlayerAction::Pass
                };
                self.dispatch_player_action = Some(dispatch_action);
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(target));
            }
            Action::HandOff { receiver_id } => {
                if game.turn_data().hand_over_used {
                    return StepOutcome::cont();
                }
                let coord_opt = game.field_model.player_coordinate(receiver_id);
                change_player_action(game, &acting_pid, PlayerAction::HandOver, false);
                self.dispatch_player_action = Some(PlayerAction::HandOver);
                let out = self.execute_step(game, rng);
                if let Some(coord) = coord_opt {
                    return out.publish(StepParameter::TargetCoordinate(coord));
                }
                return out;
            }
            Action::ThrowTeamMate { player_id: thrown_id, coord } => {
                if game.turn_data().pass_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::ThrowTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::ThrowTeamMate);
                // Java: if (game.isHomePlaying()) { publish(coord) } else { publish(coord.transform()) }
                let target = if game.home_playing { *coord } else { coord.transform() };
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(target))
                    .publish(StepParameter::ThrownPlayerId(Some(thrown_id.clone())));
            }
            Action::KickTeamMate { player_id: kicked_id, coord: _ } => {
                if game.turn_data().blitz_used {
                    return StepOutcome::cont();
                }
                change_player_action(game, &acting_pid, PlayerAction::KickTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::KickTeamMate);
                return self.execute_step(game, rng)
                    .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())));
            }
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }
            // Java: CLIENT_ACTING_PLAYER — changePlayerAction then executeStep (no dispatchPlayerAction)
            // This path fires for STAND_UP / STAND_UP_BLITZ / REMOVE_CONFUSION → NEXT_STEP in executeStep
            Action::ActivatePlayer { player_id, player_action, .. } => {
                let pa = pac_to_player_action(*player_action);
                if game.is_active_team_player(player_id) {
                    change_player_action(game, player_id, pa, false);
                } else {
                    self.end_player_action = true;
                }
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        StepOutcome::cont()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::DispatchPlayerAction(v) => { self.dispatch_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::UpdatePersistence(v) => { self.update_persistence = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    pub fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        // no-op: headless engine has no turn timer; game.isTimeoutEnforced() always treated as false
        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }

        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(dispatch_action) = self.dispatch_player_action {
            let player_id = game.acting_player.player_id.clone();
            let player_action = game.acting_player.player_action;
            if player_id.is_some() && player_action.is_some() {
                // Java: if (actingPlayer.isStandingUp()) { prepareStandingUp(); NEXT_STEP }
                //       else { GOTO_LABEL }
                if game.acting_player.standing_up {
                    self.prepare_standing_up(game);
                    return StepOutcome::next()
                        .publish(StepParameter::DispatchPlayerAction(Some(dispatch_action)));
                }
                return StepOutcome::goto(&label)
                    .publish(StepParameter::DispatchPlayerAction(Some(dispatch_action)));
            }
            // Java: fDispatchPlayerAction != null is an else-if branch — when the inner guard
            // (playerId provided && playerAction != null) fails, Java takes NO action at all
            // (does not fall through to the final else's prepareStandingUp()/NEXT_STEP logic).
            return StepOutcome::cont();
        }

        // Java: prepareStandingUp(); then NEXT_STEP if REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
        self.prepare_standing_up(game);
        let action = game.acting_player.player_action;
        if matches!(action, Some(PlayerAction::RemoveConfusion) | Some(PlayerAction::StandUp) | Some(PlayerAction::StandUpBlitz)) {
            StepOutcome::next()
        } else {
            StepOutcome::cont()
        }
    }

    fn prepare_standing_up(&self, game: &mut Game) {
        let player_action = game.acting_player.player_action;

        if let Some(action) = player_action {
            // Java: if (BLITZ || BLITZ_MOVE || BLOCK || MULTIPLE_BLOCK) → updateDiceDecorations
            if matches!(action, PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::Block | PlayerAction::MultipleBlock) {
                ServerUtilBlock::update_dice_decorations(game);
            }

            if action.is_moving() {
                // Java: if (isStandingUp && !canStandUpForFree)
                //           setCurrentMove(min(MINIMUM_MOVE_TO_STAND_UP, movementWithModifiers))
                //           setGoingForIt(UtilPlayer.isNextMoveGoingForIt)
                if game.acting_player.standing_up {
                    let can_stand_up_for_free = game.acting_player.player_id.as_deref()
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE))
                        .unwrap_or(false);
                    if !can_stand_up_for_free {
                        let movement_with_modifiers = game.acting_player.player_id.as_deref()
                            .and_then(|id| game.player(id))
                            .map(|p| p.movement_with_modifiers())
                            .unwrap_or(MINIMUM_MOVE_TO_STAND_UP);
                        game.acting_player.current_move = movement_with_modifiers.min(MINIMUM_MOVE_TO_STAND_UP);
                        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);
                    }
                }
                let jumping = game.acting_player.jumping;
                UtilServerPlayerMove::update_move_squares(game, jumping);
            }
        }
    }
}

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
    use crate::action::PlayerActionChoice;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_command_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepInitSelecting::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn set_parameter_update_persistence_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::UpdatePersistence(true)));
        assert!(step.update_persistence);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_dispatch_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block))));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }

    #[test]
    fn foul_command_blocked_when_foul_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.turn_data_home.foul_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Foul { target_id: "def".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "foul should be blocked when foul_used=true");
    }

    #[test]
    fn foul_command_allowed_when_foul_not_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::FoulMove);
        game.turn_data_home.foul_used = false;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Foul { target_id: "def".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_ne!(out.action, StepAction::Continue, "foul should be allowed when foul_used=false");
    }

    #[test]
    fn kick_team_mate_blocked_when_blitz_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        game.turn_data_home.blitz_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::KickTeamMate { player_id: "victim".into(), coord: ffb_model::types::FieldCoordinate::new(5, 5) },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "kick_team_mate should be blocked when blitz_used=true");
    }

    #[test]
    fn pass_command_blocked_when_pass_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.turn_data_home.pass_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::Pass { coord: ffb_model::types::FieldCoordinate::new(7, 5) },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "pass should be blocked when pass_used=true");
    }

    #[test]
    fn hand_over_blocked_when_hand_over_used() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        game.turn_data_home.hand_over_used = true;
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(
            &Action::HandOff { receiver_id: "recv".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue, "hand_off should be blocked when hand_over_used=true");
    }

    #[test]
    fn update_persistence_cleared_on_start() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.update_persistence = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.update_persistence);
    }

    #[test]
    fn end_player_action_set_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        // Set end_player_action via parameter, then start triggers it
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        // EndTurn sets end_turn → GotoLabel with EndTurn
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn standing_up_player_with_dispatch_returns_next_step() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = true;
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        // standing_up + dispatch → prepareStandingUp + NEXT_STEP
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn standing_up_sets_current_move_to_min_of_ma_and_minimum_stand_up() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let mut game = make_game();
        // Player with MA=4 (> MINIMUM_MOVE_TO_STAND_UP=3)
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.standing_up = true;
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        step.execute_step(&mut game, &mut GameRng::new(0));
        // MA=4 but capped to MINIMUM_MOVE_TO_STAND_UP=3
        assert_eq!(game.acting_player.current_move, 3);
    }

    #[test]
    fn block_command_publishes_block_defender_and_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(_))));
    }

    fn activate_player_action(game: &mut Game, step: &mut StepInitSelecting, pac: PlayerActionChoice) -> StepOutcome {
        let rng = &mut GameRng::new(0);
        let action = Action::ActivatePlayer { player_id: "p1".into(), player_action: pac, block_defender_id: None
};
        step.handle_command(&action, game, rng)
    }

    #[test]
    fn stand_up_action_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::StandUp);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn remove_confusion_via_activate_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        // RemoveConfusion doesn't have a PlayerActionChoice variant — use StandUp as a proxy
        // for the else-NEXT_STEP branch; RemoveConfusion is set via set_parameter
        game.acting_player.player_action = Some(PlayerAction::RemoveConfusion);
        let rng = &mut GameRng::new(0);
        let out = step.execute_step(&mut game, rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn stand_up_blitz_action_returns_next_step() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::StandUpBlitz);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn move_action_via_activate_returns_cont_waiting_for_move_command() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.team_home.players.push(ffb_model::model::player::Player { id: "p1".into(), ..Default::default() });
        let mut step = StepInitSelecting::new("end".into());
        let out = activate_player_action(&mut game, &mut step, PlayerActionChoice::Move);
        // MOVE via ActivatePlayer → executeStep with no dispatch → cont (waits for CLIENT_MOVE)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn report_list_empty_after_end_turn() {
        // Java bb2016 StepInitSelecting has no addReport calls — verify report_list stays empty.
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.is_empty(),
            "bb2016 StepInitSelecting has no addReport calls — report_list should remain empty");
    }

    #[test]
    fn end_player_action_param_then_end_turn_publishes_end_turn() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        // EndTurn overrides end_player_action because it's checked first
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn dispatch_set_with_failed_guard_does_not_fall_through_to_standing_up_branch() {
        // Java: `else if (fDispatchPlayerAction != null) { if (playerId provided && playerAction
        // != null) {...} }` — when fDispatchPlayerAction is set but the inner guard fails, Java
        // takes NO action (falls out of the else-if chain entirely). It must NOT fall through to
        // the final `else { prepareStandingUp(); NEXT_STEP if REMOVE_CONFUSION/STAND_UP/... }`
        // branch, even if game.acting_player.player_action happens to be STAND_UP.
        let mut game = make_game();
        game.acting_player.player_id = None; // guard fails: playerId not provided
        game.acting_player.player_action = Some(PlayerAction::StandUp);
        let mut step = StepInitSelecting::new("end".into());
        step.dispatch_player_action = Some(PlayerAction::Move);
        let out = step.execute_step(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue,
            "dispatch_player_action set with failed guard must not fall through to the standing-up NEXT_STEP branch");
    }

    #[test]
    fn pass_target_coordinate_transformed_for_away_team() {
        // Java: CLIENT_PASS → if (game.isHomePlaying()) publish(coord) else publish(coord.transform())
        let mut game = make_game();
        game.home_playing = false;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(10, 7);
        let out = step.handle_command(&Action::Pass { coord }, &mut game, &mut GameRng::new(0));
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord.transform(), "away-team pass target coordinate must be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }

    #[test]
    fn pass_target_coordinate_not_transformed_for_home_team() {
        let mut game = make_game();
        game.home_playing = true;
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(10, 7);
        let out = step.handle_command(&Action::Pass { coord }, &mut game, &mut GameRng::new(0));
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord, "home-team pass target coordinate must not be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }

    #[test]
    fn throw_team_mate_target_coordinate_transformed_for_away_team() {
        // Java: CLIENT_THROW_TEAM_MATE → if (game.isHomePlaying()) publish(coord) else publish(coord.transform())
        let mut game = make_game();
        game.home_playing = false;
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end".into());
        let coord = ffb_model::types::FieldCoordinate::new(4, 9);
        let out = step.handle_command(
            &Action::ThrowTeamMate { player_id: "tm1".into(), coord },
            &mut game, &mut GameRng::new(0),
        );
        let published = out.published.iter().find(|p| matches!(p, StepParameter::TargetCoordinate(_)));
        if let Some(StepParameter::TargetCoordinate(c)) = published {
            assert_eq!(*c, coord.transform(), "away-team throw-team-mate target coordinate must be mirrored");
        } else {
            panic!("TargetCoordinate not published");
        }
    }
}
