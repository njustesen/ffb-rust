/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepPro`.
///
/// Handles the Pro skill re-roll (BB2020 + BB2025).
/// Given a `PLAYER_ID` init parameter, the step:
///   1. If a re-roll source is set, attempts to consume it (unsets `usedPro` on the player).
///   2. Rolls the Pro die (D6 >= 4 = success).
///   3. If unsuccessful and OLD_PRO wasn't already re-rolled, asks for a team/skill re-roll.
///   4. Publishes `SUCCESSFUL_PRO(bool)` and advances.
///
/// Java: `StepPro extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

const MINIMUM_PRO_ROLL: i32 = 4; // Java: RollMechanic.minimumProRoll() = 4 across all editions
const OLD_PRO: &str = "OLD_PRO";

/// Java: `StepPro` (mixed, BB2020 + BB2025).
pub struct StepPro {
    /// Java: `playerId` (init param PLAYER_ID)
    pub player_id: Option<String>,
    /// AbstractStepWithReRoll state
    pub re_roll: ReRollState,
}

impl StepPro {
    pub fn new() -> Self {
        Self { player_id: None, re_roll: ReRollState::new() }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.player_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        if game.player(&player_id).is_none() {
            return StepOutcome::next();
        }

        let mut do_roll = true;
        let mut successful = false;

        // Java: if (getReRollSource() != null) { if (useReRoll(source, player)) { changeUsedPro(false) } else doRoll=false }
        if let Some(ref source) = self.re_roll.re_roll_source.clone() {
            if use_reroll(game, source, &player_id) {
                // Reset usedPro flag so the Pro roll can proceed
                if let Some(state) = game.field_model.player_state(&player_id) {
                    game.field_model.set_player_state(&player_id, state.change_used_pro(false));
                }
            } else {
                do_roll = false;
            }
        }

        // Java: if (doRoll) { successful = useReRoll(this, ReRollSources.PRO, player) }
        // ReRollSources.PRO internally rolls D6 >= minimumProRoll()
        if do_roll {
            let roll = rng.d6();
            successful = roll >= MINIMUM_PRO_ROLL;
            // Mark Pro as used (Java: mechanic.useReRoll rolls it and marks skill used)
            if let Some(state) = game.field_model.player_state(&player_id) {
                game.field_model.set_player_state(&player_id, state.change_used_pro(!successful));
            }
        }

        // Java: if (!successful && getReRolledAction() != OLD_PRO) → ask for reroll
        let already_rerolled = self.re_roll.re_rolled_action.as_ref()
            .map(|a| a.name == OLD_PRO)
            .unwrap_or(false);

        if !successful && !already_rerolled {
            if let Some(prompt) = ask_for_reroll_if_available(game, OLD_PRO, MINIMUM_PRO_ROLL, false) {
                self.re_roll.re_rolled_action = Some(ffb_model::model::re_rolled_action::ReRolledAction::new(OLD_PRO));
                self.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
                return StepOutcome::cont().with_prompt(prompt);
            }
        }

        StepOutcome::next().publish(StepParameter::SuccessfulPro(successful))
    }
}

impl Default for StepPro {
    fn default() -> Self { Self::new() }
}

impl Step for StepPro {
    fn id(&self) -> StepId { StepId::Pro }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) {
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
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_pro() {
        assert_eq!(StepPro::new().id(), StepId::Pro);
    }

    #[test]
    fn no_player_id_returns_next_step() {
        let mut step = StepPro::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn roll_4_or_more_publishes_successful_pro_true() {
        let mut step = StepPro::new();
        step.player_id = Some("p1".into());
        let mut game = make_game();
        add_player(&mut game, "p1");
        // Use a seed that gives d6 = 4 or more
        // Seed 5 gives d6 >= 4 in GameRng
        let mut rng = GameRng::new(5);
        let roll = rng.d6();
        let succeeds = roll >= MINIMUM_PRO_ROLL;
        let mut rng2 = GameRng::new(5);
        let out = step.start(&mut game, &mut rng2);
        if succeeds {
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::SuccessfulPro(true))));
        } else {
            // Still publishes SuccessfulPro(false) when no reroll available
            assert!(out.published.iter().any(|p| matches!(p, StepParameter::SuccessfulPro(false))));
        }
    }

    #[test]
    fn set_parameter_player_id() {
        let mut step = StepPro::new();
        let accepted = step.set_parameter(&StepParameter::PlayerId("pid".into()));
        assert!(accepted);
        assert_eq!(step.player_id, Some("pid".into()));
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepPro::new();
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }

    #[test]
    fn decline_reroll_clears_source() {
        let mut step = StepPro::new();
        step.player_id = Some("p1".into());
        step.re_roll.re_roll_source = Some(ReRollSource::new("TRR"));
        let mut game = make_game();
        add_player(&mut game, "p1");
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll.re_roll_source.is_none());
    }
}
