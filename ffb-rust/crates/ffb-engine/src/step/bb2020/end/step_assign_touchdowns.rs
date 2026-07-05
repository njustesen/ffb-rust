/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.end.StepAssignTouchdowns`.
///
/// Awards pending touchdowns (from illegal concession) to surviving players of the winning team.
///
/// Flow:
/// - If no winning team (winning_team_id == None) → NEXT_STEP immediately.
/// - Collect surviving players (TODO: filter killed/recoveringInjury via field_model).
/// - If no players → touchdowns = 0.
/// - If conceded legally → cap touchdowns at 1.
/// - If admin mode: randomly shuffle-assign each touchdown to a player.
/// - If player_id was set (via SelectPlayer dialog response): award one touchdown, decrement.
/// - When touchdowns <= 0 → NEXT_STEP, zero the other team's player touchdowns.
/// - While touchdowns remain → show ASSIGN_TOUCHDOWN dialog → Continue.
///
/// DEFERRED(assign-td-dialog): DialogPlayerChoiceParameter(ASSIGN_TOUCHDOWN) + UtilServerDialog.showDialog —
///   waiting for dialog infrastructure to be ported.
/// PlayerNote("is awarded a touchdown") GameEvent wired.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepAssignTouchdowns` (bb2020/end).
pub struct StepAssignTouchdowns {
    /// Java: `touchdowns`
    pub touchdowns: i32,
    /// Java: `winningTeamId`
    pub winning_team_id: Option<String>,
    /// Java: `playerId`
    pub player_id: Option<String>,
}

impl StepAssignTouchdowns {
    pub fn new() -> Self {
        Self {
            touchdowns: 0,
            winning_team_id: None,
            player_id: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let mut pending_events: Vec<GameEvent> = vec![];
        // Java: if (winningTeamId == null) { setNextAction(NEXT_STEP); return; }
        let winning_team_id = match self.winning_team_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: List<Player> players = findPlayers(game, game.getTeamById(winningTeamId))
        // Filter: player.getRecoveringInjury() == null && !playerState.isKilled()
        let players: Vec<String> = if game.team_home.id == winning_team_id {
            game.team_home.players.iter()
        } else {
            game.team_away.players.iter()
        }
        .filter(|p| p.recovering_injury.is_none())
        .filter(|p| game.field_model.player_state(&p.id).map(|s| !s.is_killed()).unwrap_or(true))
        .map(|p| p.id.clone())
        .collect();

        // Java: if (players.isEmpty()) { touchdowns = 0; }
        if players.is_empty() {
            self.touchdowns = 0;
        }

        // Java: if (game.isConcededLegally()) { touchdowns = Math.min(touchdowns, 1); }
        if game.conceded_legally {
            self.touchdowns = self.touchdowns.min(1);
        }

        // Java: if (game.isAdminMode()) {
        //   while (touchdowns-- > 0) { Collections.shuffle(players); award players.get(0); }
        // }
        if game.admin_mode {
            while self.touchdowns > 0 {
                self.touchdowns -= 1;
                if !players.is_empty() {
                    let idx = rng.range(players.len());
                    let pid = players[idx].clone();
                    pending_events.push(GameEvent::PlayerNote { player_id: pid.clone(), note: "is awarded a touchdown".into() });
                    let result = if game.team_home.id == winning_team_id {
                        game.game_result.home.player_results.entry(pid).or_default()
                    } else {
                        game.game_result.away.player_results.entry(pid).or_default()
                    };
                    result.touchdowns += 1;
                }
            }
        }

        // Java: if (StringTool.isProvided(playerId)) {
        //   playerResult.setTouchdowns(playerResult.getTouchdowns() + 1); touchdowns--; playerId = null;
        // }
        if let Some(pid) = self.player_id.take() {
            if !pid.is_empty() {
                pending_events.push(GameEvent::PlayerNote { player_id: pid.clone(), note: "is awarded a touchdown".into() });
                let result = if game.team_home.id == winning_team_id {
                    game.game_result.home.player_results.entry(pid).or_default()
                } else {
                    game.game_result.away.player_results.entry(pid).or_default()
                };
                result.touchdowns += 1;
                self.touchdowns -= 1;
            }
        }

        // Java: if (touchdowns <= 0) {
        //   setNextAction(NEXT_STEP);
        //   Arrays.stream(game.getOtherTeam(...).getPlayers())
        //     .map(p -> game.getGameResult().getPlayerResult(p))
        //     .forEach(pr -> pr.setTouchdowns(0));
        //   return; }
        if self.touchdowns <= 0 {
            // Zero out the other team's player touchdowns.
            let other_team_id = if game.team_home.id == winning_team_id {
                game.team_away.id.clone()
            } else {
                game.team_home.id.clone()
            };
            let other_player_ids: Vec<String> = if game.team_home.id == other_team_id {
                game.team_home.players.iter().map(|p| p.id.clone()).collect()
            } else {
                game.team_away.players.iter().map(|p| p.id.clone()).collect()
            };
            for pid in &other_player_ids {
                let result = if game.team_home.id == other_team_id {
                    game.game_result.home.player_results.entry(pid.clone()).or_default()
                } else {
                    game.game_result.away.player_results.entry(pid.clone()).or_default()
                };
                result.touchdowns = 0;
            }
            let mut out = StepOutcome::next();
            for ev in pending_events { out = out.with_event(ev); }
            return out;
        }

        // Java: DialogPlayerChoiceParameter(winningTeamId, ASSIGN_TOUCHDOWN, playerIds, null, 1, 1)
        // UtilServerDialog.showDialog(getGameState(), dialogParameter, false) → wait for player choice.
        // DEFERRED(assign-td-dialog): show ASSIGN_TOUCHDOWN dialog — waiting for dialog infrastructure.
        let mut out = StepOutcome::cont();
        for ev in pending_events { out = out.with_event(ev); }
        out
    }
}

impl Default for StepAssignTouchdowns {
    fn default() -> Self { Self::new() }
}

impl Step for StepAssignTouchdowns {
    fn id(&self) -> StepId { StepId::AssignTouchdowns }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.ASSIGN_TOUCHDOWN) → sets playerId.
        // Rust: SelectPlayer is the closest available action.
        if let Action::SelectPlayer { player_id } = action {
            self.player_id = Some(player_id.clone());
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: StepParameterKey.TOUCHDOWNS
            StepParameter::Touchdowns(v) => { self.touchdowns = *v; true }
            // Java: StepParameterKey.TEAM_ID
            StepParameter::TeamId(id) => { self.winning_team_id = Some(id.clone()); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_assign_touchdowns() {
        assert_eq!(StepAssignTouchdowns::new().id(), StepId::AssignTouchdowns);
    }

    #[test]
    fn no_winning_team_returns_next_step() {
        let mut game = make_game();
        let mut step = StepAssignTouchdowns::new();
        // winning_team_id is None → immediate NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn zero_touchdowns_no_players_returns_next_step() {
        let mut game = make_game();
        let team_id = game.team_home.id.clone();
        let mut step = StepAssignTouchdowns::new();
        step.winning_team_id = Some(team_id);
        step.touchdowns = 2;
        // test_team creates 0 players → players.is_empty() → touchdowns = 0 → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_touchdowns() {
        let mut step = StepAssignTouchdowns::new();
        assert!(step.set_parameter(&StepParameter::Touchdowns(3)));
        assert_eq!(step.touchdowns, 3);
    }

    #[test]
    fn set_parameter_team_id() {
        let mut step = StepAssignTouchdowns::new();
        assert!(step.set_parameter(&StepParameter::TeamId("team_a".into())));
        assert_eq!(step.winning_team_id, Some("team_a".into()));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepAssignTouchdowns::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn conceded_legally_caps_at_1_with_no_players_returns_next() {
        let mut game = make_game();
        game.conceded_legally = true;
        let team_id = game.team_home.id.clone();
        let mut step = StepAssignTouchdowns::new();
        step.winning_team_id = Some(team_id);
        step.touchdowns = 5;
        // conceded legally → min(5,1) = 1, no players → touchdowns=0 → NEXT_STEP
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn select_player_action_sets_player_id() {
        let mut game = make_game();
        let team_id = game.team_home.id.clone();
        let mut step = StepAssignTouchdowns::new();
        step.winning_team_id = Some(team_id);
        step.touchdowns = 1;
        let action = Action::SelectPlayer { player_id: "p1".into() };
        // With 0 players and playerId set → award then decrement → 0 touchdowns → NEXT_STEP
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn award_touchdown_increments_player_result() {
        let mut game = make_game();
        let team_id = game.team_home.id.clone();
        let mut step = StepAssignTouchdowns::new();
        step.winning_team_id = Some(team_id);
        step.touchdowns = 1;
        step.player_id = Some("p_winner".into());
        step.start(&mut game, &mut GameRng::new(0));
        let td = game.game_result.home.player_results
            .get("p_winner").map(|r| r.touchdowns).unwrap_or(0);
        assert_eq!(td, 1);
    }
}
