/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepSelectBlitzTarget (BB2025).
///
/// Prompts the coach to select a blitz target during FuriousOutburst / AnimalSavagery.
/// Dispatches to skill-specific sequence generators (BlackInk, BalefulHex, etc.) based on
/// the selected skill.
///
/// Init params: GOTO_LABEL_ON_END.
/// Runtime params: END_TURN, END_PLAYER_ACTION, CHECK_FORGO.
///
/// Stub: all generator dispatches (BlackInk, BalefulHex, CatchOfTheDay, LookIntoMyEyes,
/// Treacherous, ThenIStartedBlastin, RaidingParty) not translated → NEXT_STEP immediately.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_select_blitz_target::ReportSelectBlitzTarget;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::EndPlayerAction;
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;

pub struct StepSelectBlitzTarget {
    /// Java: gotoLabelOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_label_on_end: String,
    /// Java: selectedPlayerId — set from CLIENT_TARGET_SELECTED command.
    pub selected_player_id: Option<String>,
    /// Java: confirmed — set when selection is confirmed.
    pub confirmed: bool,
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: checkForgo — set by CHECK_FORGO parameter.
    pub check_forgo: bool,
    /// Java: usedSkill — set when CLIENT_USE_SKILL received.
    pub used_skill: Option<SkillId>,
}

impl StepSelectBlitzTarget {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            selected_player_id: None,
            confirmed: false,
            end_player_action: false,
            end_turn: false,
            check_forgo: false,
            used_skill: None,
        }
    }
}

impl Default for StepSelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

impl Step for StepSelectBlitzTarget {
    fn id(&self) -> StepId { StepId::SelectBlitzTarget }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step()
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.selected_player_id = Some(player_id.clone());
                // Java: addReport(new ReportSelectBlitzTarget(actingPlayer.getPlayerId(), selectedPlayerId))
                // when the selected player is not on the acting team.
                let is_opponent = game.inactive_team().player(player_id).is_some();
                if is_opponent {
                    let acting_id = game.acting_player.player_id.clone();
                    let attacker_id = acting_id.clone().unwrap_or_default();
                    game.report_list.add(ReportSelectBlitzTarget::new(
                        acting_id,
                        Some(player_id.clone()),
                    ));
                    // Java: if usedSkill has canGainFrenzyForBlitz → ReportSkillUse(GAIN_FRENZY_FOR_BLITZ)
                    //        else if canGainClawsForBlitz → ReportSkillUse(GAIN_CLAWS_FOR_BLITZ)
                    //        else if canAvoidDodging → ReportSkillUse(AVOID_DODGING)
                    if let Some(skill) = self.used_skill {
                        if skill.properties().contains(&NamedProperties::CAN_GAIN_FRENZY_FOR_BLITZ) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::GAIN_FRENZY_FOR_BLITZ,
                            ));
                        } else if skill.properties().contains(&NamedProperties::CAN_GAIN_CLAWS_FOR_BLITZ) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::GAIN_CLAWS_FOR_BLITZ,
                            ));
                        } else if skill.properties().contains(&NamedProperties::CAN_AVOID_DODGING) {
                            game.report_list.add(ReportSkillUse::new(
                                Some(attacker_id),
                                skill,
                                true,
                                SkillUse::AVOID_DODGING,
                            ));
                        }
                    }
                }
            }
            Action::EndTurn => {
                self.end_turn = true;
                self.check_forgo = true;
            }
            // Java: CLIENT_USE_SKILL → store usedSkill for ReportSkillUse emission
            Action::UseSkill { skill_id, use_skill: true } => {
                self.used_skill = Some(*skill_id);
            }
            _ => {}
        }
        self.execute_step()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)           => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)   => { self.end_player_action = *v; true }
            StepParameter::CheckForgo(v)        => { self.check_forgo = *v; true }
            StepParameter::GotoLabelOnEnd(v)    => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepSelectBlitzTarget {
    fn execute_step(&self) -> StepOutcome {
        if self.end_player_action || self.end_turn {
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action: true,
                end_turn: self.end_turn,
                check_forgo: self.check_forgo,
            });
            return StepOutcome::next().push_seq(seq);
        }
        // Stub: skill-based sequence generators (BlackInk, BalefulHex, etc.) not translated
        StepOutcome::next()
    }
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
        let mut step = StepSelectBlitzTarget::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_select_player_stores_id() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        let action = Action::SelectPlayer { player_id: "p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.selected_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "end_turn should push EndPlayerAction sequence");
    }

    #[test]
    fn end_player_action_pushes_sequence() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "end_player_action should push EndPlayerAction sequence");
    }

    #[test]
    fn handle_end_turn_sets_flags() {
        let mut game = make_game();
        let mut step = StepSelectBlitzTarget::new();
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(step.end_turn);
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepSelectBlitzTarget::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    #[test]
    fn select_opponent_adds_select_blitz_target_report() {
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        away.players.push(Player { id: "away_p1".into(), name: "A".into(), ..Default::default() });
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        let action = Action::SelectPlayer { player_id: "away_p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SELECT_BLITZ_TARGET));
    }

    #[test]
    fn select_opponent_with_frenzied_rush_adds_skill_use_report() {
        use ffb_model::model::player::Player;
        use ffb_model::report::report_id::ReportId;
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        away.players.push(Player { id: "away_p1".into(), name: "A".into(), ..Default::default() });
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        let mut step = StepSelectBlitzTarget::new();
        step.used_skill = Some(SkillId::FrenziedRush);
        let action = Action::SelectPlayer { player_id: "away_p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SELECT_BLITZ_TARGET));
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }
}
