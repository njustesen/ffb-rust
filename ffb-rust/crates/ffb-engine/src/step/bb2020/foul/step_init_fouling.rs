use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initializes the foul sequence: validates target, sets defender, increments foul counter.
/// Handles CLIENT_END_TURN and CLIENT_ACTING_PLAYER before a foul target is selected.
/// BB2020: on end_turn path publishes EndTurn(true) only — no CheckForgo (not in BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.foul.StepInitFouling`.
pub struct StepInitFouling {
    pub goto_label_on_end: String,
    pub foul_defender_id: Option<String>,
    pub end_turn: bool,
    pub end_player_action: bool,
    pub using_chainsaw: bool,
}

impl StepInitFouling {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            foul_defender_id: None,
            end_turn: false,
            end_player_action: false,
            using_chainsaw: false,
        }
    }

    pub fn with_defender(mut self, defender_id: String) -> Self {
        self.foul_defender_id = Some(defender_id);
        self
    }
}

impl Step for StepInitFouling {
    fn id(&self) -> StepId { StepId::InitFouling }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            Action::Foul { target_id } => {
                self.foul_defender_id = Some(target_id.clone());
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }
}

impl StepInitFouling {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if self.end_turn {
            let label = self.goto_label_on_end.clone();
            // BB2020: publish EndTurn only — no CheckForgo
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            let label = self.goto_label_on_end.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        let fouler_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let foul_defender_id = match self.foul_defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let fouler_player = game.team_home.players.iter()
            .chain(game.team_away.players.iter())
            .find(|p| p.id == fouler_id)
            .cloned();

        let defender_exists = game.team_home.players.iter()
            .chain(game.team_away.players.iter())
            .any(|p| p.id == foul_defender_id);

        if game.acting_player.has_fouled || !defender_exists {
            return StepOutcome::next();
        }

        game.acting_player.defender_id = Some(foul_defender_id.clone());
        game.acting_player.has_fouled = true;
        game.turn_data_mut().turn_started = true;
        game.concession_possible = false;

        // Increment foul count in game result.
        let home_has_fouler = game.team_home.players.iter().any(|p| p.id == fouler_id);
        let results = if home_has_fouler {
            &mut game.game_result.home.player_results
        } else {
            &mut game.game_result.away.player_results
        };
        results.entry(fouler_id.clone()).or_default().fouls += 1;

        // SneakiestOfTheLot (NamedProperties.allowsAdditionalFoul) exempts from foul_used.
        let has_additional_foul = fouler_player
            .as_ref()
            .map(|p| p.has_skill(SkillId::SneakiestOfTheLot))
            .unwrap_or(false);
        if !has_additional_foul {
            game.turn_data_mut().foul_used = true;
        }

        StepOutcome::next()
            .publish(StepParameter::UsingChainsaw(self.using_chainsaw))
            .publish(StepParameter::BlockDefenderId(foul_defender_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    fn bare_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 0,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0,
            race: None,
            ..Default::default()
        }
    }

    // 1. EndTurn path → GotoLabel with EndTurn published but NOT CheckForgo (BB2020)
    #[test]
    fn end_turn_goto_label_no_check_forgo() {
        let mut game = make_game();
        let mut step = StepInitFouling::new("endLabel".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("endLabel"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        // BB2020 does NOT publish CheckForgo
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(_))));
    }

    // 2. No acting player yet → NextStep (waiting for activation)
    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepInitFouling::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // 3. Valid foul: acting player + defender present → NextStep, publishes BlockDefenderId
    #[test]
    fn valid_foul_publishes_defender_id() {
        let mut game = make_game();
        game.team_home.players.push(bare_player("fouler1"));
        game.team_away.players.push(bare_player("defender1"));
        game.acting_player.player_id = Some("fouler1".into());

        let mut step = StepInitFouling::new("end".into()).with_defender("defender1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(id) if id == "defender1")));
    }

    // 4. Valid foul increments foul counter in game_result
    #[test]
    fn valid_foul_increments_foul_count() {
        let mut game = make_game();
        game.team_home.players.push(bare_player("fouler2"));
        game.team_away.players.push(bare_player("defender2"));
        game.acting_player.player_id = Some("fouler2".into());

        let mut step = StepInitFouling::new("end".into()).with_defender("defender2".into());
        step.start(&mut game, &mut GameRng::new(0));
        let fouls = game.game_result.home.player_results
            .get("fouler2").map(|r| r.fouls).unwrap_or(0);
        assert_eq!(fouls, 1);
    }

    // No Java addReport calls in StepInitFouling — verify the step produces no spurious reports.
    #[test]
    fn no_reports_emitted_on_valid_foul() {
        let mut game = make_game();
        game.team_home.players.push(bare_player("f1"));
        game.team_away.players.push(bare_player("d1"));
        game.acting_player.player_id = Some("f1".into());
        let mut step = StepInitFouling::new("end".into()).with_defender("d1".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.report_list.size(), 0, "StepInitFouling must not emit any reports");
    }

    #[test]
    fn no_reports_emitted_on_end_turn() {
        let mut game = make_game();
        let mut step = StepInitFouling::new("end".into());
        step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(game.report_list.size(), 0, "StepInitFouling end_turn path must not emit any reports");
    }

    // 5. Foul command sets defender then resolves
    #[test]
    fn foul_command_sets_defender_and_resolves() {
        let mut game = make_game();
        game.team_home.players.push(bare_player("fouler3"));
        game.team_away.players.push(bare_player("defender3"));
        game.acting_player.player_id = Some("fouler3".into());

        let mut step = StepInitFouling::new("end".into());
        let action = Action::Foul { target_id: "defender3".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(id) if id == "defender3")));
    }
}
