use ffb_model::enums::InducementPhase;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initializes the inducement sequence: checks which inducements can be used at the given
/// phase, shows the use-inducement dialog, then routes the chosen type to the Wizard sequence,
/// the Weather Mage step or the Throw a Rock sequence (§Q3.2).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.inducements.StepInitInducement`.
pub struct StepInitInducement {
    /// Java: fInducementPhase (init param)
    pub inducement_phase: Option<InducementPhase>,
    /// Java: fHomeTeam (init param)
    pub home_team: bool,
    /// Java: fInducementType (InducementType) — stored as name until InducementType is ported
    pub inducement_type: Option<String>,
    /// Java: fEndInducementPhase (transient)
    pub end_inducement_phase: bool,
    /// Java: fTouchdownOrEndOfHalf (transient)
    pub touchdown_or_end_of_half: bool,
}

impl StepInitInducement {
    pub fn new(inducement_phase: InducementPhase, home_team: bool) -> Self {
        Self {
            inducement_phase: Some(inducement_phase),
            home_team,
            inducement_type: None,
            end_inducement_phase: false,
            touchdown_or_end_of_half: false,
        }
    }
}

impl Default for StepInitInducement {
    fn default() -> Self {
        Self {
            inducement_phase: None,
            home_team: false,
            inducement_type: None,
            end_inducement_phase: false,
            touchdown_or_end_of_half: false,
        }
    }
}

impl Step for StepInitInducement {
    fn id(&self) -> StepId { StepId::InitInducement }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_INDUCEMENT — `fInducementType = command.getInducementType();
        // fEndInducementPhase = fInducementType == null`. The agents' decline (Acknowledge) is
        // Java's null type.
        if let Action::UseInducement { inducement_type: Some(itype), .. } = action {
            self.inducement_type = Some(itype.clone());
            self.end_inducement_phase = false;
        } else {
            self.inducement_type = None;
            self.end_inducement_phase = true;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InducementPhase(v) => { self.inducement_phase = Some(*v); true }
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
            _ => false,
        }
    }
}

impl StepInitInducement {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let phase = match self.inducement_phase {
            Some(p) => p,
            None => return StepOutcome::next(),
        };
        if self.end_inducement_phase {
            return self.leave_step(phase, true);
        }
        let Some(inducement_type) = self.inducement_type.clone() else {
            // Java: fTouchdownOrEndOfHalf = UtilServerSteps.checkTouchdown(getGameState())
            self.touchdown_or_end_of_half = crate::step::util_server_steps::check_touchdown(game);
            let useable = self.find_useable_inducements(game, phase);
            if useable.is_empty() {
                return self.leave_step(phase, true);
            }
            // Java: game.setDialogParameter(new DialogUseInducementParameter(teamId, useable)).
            let team_id = if self.home_team { game.team_home.id.clone() } else { game.team_away.id.clone() };
            crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                game,
                ffb_model::dialog::dialog_parameter_factory::DialogParameter::UseInducement(
                    ffb_model::dialog::dialog_use_inducement_parameter::DialogUseInducementParameter {
                        team_id: Some(team_id.clone()),
                        inducement_types: useable.iter().cloned().map(Some).collect(),
                        cards: Vec::new(),
                        player_id: None,
                    },
                ),
                false,
            );
            return StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::UseInducement {
                team_id,
                inducement_types: useable,
            });
        };
        let usages = ffb_model::factory::inducement_type_factory::for_name_in(game.rules, &inducement_type)
            .map(|t| t.usages)
            .unwrap_or_default();
        if usages.contains(&Usage::SPELL) {
            // Java: Wizard.pushSequence(...); leaveStep(false)
            let seq = crate::step::generator::common::wizard::Wizard::build_sequence();
            self.leave_step(phase, false).push_seq(seq)
        } else if usages.contains(&Usage::CHANGE_WEATHER) {
            // Java: getStepStack().push(new StepWeatherMage(gameState)); leaveStep(false)
            let seq = vec![crate::step::framework::SequenceStep::new(StepId::WeatherMage)];
            self.leave_step(phase, false).push_seq(seq)
        } else if usages.contains(&Usage::THROW_ROCK) && game.rules == ffb_model::enums::Rules::Bb2025 {
            // Java: ThrowARock.pushSequence(new ThrowARock.SequenceParams(gameState, fHomeTeam)); leaveStep(false)
            let seq = crate::step::generator::bb2025::throw_a_rock::ThrowARock::build_sequence(
                &crate::step::generator::bb2025::throw_a_rock::ThrowARockParams { home_team: self.home_team });
            self.leave_step(phase, false).push_seq(seq)
        } else {
            self.leave_step(phase, true)
        }
    }

    /// Java `leaveStep(boolean pEndInducementPhase)`.
    fn leave_step(&self, phase: InducementPhase, end_inducement_phase: bool) -> StepOutcome {
        StepOutcome::next()
            .publish(StepParameter::EndInducementPhase(end_inducement_phase))
            .publish(StepParameter::HomeTeam(self.home_team))
            .publish(StepParameter::InducementPhase(phase))
    }

    /// Java `findUseableInducements()`, sorted by name (Java collects into a HashSet). Its side
    /// effects are load-bearing: the END_OF_OWN/OPPONENT_TURN windows set BETWEEN_TURNS, and the
    /// END_OF_OPPONENT_TURN window hands home_playing to the window's own team (dropping them
    /// collapsed the three-flip turn-handover chain into a double flip).
    fn find_useable_inducements(&self, game: &mut Game, phase: InducementPhase) -> Vec<String> {
        let mut useable: Vec<String> = Vec::new();
        let turn_data = if self.home_team { &game.turn_data_home } else { &game.turn_data_away };
        let set = &turn_data.inducement_set;
        let of_usage = |usage_ok: &dyn Fn(&ffb_model::inducement::inducement::Inducement) -> bool| -> Vec<String> {
            set.get_inducements().into_iter()
                .filter(|i| usage_ok(i) && set.has_uses_left(&i.type_id))
                .map(|i| i.type_id)
                .collect()
        };
        if matches!(phase, InducementPhase::EndOfOwnTurn | InducementPhase::EndOfOpponentTurn)
            && !self.touchdown_or_end_of_half
        {
            // bb2025 offers SPELL and THROW_ROCK; bb2020's StepInitInducement SPELL only.
            let bb2025 = game.rules == ffb_model::enums::Rules::Bb2025;
            useable = of_usage(&|i| i.has_usage(Usage::SPELL) || (bb2025 && i.has_usage(Usage::THROW_ROCK)));
            // bb2016's StepInitInducement has no turn-mode change (lastTurnMode stays KICKOFF for
            // the client); the home_playing handover below is Rust's shared turn chain and stays.
            if game.rules != ffb_model::enums::Rules::Bb2016 {
                game.set_turn_mode(ffb_model::enums::TurnMode::BetweenTurns);
            }
            if phase == InducementPhase::EndOfOpponentTurn && self.home_team != game.home_playing {
                game.home_playing = !game.home_playing;
            }
        } else if phase == InducementPhase::StartOfOwnTurn {
            useable = of_usage(&|i| i.has_usage(Usage::CHANGE_WEATHER));
        }
        // This step serves all three editions; the use routing is ported for bb2025 and bb2020
        // (§Q3.2; bb2020 cards are §Q3.3 -- none can be bought yet, so none are playable). bb2016
        // (its own StepInitInducement) keeps the old no-dialog behaviour until its purchases exist.
        if game.rules == ffb_model::enums::Rules::Bb2016 {
            useable.clear();
        }
        useable.sort();
        useable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, InducementPhase};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_and_publishes_phase() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::EndOfOwnTurn, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementPhase(_))));
    }

    #[test]
    fn publishes_home_team_flag() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::StartOfOwnTurn, false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::HomeTeam(false))));
    }

    /// §Q3.2 Java findUseableInducements: START_OF_OWN_TURN offers CHANGE_WEATHER types with uses
    /// left; choosing one pushes StepWeatherMage and leaves with END_INDUCEMENT_PHASE = false.
    #[test]
    fn a_weather_mage_is_offered_at_the_start_of_the_turn_and_routes_to_the_mage_step() {
        use ffb_model::inducement::inducement::Inducement;
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(
            Inducement::new("weatherMage", 1, vec![Usage::CHANGE_WEATHER]));
        let mut step = StepInitInducement::new(InducementPhase::StartOfOwnTurn, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        match out.prompt {
            Some(ffb_model::prompts::AgentPrompt::UseInducement { inducement_types, .. }) =>
                assert_eq!(inducement_types, vec!["weatherMage".to_string()]),
            other => panic!("expected the use-inducement prompt, got {other:?}"),
        }
        let out = step.handle_command(&Action::UseInducement {
            inducement_type: Some("weatherMage".into()), card_id: None, player_ids: vec![] },
            &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.iter().flatten().any(|s| s.step_id == StepId::WeatherMage));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(false))));
    }

    /// Java: a spell is offered only at END_OF_OWN/OPPONENT_TURN, not at the start of the turn; a
    /// used-up inducement is not offered at all.
    #[test]
    fn a_used_up_wizard_is_not_offered() {
        use ffb_model::inducement::inducement::Inducement;
        let mut game = make_game();
        let mut wizard = Inducement::new("wizard", 1, vec![Usage::SPELL]);
        wizard.set_uses(1);
        game.turn_data_home.inducement_set.add_inducement(wizard);
        let mut step = StepInitInducement::new(InducementPhase::EndOfOwnTurn, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(true))));
    }

    #[test]
    fn handle_command_sets_end_inducement_phase() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::EndOfOwnTurn, true);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert!(step.end_inducement_phase);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_end_inducement_phase_true() {
        let mut game = make_game();
        let mut step = StepInitInducement::new(InducementPhase::BeforeSetup, true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(true))));
    }

    #[test]
    fn set_parameter_inducement_phase_accepted() {
        let mut step = StepInitInducement::default();
        assert!(step.set_parameter(&StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn)));
        assert_eq!(step.inducement_phase, Some(InducementPhase::StartOfOwnTurn));
    }

    #[test]
    fn set_parameter_home_team_accepted() {
        let mut step = StepInitInducement::default();
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }

    #[test]
    fn default_with_set_parameter_publishes_correctly() {
        let mut game = make_game();
        let mut step = StepInitInducement::default();
        step.set_parameter(&StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn));
        step.set_parameter(&StepParameter::HomeTeam(false));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndInducementPhase(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementPhase(InducementPhase::StartOfOwnTurn))));
    }
}
