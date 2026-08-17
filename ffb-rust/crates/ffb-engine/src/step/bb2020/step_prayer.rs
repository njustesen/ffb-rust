/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepPrayer` (BB2020).
///
/// Applies a single "Prayer to Nuffle" effect during play.
///
/// Differs from BB2025 only in the report class used (BB2020 uses ReportPrayerBb2020,
/// BB2025 uses ReportPrayer). Reports are not translated, so behavior is identical.
///
/// Dispatches through `inducements::bb2020::prayers::prayer_handler_factory` (Java's
/// `PrayerHandlerFactory.forPrayer`). Previously a stub that always took Java's "no handler found"
/// path, so a prayer awarded by the BB2020 Cheering Fans kickoff was reported and then silently
/// dropped — its effect dice were never rolled.
use ffb_model::model::game::Game;
use ffb_model::events::GameEvent;
use ffb_model::util::rng::GameRng;
use ffb_model::report::bb2020::report_prayer_roll::ReportPrayerRoll;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepPrayer {
    pub roll: i32,
    pub team_id: Option<String>,
    pub player_id: Option<String>,
    /// The skill chosen from a `DialogSelectSkillParameter` (Intensive Training).
    pub selected_skill: Option<ffb_model::enums::SkillId>,
    pub first_run: bool,
    /// `GameEvent::PrayerRoll` parked on the first run and attached at the step's exit.
    pending_prayer_event: Option<GameEvent>,
}

impl StepPrayer {
    pub fn new(roll: i32, team_id: impl Into<String>) -> Self {
        Self { roll, team_id: Some(team_id.into()), player_id: None, selected_skill: None, first_run: true, pending_prayer_event: None }
    }
}

impl Default for StepPrayer {
    fn default() -> Self {
        Self { roll: 0, team_id: None, player_id: None, selected_skill: None, first_run: true, pending_prayer_event: None }
    }
}

impl Step for StepPrayer {
    fn id(&self) -> StepId { StepId::Prayer }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let out = self.execute_step(game, rng);
        match self.pending_prayer_event.take() {
            Some(ev) => out.with_event(ev),
            None => out,
        }
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                self.player_id = Some(player_id.clone());
            }
            // Java: CLIENT_PLAYER_CHOICE → `playerId = clientCommandPlayerChoice.getPlayerId()`.
            Action::PlayerChoice { player_id, .. } => {
                self.player_id = player_id.clone();
            }
            // Java: CLIENT_PRAYER_SELECTION carries the chosen skill; the player id travels with
            // the dialog, so the step remembers it from when it showed the prompt.
            Action::SelectSkill { skill_id } => {
                self.selected_skill = Some(*skill_id);
            }
            _ => {}
        }
        let out = self.execute_step(game, rng);
        match self.pending_prayer_event.take() {
            Some(ev) => out.with_event(ev),
            None => out,
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PrayerRoll(v) => { self.roll = *v; true }
            StepParameter::TeamId(v) => { self.team_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

impl StepPrayer {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        use ffb_model::factory::bb2020::prayer_factory::PrayerFactory;
        use ffb_model::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;
        use ffb_model::option::game_option_id;
        use crate::inducements::bb2020::prayers::prayer_handler_factory;

        // Java: `prayerFactory.forRoll(roll)` then `handlerFactory.forPrayer(...)`.
        // Java reads this through `getOptionWithDefault`, whose factory default for
        // INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE is TRUE (16 prayers). Rust's `options.is_enabled`
        // returns FALSE for an unset option -- it does not consult the factory -- so the league
        // table was silently off and only rolls 1..=8 were considered, picking a different prayer
        // and consuming the Collections stream differently from Java.
        let use_league = game.options
            .get_option_with_default(ffb_model::option::game_option_id::GameOptionId::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE)
            .get_value_as_string() == "true";
        let mut factory = PrayerFactory::new();
        factory.initialize(use_league);
        let handler = factory
            .for_roll(self.roll)
            .and_then(|prayer| prayer_handler_factory::for_prayer(&format!("{prayer:?}")));

        let team_id = self.team_id.clone().unwrap_or_default();

        if self.first_run {
            self.first_run = false;
            // Java: getResult().addReport(new ReportPrayerRoll(roll))
            game.report_list.add(ReportPrayerRoll::new(self.roll));
            // Coverage: `GameEvent::PrayerRoll` had no construction site in the engine. Prayers
            // demonstrably fire — an IRON_MAN prayer is what exposed the missing stat-limit clamp —
            // yet the counter read 0. This is the twin the driver actually routes to for BB2020
            // (`driver.rs:390`), the edition that HAS prayers; patching only the BB2025 file left
            // the counter at 0, which is how this was caught.
            self.pending_prayer_event = Some(GameEvent::PrayerRoll {
                team_id: team_id.clone(),
                roll: self.roll,
                prayer_id: handler.as_ref().map(|h| h.handled_prayer_name().to_string())
                    .unwrap_or_else(|| self.roll.to_string()),
            });

            match handler {
                // Java: `prayerHandler.get().initEffect(this, gameState, teamId)` — note Java does
                // NOT set NEXT_STEP on this branch; the handler decides. Our trait returns
                // "effect fully applied", i.e. false means it is waiting on a dialog.
                Some(h) => {
                    // Java's FINAL `PrayerHandler.initEffect(step, gameState, prayingTeamId)` does
                    // `inducementSet.addPrayer(handledPrayer())` BEFORE delegating to the concrete
                    // effect (`PrayerHandler.java:46-47`). Rust's trait has no such wrapper — the
                    // handlers implement the concrete `init_effect` directly — so the recording was
                    // simply never done, and NOT ONE granted prayer ever landed in an inducement
                    // set. `StepEndTurn.deactivatePrayers` iterates exactly that collection, so no
                    // prayer effect could ever expire: Treacherous Trapdoor's doors (BB2020
                    // duration `UntilEndOfHalf`) survived into the second half and a player standing
                    // on one rolled a Trap Door d6 Java never rolls (slann_fumbbl bb2020 seed 29).
                    let prayer_name = h.handled_prayer_name();
                    if game.team_home.id == team_id {
                        game.turn_data_home.inducement_set.add_prayer(prayer_name);
                    } else {
                        game.turn_data_away.inducement_set.add_prayer(prayer_name);
                    }
                    let mut prayer_state = std::mem::take(&mut game.prayer_state);
                    let applied = h.init_effect(&mut prayer_state, game, rng, &team_id);
                    game.prayer_state = prayer_state;
                    if applied {
                        StepOutcome::next()
                    } else if let Some((player_id, skills)) = h.skill_dialog(game) {
                        // Java: `UtilServerDialog.showDialog(gameState, new DialogSelectSkillParameter(
                        //      player.getId(), skills, SkillChoiceMode.INTENSIVE_TRAINING), false)`.
                        // The skills are already in Java's order (sorted by name); the prompt groups
                        // them by category because that is the shape `AgentPrompt::SelectSkill` takes.
                        self.player_id = Some(player_id.clone());
                        let rules = game.rules;
                        let mut available: Vec<(ffb_model::enums::SkillCategory, Vec<u16>)> = Vec::new();
                        for s in &skills {
                            let cat = s.category_and_name_for(rules).0;
                            match available.iter_mut().find(|(c, _)| *c == cat) {
                                Some((_, v)) => v.push(*s as u16),
                                None => available.push((cat, vec![*s as u16])),
                            }
                        }
                        StepOutcome::cont().with_prompt(
                            ffb_model::prompts::AgentPrompt::SelectSkill { player_id, available })
                    } else if let Some(mode) = h.dialog_choice_mode() {
                        // Java: `SelectPlayerPrayerHandler.createDialog` →
                        // `UtilServerDialog.showDialog(gameState, new DialogPlayerChoiceParameter(
                        //      prayingTeam.getId(), choiceMode(), playerIds, …))`.
                        // The mode is declared non-declinable, so the coach MUST answer; without
                        // the prompt the step returned `Continue` with no dialog, which is the
                        // engine's definition of a stall (and Java's ParityRunner reported
                        // `UNHANDLED_STEP: PRAYER`, then NPE'd in applySelection on a null player).
                        StepOutcome::cont().with_prompt(
                            ffb_model::prompts::AgentPrompt::PlayerChoice {
                                eligible_players: h.eligible_dialog_players(game, &team_id),
                                reason: mode.to_string(),
                                descriptions: Vec::new(),
                            })
                    } else {
                        StepOutcome::cont()
                    }
                }
                // Java: `else { getResult().setNextAction(NEXT_STEP); }`
                None => StepOutcome::next(),
            }
        } else {
            // Java: `prayerHandler.ifPresent(h -> h.applySelection(...)); setNextAction(NEXT_STEP);`
            if let Some(h) = handler {
                // Java: `new PrayerDialogSelection(playerId, skill)` — the id the coach chose,
                // NOT the praying team's id (which is what this passed before).
                let player_id = self.player_id.clone().unwrap_or_default();
                let mut prayer_state = std::mem::take(&mut game.prayer_state);
                match self.selected_skill {
                    // The selection carried a skill (Intensive Training).
                    Some(skill_id) => h.apply_skill_selection(&mut prayer_state, game, &player_id, skill_id),
                    None => h.apply_selection(&mut prayer_state, game, &player_id),
                }
                game.prayer_state = prayer_state;
            }
            StepOutcome::next()
        }
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
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_sets_first_run_false() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        assert!(step.first_run);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.first_run);
    }

    #[test]
    fn handle_command_select_player_stores_id() {
        let mut game = make_game();
        let mut step = StepPrayer::new(3, "home");
        let action = Action::SelectPlayer { player_id: "p1".into() };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn prayer_roll_parameter_accepted() {
        let mut step = StepPrayer::default();
        step.set_parameter(&StepParameter::PrayerRoll(5));
        assert_eq!(step.roll, 5);
    }

    #[test]
    fn team_id_parameter_accepted() {
        let mut step = StepPrayer::default();
        step.set_parameter(&StepParameter::TeamId("away".into()));
        assert_eq!(step.team_id.as_deref(), Some("away"));
    }
}
