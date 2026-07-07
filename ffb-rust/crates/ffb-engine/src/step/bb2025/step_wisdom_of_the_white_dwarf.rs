/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepWisdomOfTheWhiteDwarf (BB2025).
///
/// Grants a skill to a teammate within 2 squares (Wisdom of the White Dwarf ability).
///
/// Commands: CLIENT_PLAYER_CHOICE (teammate selection), CLIENT_PRAYER_SELECTION (skill to grant).
///
/// Stub: NamedProperties.canGrantSkillsToTeamMates and Constant.getGrantAbleSkills are not yet
/// translated. The step returns NEXT_STEP immediately (matches Java "no eligible players" path).
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWisdomOfTheWhiteDwarf {
    /// Java: playerId — selected teammate receiving the skill.
    pub player_id: Option<String>,
    /// Java: skill — selected skill to grant (set from CLIENT_PRAYER_SELECTION).
    pub granted_skill: Option<SkillId>,
    /// Java: grantingSkillId — the WisdomOfTheWhiteDwarf skill on the acting player.
    pub granting_skill: Option<SkillId>,
}

impl StepWisdomOfTheWhiteDwarf {
    pub fn new() -> Self {
        Self { player_id: None, granted_skill: None, granting_skill: None }
    }
}

impl Default for StepWisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl Step for StepWisdomOfTheWhiteDwarf {
    fn id(&self) -> StepId { StepId::WisdomOfTheWhiteDwarf }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Stub: NamedProperties.canGrantSkillsToTeamMates not translated → NEXT_STEP
        StepOutcome::next()
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.player_id = Some(player_id.clone());
                // Java: addReport(new ReportSkillUse(actingPlayerId, grantingSkill, true, GRANT_SKILL_TO_TEAM_MATE))
                if let Some(granting) = self.granting_skill {
                    let acting_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        acting_id,
                        granting,
                        true,
                        SkillUse::GRANT_SKILL_TO_TEAM_MATE,
                    ));
                }
                // Java: addReport(new ReportPlayerEvent(target.getId(), "gains " + gainedSkill.getName()))
                if let Some(skill) = self.granted_skill {
                    let skill_name = skill.class_name();
                    game.report_list.add(ReportPlayerEvent::new(
                        Some(player_id.clone()),
                        Some(format!("gains {skill_name}")),
                    ));
                }
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
        Game::new(home, away, Rules::Bb2025)
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
    fn handle_unrecognized_action_stores_no_player() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        use crate::action::Action;
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.player_id.is_none());
    }

    #[test]
    fn select_player_with_granting_skill_adds_skill_use_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        step.granting_skill = Some(SkillId::WisdomOfTheWhiteDwarf);
        let action = crate::action::Action::SelectPlayer { player_id: "teammate".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }

    #[test]
    fn select_player_with_granted_skill_adds_player_event_report() {
        use ffb_model::enums::SkillId;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        step.granted_skill = Some(SkillId::Block);
        let action = crate::action::Action::SelectPlayer { player_id: "teammate".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT));
    }
}
