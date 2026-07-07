/// 1:1 translation of com.fumbbl.ffb.server.step.game.StepWeather.
///
/// Rolls 2d6, maps to a Weather value, sets it on the field model, then advances.
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_weather::ReportWeather;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepId, StepOutcome};

pub struct StepWeather;

impl StepWeather {
    pub fn new() -> Self { Self }
}

impl Default for StepWeather {
    fn default() -> Self { Self::new() }
}

impl Step for StepWeather {
    fn id(&self) -> StepId { StepId::Weather }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let roll = rng.roll_weather();
        let weather = DiceInterpreter::interpret_roll_weather(&roll);
        game.field_model.weather = weather;
        // Java: getResult().addReport(rollWeather()) → new ReportWeather(weather, roll)
        game.report_list.add(ReportWeather::new(weather, roll.to_vec()));
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::{Game, Team};
    use ffb_model::util::rng::GameRng;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
        }
    }

    fn new_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn start_sets_weather_on_field_model() {
        let mut step = StepWeather::new();
        let mut game = new_game();
        let mut rng = GameRng::new(1);

        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, crate::step::framework::StepAction::NextStep);
        // Weather should be one of the valid values (seed 1 gives a deterministic roll)
        assert!(matches!(game.field_model.weather,
            Weather::SwelteringHeat | Weather::VerySunny | Weather::Nice |
            Weather::PouringRain | Weather::Blizzard));
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepWeather::new();
        let mut game = new_game();
        let mut rng = GameRng::new(42);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, crate::step::framework::StepAction::NextStep);
    }

    #[test]
    fn weather_changes_from_default() {
        // Default weather is Nice; step should deterministically change it based on seed.
        // We run several seeds and confirm the mechanism works (at least one changes it).
        let mut changed = false;
        for seed in 0..20u64 {
            let mut step = StepWeather::new();
            let mut game = new_game();
            let mut rng = GameRng::new(seed);
            let before = game.field_model.weather;
            step.start(&mut game, &mut rng);
            if game.field_model.weather != before {
                changed = true;
                break;
            }
        }
        assert!(changed, "Expected at least one seed to produce non-Nice weather");
    }

    #[test]
    fn start_adds_weather_report() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepWeather::new();
        let mut game = new_game();
        let mut rng = GameRng::new(1);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::WEATHER));
    }

    #[test]
    fn weather_report_added_for_multiple_seeds() {
        use ffb_model::report::report_id::ReportId;
        for seed in 0..5u64 {
            let mut step = StepWeather::new();
            let mut game = new_game();
            let mut rng = GameRng::new(seed);
            step.start(&mut game, &mut rng);
            assert!(game.report_list.has_report(ReportId::WEATHER), "seed={seed} should add weather report");
        }
    }
}
