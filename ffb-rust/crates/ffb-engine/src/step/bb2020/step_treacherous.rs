use ffb_model::enums::{ApothecaryMode, PlayerAction, SkillId};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_stab::InjuryTypeStab;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury;

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepTreacherous` (BB2020).
///
/// Resolves the Treacherous skill: secretly stabs a teammate who carries the ball.
///
/// Differs from BB2025 in `markActionUsed`:
///  - `ThrowTeamMate` → `pass_used` (not `ttm_used`)
///  - Includes `KickEmBlitz` → `blitz_used` and `KickTeamMate` → `ktm_used`
///  - No `PUNT`/`PUNT_MOVE` case
///
/// Publishes `DropPlayerContext` (not raw `InjuryResult`) when the stab lands.
pub struct StepTreacherous {
    /// Java: `endPlayerAction`
    pub end_player_action: bool,
    /// Java: `endTurn`
    pub end_turn: bool,
    /// Java: `goToLabelOnFailure`
    pub goto_label_on_failure: String,
}

impl StepTreacherous {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false, goto_label_on_failure: String::new() }
    }
}

impl Default for StepTreacherous {
    fn default() -> Self { Self::new() }
}

impl Step for StepTreacherous {
    fn id(&self) -> StepId { StepId::Treacherous }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)            => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)    => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepTreacherous {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canStabTeamMateForBall)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::Treacherous) && !p.used_skills.contains(&SkillId::Treacherous))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        // Java: markActionUsed(game, actingPlayer)
        Self::mark_action_used(game);

        // Java: if (endTurn || endPlayerAction) → ReportSkillWasted + GOTO_LABEL + markSkillUsed
        if self.end_turn || self.end_player_action {
            Self::mark_skill_used(game, &player_id);
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        // Java: treacherousTarget — find adjacent active-team player with ball
        // Condition: !hasActedIgnoringNegativeTraits() || justStoodUp()
        let can_try = !game.acting_player.has_acted || game.acting_player.standing_up;
        let actor_coord = game.field_model.player_coordinate(&player_id);

        let outcome = if can_try {
            let target = actor_coord.and_then(|ac| Self::find_treacherous_target(game, &player_id, ac));

            if let Some(target_id) = target {
                // Java: fieldModel.setBallCoordinate(actingPlayer.getPlayerCoordinate())
                if let Some(ac) = actor_coord {
                    game.field_model.ball_coordinate = Some(ac);
                }

                let defender_coord = game.field_model.player_coordinate(&target_id)
                    .unwrap_or(FieldCoordinate::new(0, 0));

                // Java: UtilServerInjury.handleInjury(step, InjuryTypeStab(true,true), attacker, defender, coord, null, null, DEFENDER)
                let mut injury_type = InjuryTypeStab::new();
                let injury_result = handle_injury(
                    game, rng, &mut injury_type,
                    Some(player_id.as_str()), &target_id,
                    defender_coord, None, None,
                    ApothecaryMode::Defender,
                );

                // Java: publishParameter(DROP_PLAYER_CONTEXT, new DropPlayerContext(injuryResult, false, false, null, playerId, DEFENDER, false))
                let dpc = DropPlayerContext::with_injury(injury_result, target_id, ApothecaryMode::Defender, false);
                StepOutcome::next().publish(StepParameter::DropPlayerContext(Box::new(dpc)))
            } else {
                StepOutcome::next()
            }
        } else {
            StepOutcome::next()
        };

        // Java: actingPlayer.markSkillUsed(skill)
        Self::mark_skill_used(game, &player_id);

        outcome
    }

    /// Java: `treacherousTarget` — find adjacent active-team player who carries the ball.
    fn find_treacherous_target(
        game: &Game,
        actor_id: &str,
        actor_coord: FieldCoordinate,
    ) -> Option<String> {
        game.active_team().players.iter()
            .filter(|p| p.id != actor_id)
            .find(|p| {
                if let Some(pc) = game.field_model.player_coordinate(&p.id) {
                    game.field_model.ball_coordinate == Some(pc) && pc.is_adjacent(actor_coord)
                } else {
                    false
                }
            })
            .map(|p| p.id.clone())
    }

    /// Java: `markActionUsed` (BB2020 variant).
    /// ThrowTeamMate/KickTeamMate map differently than BB2025; no PUNT case.
    fn mark_action_used(game: &mut Game) {
        let action = game.acting_player.player_action;
        let turn = game.turn_data_mut();
        match action {
            Some(PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::KickEmBlitz) => turn.blitz_used = true,
            Some(PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove) => turn.ktm_used = true,
            Some(
                PlayerAction::Pass | PlayerAction::PassMove |
                PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove
            ) => turn.pass_used = true,
            Some(PlayerAction::HandOver | PlayerAction::HandOverMove) => turn.hand_over_used = true,
            Some(PlayerAction::Foul | PlayerAction::FoulMove) => turn.foul_used = true,
            _ => {}
        }
    }

    fn mark_skill_used(game: &mut Game, player_id: &str) {
        let is_home = game.team_home.player(player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(player_id) {
                p.used_skills.insert(SkillId::Treacherous);
            }
        } else if let Some(p) = game.team_away.player_mut(player_id) {
            p.used_skills.insert(SkillId::Treacherous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_player(id: &str, skill: Option<SkillId>) -> Player {
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skill.map(|s| vec![SkillWithValue { skill_id: s, value: None }])
                .unwrap_or_default(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_game_with_treacherous() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::Treacherous)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.has_acted = false;
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_with_treacherous();
        game.team_home.players.last_mut().unwrap().starting_skills.clear();
        let mut step = StepTreacherous::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label_and_marks_used() {
        let (mut game, actor_id) = make_game_with_treacherous();
        let mut step = StepTreacherous::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::Treacherous));
    }

    #[test]
    fn no_adjacent_target_returns_next_step() {
        let (mut game, actor_id) = make_game_with_treacherous();
        let mut step = StepTreacherous::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::Treacherous));
    }

    #[test]
    fn target_with_ball_publishes_drop_player_context() {
        let (mut game, _) = make_game_with_treacherous();
        let mate_id = "mate".to_string();
        game.team_home.players.push(make_player(&mate_id, None));
        let adj = FieldCoordinate::new(11, 7);
        game.field_model.set_player_coordinate(&mate_id, adj);
        game.field_model.set_player_state(&mate_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.ball_coordinate = Some(adj);

        let mut step = StepTreacherous::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(10, 7)));
    }

    #[test]
    fn mark_action_used_ttm_sets_pass_used() {
        let (mut game, _) = make_game_with_treacherous();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        StepTreacherous::mark_action_used(&mut game);
        assert!(game.turn_data_mut().pass_used);
        assert!(!game.turn_data_mut().ttm_used);
    }

    #[test]
    fn mark_action_used_kick_team_mate_sets_ktm() {
        let (mut game, _) = make_game_with_treacherous();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        StepTreacherous::mark_action_used(&mut game);
        assert!(game.turn_data_mut().ktm_used);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepTreacherous::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }
}
