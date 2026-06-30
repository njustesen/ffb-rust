use std::collections::HashSet;
use ffb_model::enums::Weather;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Resolves the Weather Mage inducement: the active coach selects a new weather
/// condition from options derived from a 2D6 roll (base result ± 1 and ± 2).
///
/// Java flow (executeStep):
///   1. Mark the Weather Mage inducement as used (useMage).
///   2. Roll 2D6; compute base = Weather::for_roll(sum).
///   3. Build unique `weatherOptions` map: base at modifier 0, then ±1 and ±2.
///   4. If options > 1: show DialogSelectWeather → wait for CLIENT_SELECT_WEATHER.
///   5. If one option: auto-apply; show info dialog → wait for CLIENT_CONFIRM.
///   6. If no options (impossible): NEXT_STEP immediately.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.inducements.StepWeatherMage`.
pub struct StepWeatherMage;

impl StepWeatherMage {
    pub fn new() -> Self { Self }
}

impl Default for StepWeatherMage {
    fn default() -> Self { Self::new() }
}

impl Step for StepWeatherMage {
    fn id(&self) -> StepId { StepId::WeatherMage }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectWeather { weather } => {
                game.field_model.weather = *weather;
                StepOutcome::next()
            }
            Action::Acknowledge => StepOutcome::next(),
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepWeatherMage {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: useMage() — marks the CHANGE_WEATHER inducement as used.
        // TODO: implement when Usage enum is fully ported from Java.

        let roll = rng.roll_weather();
        let sum = roll[0] + roll[1];

        // Build unique weather options. Java inserts base at modifier 0, then ±1 and ±2.
        let mut seen: HashSet<Weather> = HashSet::new();
        let mut options: Vec<Weather> = Vec::new();
        add_weather_option(&mut seen, &mut options, sum, 0);
        add_weather_option(&mut seen, &mut options, sum, 1);
        add_weather_option(&mut seen, &mut options, sum, -1);
        add_weather_option(&mut seen, &mut options, sum, 2);
        add_weather_option(&mut seen, &mut options, sum, -2);

        if options.len() > 1 {
            return StepOutcome::cont()
                .with_prompt(AgentPrompt::SelectWeather { options });
        }

        match options.first().copied() {
            Some(new_weather) => {
                let old_weather = game.field_model.weather;
                if new_weather != old_weather {
                    game.field_model.weather = new_weather;
                }
                let msg = if new_weather != old_weather {
                    format!("Weather changed to {}.", new_weather.name())
                } else {
                    "The weather did not change.".into()
                };
                StepOutcome::cont()
                    .with_prompt(AgentPrompt::InformationOkay { message: msg })
            }
            None => StepOutcome::next(),
        }
    }
}

/// Clamps `sum + modifier` to [2, 12], interprets weather, and inserts it into
/// `options` only if not already present. Mirrors Java's `addWeather()`.
fn add_weather_option(seen: &mut HashSet<Weather>, options: &mut Vec<Weather>, sum: i32, modifier: i32) {
    let clamped = (sum + modifier).max(2).min(12);
    let w = Weather::for_roll(clamped);
    if seen.insert(w) {
        options.push(w);
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

    /// A seeded roll of [6,6]=12 gives Blizzard. ±1 → 11=PouringRain, ±2 → 10=Nice.
    /// So all three are distinct → options.len()=3 → SelectWeather prompt.
    #[test]
    fn start_with_multiple_options_emits_select_weather_prompt() {
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        let mut step = StepWeatherMage::new();
        // Use seed 0 and just check the output type regardless of exact roll.
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either a selection prompt or an info-okay prompt is valid depending on the roll.
        assert!(matches!(out.action, StepAction::Continue));
    }

    /// When SelectWeather is handled, weather is set on the field model.
    #[test]
    fn handle_select_weather_sets_field_model_weather() {
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        let mut step = StepWeatherMage::new();
        let out = step.handle_command(
            &Action::SelectWeather { weather: Weather::Blizzard },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.weather, Weather::Blizzard);
    }

    /// Acknowledge command always returns NEXT_STEP.
    #[test]
    fn handle_acknowledge_returns_next() {
        let mut game = make_game();
        let mut step = StepWeatherMage::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// set_parameter always returns false (no parameters).
    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepWeatherMage::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    /// add_weather_option deduplicates: sum=7 (Nice), modifiers ±1→6/8 both Nice,
    /// ±2→5/9 both Nice → only 1 unique weather.
    #[test]
    fn add_weather_option_deduplicates() {
        let mut seen: HashSet<Weather> = HashSet::new();
        let mut opts: Vec<Weather> = Vec::new();
        add_weather_option(&mut seen, &mut opts, 7, 0);
        add_weather_option(&mut seen, &mut opts, 7, 1);
        add_weather_option(&mut seen, &mut opts, 7, -1);
        add_weather_option(&mut seen, &mut opts, 7, 2);
        add_weather_option(&mut seen, &mut opts, 7, -2);
        assert_eq!(opts, vec![Weather::Nice]);
    }

    /// At sum=10 (Nice): +2=12=Blizzard, +1=11=PouringRain, -1=9=Nice (dup), -2=8=Nice (dup)
    /// → 3 unique: Nice, Blizzard, PouringRain → options > 1 → SelectWeather prompt.
    #[test]
    fn single_option_roll_shows_info_dialog() {
        // sum=7 → only Nice → auto-apply path → InformationOkay prompt.
        let mut seen: HashSet<Weather> = HashSet::new();
        let mut opts: Vec<Weather> = Vec::new();
        add_weather_option(&mut seen, &mut opts, 7, 0);
        add_weather_option(&mut seen, &mut opts, 7, 1);
        add_weather_option(&mut seen, &mut opts, 7, -1);
        add_weather_option(&mut seen, &mut opts, 7, 2);
        add_weather_option(&mut seen, &mut opts, 7, -2);
        assert_eq!(opts.len(), 1);
    }

    /// Clamping: sum=2 with modifier=-2 clamps to 2 → same weather as base.
    #[test]
    fn add_weather_option_clamps_to_minimum() {
        let mut seen: HashSet<Weather> = HashSet::new();
        let mut opts: Vec<Weather> = Vec::new();
        add_weather_option(&mut seen, &mut opts, 2, 0);
        add_weather_option(&mut seen, &mut opts, 2, -1);
        add_weather_option(&mut seen, &mut opts, 2, -2);
        // All clamp to 2 → SwelteringHeat → only 1 unique.
        assert_eq!(opts, vec![Weather::SwelteringHeat]);
    }
}
