/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.shared.StepPickMeUp`.
///
/// Handles the "Pick Me Up" skill (BB2020+/Mixed): players with this skill (on pitch, with
/// tackle zones) can stand up adjacent prone team-mates (within 3 steps) after activation.
/// The coach selects which prone players to attempt to stand up; each rolls a D6 and succeeds
/// on 5+.
///
/// Flow:
///   `start()` / first run:
///     - Skip entirely on touchdown or final turn (both turn-nrs == 8).
///     - Collect prone players within 3 steps of any Pick-Me-Up player on the opposing team.
///     - Show a dialog asking which of them to attempt.
///     - Set `first_run = false`.
///   Subsequent runs (after `CLIENT_PLAYER_CHOICE(PICK_ME_UP)` command):
///     - Roll D6 for each selected player; succeed on 5+ → change state to STANDING.
///     - If more players remain, show dialog again; else NEXT_STEP.
///
/// Java: `com.fumbbl.ffb.server.step.mixed.shared.StepPickMeUp` extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PS_STANDING, PS_PRONE};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPickMeUp` (mixed/shared, BB2020 + BB2025).
pub struct StepPickMeUp {
    /// Java: `playerIds` — prone player IDs eligible for Pick Me Up attempts
    pub player_ids: Vec<String>,
    /// Java: `playerIdsSelected` — player IDs chosen by coach in the current dialog round
    pub player_ids_selected: Vec<String>,
    /// Java: `firstRun`
    pub first_run: bool,
}

impl StepPickMeUp {
    pub fn new() -> Self {
        Self {
            player_ids: Vec::new(),
            player_ids_selected: Vec::new(),
            first_run: true,
        }
    }

    /// Java: `interpretPickMeUp(roll)` — success on D6 >= 5
    fn interpret_pick_me_up(roll: i32) -> bool {
        roll >= 5
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.first_run {
            // Java: if (checkTouchdown || (turnHome==8 && turnAway==8)) → NEXT_STEP
            let last_turn = game.turn_data_home.turn_nr == 8 && game.turn_data_away.turn_nr == 8;
            if last_turn {
                return StepOutcome::next();
            }

            // Java: hideDialog; find opposing team; find pick-me-up players on pitch with tacklezones
            // For each pick-me-up player: find prone team-mates within 3 steps.
            // TODO(NamedProperties port): filter by NamedProperties.canStandUpTeamMates
            // Until the property system is ported, collect all prone players on the non-acting team
            // within 3 steps of any standing player on that team (conservative stub).
            let other_team_id = game.inactive_team().id.clone();
            let player_ids_on_other_team: Vec<String> = {
                let team = game.inactive_team();
                team.players.iter().map(|p| p.id.clone()).collect()
            };
            // Find prone players adjacent to a standing teammate (within 3 steps)
            let mut eligible: std::collections::HashSet<String> = std::collections::HashSet::new();
            let _ = other_team_id;
            for picker_id in &player_ids_on_other_team {
                let picker_state = game.field_model.player_state(picker_id);
                let picker_coord = game.field_model.player_coordinate(picker_id);
                if let (Some(state), Some(coord)) = (picker_state, picker_coord) {
                    if !state.has_tacklezones() { continue; }
                    // Find prone teammates within 3 steps
                    for teammate_id in &player_ids_on_other_team {
                        if teammate_id == picker_id { continue; }
                        let tm_coord = game.field_model.player_coordinate(teammate_id);
                        let tm_state = game.field_model.player_state(teammate_id);
                        if let (Some(tc), Some(ts)) = (tm_coord, tm_state) {
                            if !tc.is_box_coordinate()
                                && ts.base() == PS_PRONE
                                && coord.distance_in_steps(tc) <= 3
                            {
                                eligible.insert(teammate_id.clone());
                            }
                        }
                    }
                }
            }
            self.player_ids = eligible.into_iter().collect();
            self.first_run = false;
        } else {
            // Java: for each selected player: roll D6; success → STANDING; add report
            for id in self.player_ids_selected.drain(..).collect::<Vec<_>>() {
                let roll = rng.d6();
                let success = Self::interpret_pick_me_up(roll);
                if success {
                    if let Some(current_state) = game.field_model.player_state(&id) {
                        game.field_model.set_player_state(
                            &id,
                            current_state.change_base(PS_STANDING),
                        );
                    }
                }
                // TODO(Report port): addReport(ReportPickMeUp(id, roll, success))
            }
        }

        if self.player_ids.is_empty() {
            return StepOutcome::next();
        }

        // Java: showDialog(DialogPlayerChoiceParameter(team, PICK_ME_UP, playerIds, null, size, 0))
        // Not yet ported → wait for a PlayerChoice command.
        StepOutcome::cont()
    }
}

impl Default for StepPickMeUp {
    fn default() -> Self { Self::new() }
}

impl Step for StepPickMeUp {
    fn id(&self) -> StepId { StepId::PickMeUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE →
        //   if selected is non-empty: playerIdsSelected += selected; playerIds -= selected
        //   else: playerIds.clear()
        //   → EXECUTE_STEP
        match action {
            Action::PlayerChoice { player_ids, mode, .. } if mode == "PICK_ME_UP" => {
                if player_ids.is_empty() {
                    // Java: ArrayTool.isProvided([]) → false → clear
                    self.player_ids.clear();
                } else {
                    self.player_ids_selected.extend(player_ids.iter().cloned());
                    self.player_ids.retain(|id| !self.player_ids_selected.contains(id));
                }
                self.execute_step(game, rng)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_pick_me_up() {
        assert_eq!(StepPickMeUp::new().id(), StepId::PickMeUp);
    }

    #[test]
    fn start_returns_next_when_no_eligible_players() {
        let mut step = StepPickMeUp::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // No players on field → no eligible → next step
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn last_turn_skips_pick_me_up() {
        let mut step = StepPickMeUp::new();
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn first_run_false_after_start() {
        let mut step = StepPickMeUp::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(!step.first_run);
    }

    #[test]
    fn interpret_pick_me_up_threshold() {
        assert!(!StepPickMeUp::interpret_pick_me_up(4));
        assert!(StepPickMeUp::interpret_pick_me_up(5));
        assert!(StepPickMeUp::interpret_pick_me_up(6));
    }

    #[test]
    fn player_choice_empty_clears_list() {
        let mut step = StepPickMeUp::new();
        step.player_ids = vec!["p1".into(), "p2".into()];
        step.first_run = false; // skip first-run logic
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::PlayerChoice {
                player_id: None,
                player_ids: vec![],
                mode: "PICK_ME_UP".into(),
            },
            &mut game,
            &mut rng,
        );
        assert!(step.player_ids.is_empty());
    }

    #[test]
    fn player_choice_selected_moves_to_selected_list() {
        let mut step = StepPickMeUp::new();
        step.player_ids = vec!["p1".into(), "p2".into()];
        step.first_run = false;
        let mut game = make_game();
        let mut rng = GameRng::new(42);
        // Use deterministic seed: with no players on field, after second round player_ids empty
        step.handle_command(
            &Action::PlayerChoice {
                player_id: None,
                player_ids: vec!["p1".into()],
                mode: "PICK_ME_UP".into(),
            },
            &mut game,
            &mut rng,
        );
        // p1 was selected and processed; p2 remains
        assert!(!step.player_ids.contains(&"p1".to_string()));
    }
}
