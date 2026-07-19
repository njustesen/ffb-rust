/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepInitThrowTeamMate`.
///
/// Initialises the Throw Team-Mate sequence:
/// - EndTurn / EndPlayerAction → publish + goto end label.
/// - CLIENT_ACTING_PLAYER with player_id → changePlayerAction.
/// - CLIENT_ACTING_PLAYER without player_id → endPlayerAction = true.
/// - ThrowTeamMate with player chosen (and no target) → select thrown player,
///   publish ThrownPlayerId/ThrownPlayerState/OldDefenderState/ThrownPlayerCoordinate/
///   ThrownPlayerHasBall; change player state to PICKED_UP.
/// - ThrowTeamMate with target set → set pass coordinate + range ruler.
///
/// BB2020 differences vs BB2016:
///  - Also publishes OLD_DEFENDER_STATE (original player state before PICKED_UP).
///  - `kicked` flag: when set, changes acting player action to KickTeamMate instead of ThrowTeamMate.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), TARGET_COORDINATE (opt),
///   THROWN_PLAYER_ID (opt), IS_KICKED_PLAYER (opt).
///
/// client-only: UtilRangeRuler.createRangeRuler — range ruler is client-side display only.
use ffb_model::enums::{PlayerAction, PS_PICKED_UP};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2020::pass_mechanic::PassMechanic as Bb2020PassMechanic;
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitThrowTeamMate` (bb2020/ttm).
pub struct StepInitThrowTeamMate {
    /// Java: `gotoLabelOnEnd` — init param (mandatory).
    goto_label_on_end: String,
    /// Java: `thrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `targetCoordinate`
    target_coordinate: Option<FieldCoordinate>,
    /// Java: `endTurn`
    end_turn: bool,
    /// Java: `endPlayerAction`
    end_player_action: bool,
    /// Java: `kicked` — optional init param (BB2020: KickTeamMate vs ThrowTeamMate).
    kicked: bool,
}

impl StepInitThrowTeamMate {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            thrown_player_id: None,
            target_coordinate: None,
            end_turn: false,
            end_player_action: false,
            kicked: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if self.end_turn {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(thrown_id) = &self.thrown_player_id {
            if let Some(target) = self.target_coordinate {
                // Phase 2: target chosen → set pass coordinate.
                game.pass_coordinate = Some(target);
                // Java: game.getFieldModel().setRangeRuler(UtilRangeRuler.createRangeRuler(...));
                //       if (rangeRuler != null) NEXT_STEP;  (else: falls through, no next action set → waits)
                // The ruler object itself is client-side display only, but its null-vs-not-null gate
                // (legal TTM throw distance) is real: an out-of-range target must NOT advance.
                let thrower_coord = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id));
                let pass_mechanic = Bb2020PassMechanic::new();
                let in_range = pass_mechanic
                    .find_passing_distance(game, thrower_coord, Some(target), true)
                    .is_some();
                if in_range {
                    return StepOutcome::next();
                }
                return StepOutcome::cont();
            } else {
                // Phase 1: player chosen — set up defender, publish parameters.
                let thrown_id = thrown_id.clone();
                if game.player(&thrown_id).is_some() {
                    game.defender_id = Some(thrown_id.clone());
                    let old_state = game.field_model.player_state(&thrown_id).unwrap_or_default();
                    let thrown_state = old_state.change_base(PS_PICKED_UP);
                    let coord = game.field_model.player_coordinate(&thrown_id);

                    let has_ball = coord
                        .zip(game.field_model.ball_coordinate)
                        .map(|(pc, bc)| pc == bc && !game.field_model.ball_moving)
                        .unwrap_or(false);

                    game.field_model.set_player_state(&thrown_id, thrown_state);

                    // BB2020: KickTeamMate vs ThrowTeamMate action.
                    let action = if self.kicked { PlayerAction::KickTeamMate } else { PlayerAction::ThrowTeamMate };
                    game.acting_player.player_action = Some(action);

                    let mut outcome = StepOutcome::next()
                        .publish(StepParameter::ThrownPlayerId(Some(thrown_id)))
                        .publish(StepParameter::ThrownPlayerState(thrown_state))
                        .publish(StepParameter::OldDefenderState(old_state))
                        .publish(StepParameter::ThrownPlayerHasBall(has_ball));
                    if let Some(c) = coord {
                        outcome = outcome.publish(StepParameter::ThrownPlayerCoordinate(Some(c)));
                    }
                    return outcome;
                }
            }
        }
        StepOutcome::next()
    }
}

impl Default for StepInitThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitThrowTeamMate {
    fn id(&self) -> StepId { StepId::InitThrowTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            Action::ThrowTeamMate { player_id, coord } => {
                // Java: first call with no target → sets player_id;
                //       second call (player already set + target not null) → sets target.
                if self.thrown_player_id.is_some() && *coord != FieldCoordinate::new(0, 0) {
                    self.target_coordinate = Some(*coord);
                } else {
                    self.thrown_player_id = Some(player_id.clone());
                    self.target_coordinate = None;
                }
                self.execute_step(game)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)          => { self.goto_label_on_end = s.clone(); true }
            StepParameter::ThrownPlayerId(Some(id))   => { self.thrown_player_id = Some(id.clone()); true }
            StepParameter::ThrownPlayerId(None)       => { self.thrown_player_id = None; true }
            StepParameter::TargetCoordinate(c)        => { self.target_coordinate = Some(*c); true }
            StepParameter::EndTurn(v)                 => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)         => { self.end_player_action = *v; true }
            StepParameter::IsKickedPlayer(v)          => { self.kicked = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender, PlayerAction};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_init_throw_team_mate() {
        assert_eq!(StepInitThrowTeamMate::new().id(), StepId::InitThrowTeamMate);
    }

    #[test]
    fn end_turn_publishes_and_gotos_label() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::EndTurn(true));
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn selecting_player_publishes_state_sets_picked_up_and_old_state() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        let mut game = make_game();
        add_player(&mut game, "p1");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerState(_))));
        // BB2020: also publishes OldDefenderState.
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::OldDefenderState(_))));
        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_PICKED_UP);
    }

    #[test]
    fn kicked_flag_sets_kick_team_mate_action() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::IsKickedPlayer(true));
        let mut game = make_game();
        add_player(&mut game, "p1");
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::KickTeamMate));
    }

    #[test]
    fn no_kicked_flag_sets_throw_team_mate_action() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        let mut game = make_game();
        add_player(&mut game, "p1");
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::ThrowTeamMate));
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut step = StepInitThrowTeamMate::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    // Java: `if (targetCoordinate != null) { ... if (rangeRuler != null) NEXT_STEP; }` — an
    // out-of-range target (findPassingDistance returns null) must NOT advance to NEXT_STEP.
    // Previously the Rust code always returned NEXT_STEP regardless of distance, which is a
    // real behavioral divergence: it would let a player select a target beyond any legal
    // TTM throw distance and still proceed.
    #[test]
    fn out_of_range_target_does_not_advance() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::TargetCoordinate(FieldCoordinate::new(20, 0)));
        let mut game = make_game();
        add_player(&mut game, "thrower");
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(0, 0));
        game.acting_player.player_id = Some("thrower".into());
        add_player(&mut game, "p1");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::Continue),
            "out-of-range target must not advance to NEXT_STEP, got {:?}", out.action);
        // Java: game.setPassCoordinate(targetCoordinate) still happens unconditionally.
        assert_eq!(game.pass_coordinate, Some(FieldCoordinate::new(20, 0)));
    }

    #[test]
    fn in_range_target_advances_to_next_step() {
        let mut step = StepInitThrowTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into())));
        step.set_parameter(&StepParameter::TargetCoordinate(FieldCoordinate::new(6, 5)));
        let mut game = make_game();
        add_player(&mut game, "thrower");
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("thrower".into());
        add_player(&mut game, "p1");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep),
            "in-range target should advance to NEXT_STEP, got {:?}", out.action);
    }
}
