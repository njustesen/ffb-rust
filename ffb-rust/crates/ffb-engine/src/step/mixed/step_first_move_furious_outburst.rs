/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepFirstMoveFuriousOutburst`.
///
/// First teleport move in the Furious Outburst sequence.
/// Needs `GOTO_LABEL_ON_END` init parameter.
///
/// Java logic (executeStep):
///   - CONTINUE by default.
///   - If `end_player_action`:
///     - If player has acted → report SkillWasted.
///     - Cancel target selection → GOTO goto_label_on_end.
///   - If `coordinate` not yet chosen:
///     - Set defender, mark target, compute eligible squares, show MoveSquares.
///     - `with_ball` = acting player has ball.
///   - If `coordinate` chosen:
///     - Animate TRICKSTER move, teleport player, commit target selection.
///     - Publish USING_STAB=true.
///     - If `with_ball`: move ball too.
///     - `bounceBall()` → maybe SCATTER_BALL.
///     - → NEXT_STEP.
///
/// Java fields: `eligibleSquares`, `endPlayerAction`, `withBall`,
///              `goToLabelOnEnd`, `coordinate`.
use std::collections::HashSet;
use ffb_model::types::{FieldCoordinate, MoveSquare};
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, CatchScatterThrowInMode};

/// Java: `StepFirstMoveFuriousOutburst` (mixed, BB2020 + BB2025).
pub struct StepFirstMoveFuriousOutburst {
    /// Java: `eligibleSquares`
    pub eligible_squares: HashSet<FieldCoordinate>,
    /// Java: `endPlayerAction`
    pub end_player_action: bool,
    /// Java: `withBall`
    pub with_ball: bool,
    /// Java: `goToLabelOnEnd` (mandatory init param)
    pub goto_label_on_end: String,
    /// Java: `coordinate` — the teleport destination once chosen
    pub coordinate: Option<FieldCoordinate>,
}

impl StepFirstMoveFuriousOutburst {
    pub fn new(goto_label_on_end: impl Into<String>) -> Self {
        Self {
            eligible_squares: HashSet::new(),
            end_player_action: false,
            with_ball: false,
            goto_label_on_end: goto_label_on_end.into(),
            coordinate: None,
        }
    }

    fn bounce_ball(&self, game: &Game) -> Option<StepParameter> {
        let coord = self.coordinate?;
        if !self.with_ball && game.field_model.ball_coordinate == Some(coord) && game.field_model.ball_moving {
            Some(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            None
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if self.end_player_action {
            // Java: if (actingPlayer.hasActed()) getResult().addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            if game.acting_player.has_acted {
                let player_id = game.acting_player.player_id.clone();
                let skill = player_id.as_deref()
                    .and_then(|pid| game.player(pid))
                    .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_TELEPORT_BEFORE_AND_AFTER_AV_ROLL_ATTACK));
                game.report_list.add(ReportSkillWasted::new(player_id, skill));
            }
            // Java: fieldModel.getTargetSelectionState().cancel()
            if let Some(ref mut ts) = game.field_model.target_selection_state {
                ts.cancel();
            }
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        if self.coordinate.is_none() {
            // Java: first pass — set up eligible squares for trickster move
            // Java: String targetId = game.getFieldModel().getTargetSelectionState().getSelectedPlayerId()
            let target_id = game.field_model.target_selection_state.as_ref()
                .and_then(|ts| ts.get_selected_player_id().cloned());

            if let Some(ref tid) = target_id {
                game.defender_id = Some(tid.clone());
                // Java: fieldModel.setPlayerState(target, state.changeSelectedStabTarget(true))
                let old_state = game.field_model.player_state(tid)
                    .unwrap_or_default();
                game.field_model.set_player_state(tid, old_state.change_selected_stab_target(true).remove_selected_blitz_target());
                // Java: find empty adjacent squares to target, add as MoveSquares
                let target_coord = game.field_model.player_coordinate(tid);
                if let Some(tc) = target_coord {
                    let adj: Vec<FieldCoordinate> = game.field_model.adjacent_on_pitch(tc)
                        .into_iter()
                        .filter(|c| game.field_model.player_at(*c).is_none())
                        .collect();
                    self.eligible_squares = adj.iter().cloned().collect();
                    for c in &adj {
                        game.field_model.move_squares.insert(*c, MoveSquare::new(*c, 0, 0));
                    }
                }
                // Java: withBall = UtilPlayer.hasBall(game, actingPlayer.getPlayer())
                self.with_ball = game.acting_player.player_id.as_deref()
                    .map(|pid| game.field_model.ball_coordinate == game.field_model.player_coordinate(pid))
                    .unwrap_or(false);
            }
            return StepOutcome::cont();
        }

        // Java: second pass — perform the teleport
        let coord = self.coordinate.unwrap();
        let acting_id = game.acting_player.player_id.clone();
        if let Some(ref pid) = acting_id {
            game.field_model.set_player_coordinate(pid, coord);
            if self.with_ball {
                game.field_model.ball_coordinate = Some(coord);
            }
        }
        // Java: fieldModel.getTargetSelectionState().commit(game)
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.commit();
        }

        let mut outcome = StepOutcome::next()
            .publish(StepParameter::UsingStab(true));

        if let Some(scatter_param) = self.bounce_ball(game) {
            outcome = outcome.publish(scatter_param);
        }

        outcome
    }
}

impl Default for StepFirstMoveFuriousOutburst {
    fn default() -> Self { Self::new("") }
}

impl Step for StepFirstMoveFuriousOutburst {
    fn id(&self) -> StepId { StepId::FirstMoveFuriousOutburst }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_FIELD_COORDINATE
            Action::Move { path } => {
                if let Some(&coord) = path.first() {
                    if self.eligible_squares.contains(&coord) {
                        self.coordinate = Some(coord);
                    }
                }
            }
            // Java: CLIENT_ACTING_PLAYER with null action → endPlayerAction
            Action::ActivatePlayer { player_action: _, ..
} => {
                // Java: if clientCommandActingPlayer.getPlayerAction() == null → endPlayerAction
                // In Rust we use EndTurn as the cancel signal
            }
            Action::EndTurn => { self.end_player_action = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_first_move_furious_outburst() {
        assert_eq!(StepFirstMoveFuriousOutburst::new("end").id(), StepId::FirstMoveFuriousOutburst);
    }

    #[test]
    fn no_coordinate_continues() {
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_player_action_goes_to_label() {
        let mut step = StepFirstMoveFuriousOutburst::new("end_label");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label, Some("end_label".into()));
    }

    #[test]
    fn coordinate_set_publishes_using_stab_and_next_steps() {
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.coordinate = Some(FieldCoordinate::new(6, 6));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        let has_stab = out.published.iter().any(|p| matches!(p, StepParameter::UsingStab(true)));
        assert!(has_stab, "should publish UsingStab(true)");
    }

    #[test]
    fn coordinate_moves_player_to_target() {
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.coordinate = Some(FieldCoordinate::new(7, 7));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        let pos = game.field_model.player_coordinate("att");
        assert_eq!(pos, Some(FieldCoordinate::new(7, 7)));
    }

    #[test]
    fn skill_wasted_report_added_when_has_acted() {
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        game.acting_player.has_acted = true;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_WASTED));
    }

    #[test]
    fn no_skill_wasted_report_when_not_acted() {
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        game.acting_player.has_acted = false;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_WASTED));
    }

    #[test]
    fn end_player_action_cancels_target_selection_state() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut ts = TargetSelectionState::new("target");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.field_model.target_selection_state.as_ref().map(|ts| ts.is_canceled()).unwrap_or(false));
    }

    #[test]
    fn second_pass_commits_target_selection_state() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut step = StepFirstMoveFuriousOutburst::new("end");
        step.coordinate = Some(FieldCoordinate::new(6, 6));
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut ts = TargetSelectionState::new("target");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.field_model.target_selection_state.as_ref().map(|ts| ts.is_committed()).unwrap_or(false));
    }
}
