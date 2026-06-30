use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepInitInducement.
///
/// Initialises an inducement activation sequence. Shows a dialog listing
/// available spells (by InducementType with SPELL usage) and playable cards.
/// On CLIENT_USE_INDUCEMENT:
///   - if type/card null → end inducement phase
///   - if type has SPELL usage → push Wizard sequence
///   - if card → push Card sequence
///   - else → end
///
/// Init: mandatory INDUCEMENT_PHASE, HOME_TEAM.
/// Sets: HOME_TEAM, INDUCEMENT_PHASE, END_INDUCEMENT_PHASE for all steps on the stack.
pub struct StepInitInducement {
    /// Java: fInducementPhase (mandatory)
    pub inducement_phase: Option<InducementPhase>,
    /// Java: fHomeTeam (mandatory, default false)
    pub home_team: bool,
    /// Java: fInducementType (from CLIENT_USE_INDUCEMENT)
    pub inducement_type: Option<String>,
    /// Java: fCard (from CLIENT_USE_INDUCEMENT)
    pub card: Option<String>,
    /// Java: fEndInducementPhase (transient)
    pub end_inducement_phase: bool,
    /// Java: fTouchdownOrEndOfHalf (transient)
    pub touchdown_or_end_of_half: bool,
}

impl StepInitInducement {
    pub fn new() -> Self {
        Self {
            inducement_phase: None,
            home_team: false,
            inducement_type: None,
            card: None,
            end_inducement_phase: false,
            touchdown_or_end_of_half: false,
        }
    }
}

impl Default for StepInitInducement {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitInducement {
    fn id(&self) -> StepId { StepId::InitInducement }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_INDUCEMENT
        //   fInducementType = useInducementCommand.getInducementType()
        //   fCard = useInducementCommand.getCard()
        //   fEndInducementPhase = (type == null && card == null)
        if let Action::PlayCard { card_id, target_player_id: _ } = action {
            // card_id == "" treated as cancel (null in Java)
            if card_id.is_empty() {
                self.inducement_type = None;
                self.card = None;
                self.end_inducement_phase = true;
            } else {
                self.card = Some(card_id.clone());
                self.inducement_type = None;
                self.end_inducement_phase = false;
            }
        }
        // Java: CLIENT_USE_INDUCEMENT with spell type
        // TODO: dedicated Action::UseInducement variant (mapped from WizardSpell for now)
        if let Action::WizardSpell { spell: _, coord: _ } = action {
            // Treat as a spell-type inducement — push Wizard sequence
            self.inducement_type = Some("SPELL".to_string());
            self.end_inducement_phase = false;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InducementPhase(phase) => {
                self.inducement_phase = Some(*phase);
                true
            }
            StepParameter::HomeTeam(v) => {
                self.home_team = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepInitInducement {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let phase = match self.inducement_phase {
            Some(p) => p,
            None => return self.leave_step(true),
        };

        if self.end_inducement_phase {
            // Java: leaveStep(true)
            return self.leave_step(true);
        }

        if self.card.is_none() && self.inducement_type.is_none() {
            // Java: fTouchdownOrEndOfHalf = UtilServerSteps.checkTouchdown(getGameState())
            // Java: find playable cards + useable inducements
            // Java: if any → show dialog; else → leaveStep(true)
            // TODO: UtilServerSteps.checkTouchdown, card/inducement lookup
            // Stub: check if dialog was pending; if not yet prompted, show (Continue)
            // For now advance — hooks will inject actual logic
            return self.leave_step(true);
        }

        if self.inducement_type.as_deref() == Some("SPELL") {
            // Java: push Wizard sequence
            // TODO: SequenceGeneratorFactory.Wizard.pushSequence(...)
            return self.leave_step(false);
        }

        if self.card.is_some() {
            // Java: push Card sequence
            // TODO: SequenceGeneratorFactory.Card.pushSequence(...)
            return self.leave_step(false);
        }

        self.leave_step(true)
    }

    fn leave_step(&self, end_inducement_phase: bool) -> StepOutcome {
        // Java: publishParameter(END_INDUCEMENT_PHASE, ...)
        // Java: publishParameter(HOME_TEAM, ...)
        // Java: publishParameter(INDUCEMENT_PHASE, ...)
        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        let mut out = StepOutcome::next()
            .publish(StepParameter::EndInducementPhase(end_inducement_phase))
            .publish(StepParameter::HomeTeam(self.home_team));
        if let Some(phase) = self.inducement_phase {
            out = out.publish(StepParameter::InducementPhase(phase));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{InducementPhase, Rules};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn step_id_is_init_inducement() {
        let step = StepInitInducement::new();
        assert_eq!(step.id(), StepId::InitInducement);
    }

    #[test]
    fn inducement_phase_parameter_accepted() {
        let mut step = StepInitInducement::new();
        let ok = step.set_parameter(&StepParameter::InducementPhase(InducementPhase::EndOfOwnTurn));
        assert!(ok);
        assert_eq!(step.inducement_phase, Some(InducementPhase::EndOfOwnTurn));
    }

    #[test]
    fn home_team_parameter_accepted() {
        let mut step = StepInitInducement::new();
        let ok = step.set_parameter(&StepParameter::HomeTeam(true));
        assert!(ok);
        assert!(step.home_team);
    }

    #[test]
    fn end_inducement_phase_published_on_start() {
        let mut step = StepInitInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.home_team = true;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_end = out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(_)));
        assert!(has_end);
    }

    #[test]
    fn home_team_published_on_leave() {
        let mut step = StepInitInducement::new();
        step.inducement_phase = Some(InducementPhase::StartOfOwnTurn);
        step.home_team = false;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let published_home = out.published.iter().any(|p| matches!(p, StepParameter::HomeTeam(false)));
        assert!(published_home);
    }

    #[test]
    fn inducement_phase_published_on_leave() {
        let mut step = StepInitInducement::new();
        step.inducement_phase = Some(InducementPhase::BeforeSetup);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let published_phase = out.published.iter().any(|p| {
            matches!(p, StepParameter::InducementPhase(InducementPhase::BeforeSetup))
        });
        assert!(published_phase);
    }

    #[test]
    fn cancel_play_card_sets_end_inducement_phase() {
        let mut step = StepInitInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        let mut game = make_game();
        let action = Action::PlayCard { card_id: "".to_string(), target_player_id: None };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.end_inducement_phase);
    }
}
