use ffb_model::enums::{PlayerAction, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::mechanics::is_skill_roll_successful;

/// BB2020 minimum roll for hypnotic gaze (fixed at 3 per AgilityMechanic).
const MINIMUM_ROLL_HYPNOTIC_GAZE: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepHypnoticGaze.
///
/// BB2020 logic is identical to BB2025.
///
/// Init params: GOTO_LABEL_ON_END (mandatory).
/// Sets: END_PLAYER_ACTION for all steps on the stack.
pub struct StepHypnoticGaze {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepHypnoticGaze {
    pub fn new(goto_label_on_end: String) -> Self {
        Self { goto_label_on_end, re_rolled_action: None, re_roll_source: None }
    }
}

impl Step for StepHypnoticGaze {
    fn id(&self) -> StepId { StepId::HypnoticGaze }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepHypnoticGaze {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let is_gaze_action = game.acting_player.player_action == Some(PlayerAction::Gaze);
        let defender_id = game.defender_id.clone();

        let defender_is_opponent = defender_id.as_deref().map(|id| {
            let home_has = game.team_home.player(id).is_some();
            if game.home_playing { !home_has } else { home_has }
        }).unwrap_or(false);

        let mut do_gaze = is_gaze_action && defender_id.is_some() && defender_is_opponent;

        if !do_gaze {
            game.defender_id = None;
            return StepOutcome::next();
        }

        let acting_player_id = game.acting_player.player_id.clone();

        if self.re_rolled_action.as_deref() == Some("HYPNOTIC_GAZE") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                let pid = acting_player_id.as_deref().unwrap_or("");
                if !use_reroll(game, &source, pid) {
                    do_gaze = false;
                }
            } else {
                do_gaze = false;
            }
        } else {
            let has_gaze_skill = acting_player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::INFLICTS_CONFUSION))
                .unwrap_or(false);
            do_gaze = has_gaze_skill;
        }

        let mut goto_end_label = true;

        if do_gaze {
            let roll = rng.d6();
            let successful = is_skill_roll_successful(roll, MINIMUM_ROLL_HYPNOTIC_GAZE);

            if successful {
                if let Some(ref did) = defender_id {
                    if let Some(old_state) = game.field_model.player_state(did) {
                        if !old_state.is_confused() && !old_state.is_hypnotized() {
                            game.field_model.set_player_state(did, old_state.change_hypnotized(true));
                        }
                    }
                }
            } else if self.re_rolled_action.is_none() {
                let pid = acting_player_id.as_deref().unwrap_or("");
                if let Some(prompt) = ask_for_reroll_if_available(game, "HYPNOTIC_GAZE", MINIMUM_ROLL_HYPNOTIC_GAZE, false) {
                    self.re_rolled_action = Some("HYPNOTIC_GAZE".into());
                    self.re_roll_source = Some("TRR".into());
                    let _ = pid;
                    goto_end_label = false;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
        }

        if goto_end_label {
            game.defender_id = None;
            let label = self.goto_label_on_end.clone();
            StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true))
        } else {
            StepOutcome::cont()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerAction, Rules, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn non_gaze_action_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_action_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = None;
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_no_defender_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = None;
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_with_defender_clears_defender_and_goes_to_end_label() {
        let mut game = make_game();
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));

        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("d1", FieldCoordinate::new(6, 5));

        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());

        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepHypnoticGaze::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }
}
