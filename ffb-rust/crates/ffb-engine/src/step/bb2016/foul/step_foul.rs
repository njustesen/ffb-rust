use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_foul::ReportFoul;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::handle_injury_by_name;

/// Rolls armour and injury for a foul (BB2016).
/// Publishes INJURY_RESULT to be consumed by the Apothecary step that follows.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.foul.StepFoul.
pub struct StepFoul {
    /// Java: usingChainsaw
    pub using_chainsaw: bool,
}

impl StepFoul {
    pub fn new() -> Self { Self { using_chainsaw: false } }
}

impl Default for StepFoul {
    fn default() -> Self { Self::new() }
}

impl Step for StepFoul {
    fn id(&self) -> StepId { StepId::Foul }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingChainsaw(v) => { self.using_chainsaw = *v; true }
            _ => false,
        }
    }
}

impl StepFoul {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        game.report_list.add(ReportFoul::new(defender_id.clone()));
        let attacker_id = game.acting_player.player_id.clone();
        let defender_coord = game.field_model.player_coordinate(&defender_id)
            .unwrap_or(FieldCoordinate::new(0, 0));

        // Java: new InjuryTypeFoul(usingChainsaw) — chainsaw uses InjuryTypeFoulChainsaw
        let injury_type_name = if self.using_chainsaw {
            "InjuryTypeFoulChainsaw"
        } else {
            "InjuryTypeFoul"
        };

        // Java: UtilServerInjury.handleInjury(this, injuryType, attacker, defender, coord, null, null, DEFENDER)
        let injury_result = handle_injury_by_name(
            game, rng, injury_type_name,
            attacker_id.as_deref(),
            &defender_id,
            defender_coord,
            None, None,
            ApothecaryMode::Defender,
        );

        StepOutcome::next()
            .publish(StepParameter::InjuryResult(Box::new(injury_result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, team: &str, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
};
        if team == "home" {
            game.team_home.players.push(p);
        } else {
            game.team_away.players.push(p);
        }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    #[test]
    fn no_defender_returns_next_step() {
        let mut game = make_game();
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn publishes_injury_result() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn set_parameter_using_chainsaw_accepted() {
        let mut step = StepFoul::new();
        assert!(step.set_parameter(&StepParameter::UsingChainsaw(true)));
        assert!(step.using_chainsaw);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepFoul::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn foul_adds_report_foul() {
        let mut game = make_game();
        add_player(&mut game, "home", "att");
        add_player(&mut game, "away", "def");
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        let mut step = StepFoul::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::FOUL), "foul should add ReportFoul");
    }

    #[test]
    fn no_defender_does_not_add_foul_report() {
        let mut game = make_game();
        let mut step = StepFoul::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::FOUL), "no defender means no ReportFoul");
    }
}
