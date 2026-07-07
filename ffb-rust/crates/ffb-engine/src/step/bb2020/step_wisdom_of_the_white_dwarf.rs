/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepWisdomOfTheWhiteDwarf` (BB2020).
///
/// Grants a skill to a teammate within 2 squares.
///
/// Differs from BB2025 only in the report class (BB2020 uses ReportSkillUseOtherPlayer from
/// `com.fumbbl.ffb.report.bb2020`).
///
/// Stub: NamedProperties.canGrantSkillsToTeamMates not translated → NEXT_STEP immediately.
///
/// Reports wired:
///   - `ReportSkillUseOtherPlayer` — emitted when teammate is selected and skill is not yet set
///     (Java: `getResult().addReport(new ReportSkillUseOtherPlayer(..., SkillUse.GAIN_GRANTED_SKILL, ...))`).
///   - `ReportPlayerEvent` — emitted when skill is granted to the acting player
///     (Java: `getResult().addReport(new ReportPlayerEvent(actingPlayer.getPlayerId(), "gains " + skillName))`).
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::bb2020::report_skill_use_other_player::ReportSkillUseOtherPlayer;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWisdomOfTheWhiteDwarf {
    /// Java: `playerId` — selected teammate receiving the granted skill.
    pub player_id: Option<String>,
    /// Java: `skill` — skill chosen to grant (set from CLIENT_PRAYER_SELECTION).
    pub granted_skill_name: Option<String>,
}

impl StepWisdomOfTheWhiteDwarf {
    pub fn new() -> Self { Self { player_id: None, granted_skill_name: None } }
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
                // Java: CLIENT_PLAYER_CHOICE → sets playerId, then calls executeStep().
                // executeStep() finds skill == null → emits ReportSkillUseOtherPlayer.
                self.player_id = Some(player_id.clone());
                // Java: getResult().addReport(new ReportSkillUseOtherPlayer(
                //   actingPlayer.getPlayerId(),
                //   game.getPlayerById(playerId).getSkillWithProperty(NamedProperties.canGrantSkillsToTeamMates),
                //   SkillUse.GAIN_GRANTED_SKILL,
                //   playerId))
                let acting_id = game.acting_player.player_id.clone().unwrap_or_default();
                game.report_list.add(ReportSkillUseOtherPlayer::new(
                    acting_id,
                    SkillUse::GAIN_GRANTED_SKILL.get_name().to_string(),
                    SkillUse::GAIN_GRANTED_SKILL.get_name().to_string(),
                    player_id.clone(),
                ));
            }
            Action::SelectSkill { skill_id } => {
                // Java: CLIENT_PRAYER_SELECTION → sets skill, then calls executeStep().
                // executeStep() grants skill and emits ReportPlayerEvent.
                let skill_name = format!("{:?}", skill_id);
                self.granted_skill_name = Some(skill_name.clone());
                // Java: getResult().addReport(new ReportPlayerEvent(
                //   actingPlayer.getPlayerId(), "gains " + gainedSkill.getSkill().getName()))
                let acting_id = game.acting_player.player_id.clone();
                game.report_list.add(ReportPlayerEvent::new(
                    acting_id,
                    Some(format!("gains {}", skill_name)),
                ));
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
    use ffb_model::enums::{Rules, SkillId};

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

    #[test]
    fn select_player_emits_skill_use_other_player_report() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let action = Action::SelectPlayer { player_id: "p2".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE_OTHER_PLAYER));
    }

    #[test]
    fn select_skill_emits_player_event_report() {
        let mut game = make_game();
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let action = Action::SelectSkill { skill_id: SkillId::Block };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT));
    }
}
