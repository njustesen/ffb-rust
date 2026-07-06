/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepWisdomOfTheWhiteDwarf` (BB2020).
///
/// Grants a skill to a teammate within 2 squares.
///
/// Differs from BB2025 only in the report class (BB2020 uses ReportWisdomOfTheWhiteDwarfBb2020).
/// Reports are not translated, so behavior is identical.
///
/// Stub: NamedProperties.canGrantSkillsToTeamMates not translated → NEXT_STEP immediately.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWisdomOfTheWhiteDwarf {
    pub player_id: Option<String>,
}

impl StepWisdomOfTheWhiteDwarf {
    pub fn new() -> Self { Self { player_id: None } }
}

impl Default for StepWisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl Step for StepWisdomOfTheWhiteDwarf {
    fn id(&self) -> StepId { StepId::WisdomOfTheWhiteDwarf }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn handle_command(&mut self, action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.player_id = Some(player_id.clone());
            }
            _ => {}
        }
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_select_player_stores_id() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let action = Action::SelectPlayer { player_id: "teammate".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.player_id.as_deref(), Some("teammate"));
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn id_is_wisdom_of_the_white_dwarf() {
        assert_eq!(StepWisdomOfTheWhiteDwarf::new().id(), StepId::WisdomOfTheWhiteDwarf);
    }

    #[test]
    fn non_select_player_action_does_not_store_id() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.player_id.is_none());
    }
}
