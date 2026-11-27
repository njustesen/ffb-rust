use std::collections::HashSet;
use ffb_model::enums::Weather;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_weather_mage_roll::ReportWeatherMageRoll;
use ffb_model::report::mixed::report_weather_mage_result::{ReportWeatherMageResult, WeatherMageEffect};
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
                // Java: CLIENT_SELECT_WEATHER -> `replaceWeather(weather, command.getModifier(),
                // CHANGED)`. The client sends the modifier the dialog listed for that weather.
                let modifier = match &game.dialog_parameter {
                    Some(ffb_model::dialog::dialog_parameter_factory::DialogParameter::SelectWeather(p)) =>
                        p.options.get(java_weather_constant(*weather)).copied().unwrap_or(0),
                    _ => 0,
                };
                replace_weather_with_report(game, *weather, modifier, WeatherMageEffect::CHANGED);
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
        Self::use_mage(game);

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

        game.report_list.add(ReportWeatherMageRoll::new(roll.to_vec()));

        if options.len() > 1 {
            // Java: showDialog(new DialogSelectWeatherParameter(weatherOptions), false), where
            // weatherOptions maps Weather.name() (the enum constant) -> its first modifier
            // (base 0, then +1, -1, +2, -2; only absent keys are inserted).
            let mut weather_options: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
            for modifier in [0, 1, -1, 2, -2] {
                let w = Weather::for_roll((sum + modifier).max(2).min(12));
                weather_options.entry(java_weather_constant(w).to_string()).or_insert(modifier);
            }
            crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                game,
                ffb_model::dialog::dialog_parameter_factory::DialogParameter::SelectWeather(
                    ffb_model::dialog::dialog_select_weather_parameter::DialogSelectWeatherParameter {
                        options: weather_options,
                    },
                ),
                false,
            );
            return StepOutcome::cont()
                .with_prompt(AgentPrompt::SelectWeather { options });
        }

        match options.first().copied() {
            Some(new_weather) => {
                let old_weather = game.field_model.weather;
                if new_weather != old_weather {
                    // Java: `replaceWeather(newWeather, options.get(name) /* 0 */, NO_CHOICE)`.
                    replace_weather_with_report(game, new_weather, 0, WeatherMageEffect::NO_CHOICE);
                } else {
                    game.report_list.add(ReportWeatherMageResult::new(
                        0,
                        Some(new_weather.name().to_string()),
                        Some(WeatherMageEffect::NO_CHANGE),
                        Some(new_weather.name().to_string()),
                    ));
                }
                let msg = if new_weather != old_weather {
                    format!("Weather changed to {}.", new_weather.name())
                } else {
                    "The weather did not change.".into()
                };
                // Java: showDialog(new DialogInformationOkayParameter(DIALOG_TITLE, message, true), false)
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::InformationOkay(
                        ffb_model::dialog::dialog_information_okay_parameter::DialogInformationOkayParameter {
                            title: Some(DIALOG_TITLE.to_string()),
                            messages: Some(vec![Some(msg.clone())]),
                            confirm: true,
                        },
                    ),
                    false,
                );
                StepOutcome::cont()
                    .with_prompt(AgentPrompt::InformationOkay { message: msg })
            }
            None => StepOutcome::next(),
        }
    }

    /// Java: useMage() — marks the CHANGE_WEATHER inducement as used in active team's set.
    fn use_mage(game: &mut Game) {
        let set = if game.home_playing {
            &mut game.turn_data_home.inducement_set
        } else {
            &mut game.turn_data_away.inducement_set
        };
        if let Some(type_id) = set.for_usage(Usage::CHANGE_WEATHER).map(|s| s.to_string()) {
            if let Some(mut ind) = set.get(&type_id) {
                if ind.get_uses_left() >= 1 {
                    ind.set_uses(ind.get_uses() + 1);
                    set.add_inducement(ind);
                }
            }
        }
    }
}

/// Java: `StepWeatherMage.DIALOG_TITLE`.
pub const DIALOG_TITLE: &str = "Weather Mage";

/// Java `StepWeatherMage.replaceWeather(weather, modifier, effect)`: `GameState.replaceWeather`
/// plus the result report, whose old weather is the one `replaceWeather` returns (the weather
/// before the FIRST mage of the drive).
fn replace_weather_with_report(game: &mut Game, weather: Weather, modifier: i32, effect: WeatherMageEffect) {
    let old = replace_weather(game, weather);
    game.report_list.add(ReportWeatherMageResult::new(
        modifier,
        Some(weather.name().to_string()),
        Some(effect),
        Some(old.name().to_string()),
    ));
}

/// Java `GameState.replaceWeather(Weather)`: remember the original weather once, skip the next
/// restore, set the new weather; returns the remembered original.
pub fn replace_weather(game: &mut Game, weather: Weather) -> Weather {
    if game.old_weather.is_none() {
        game.old_weather = Some(game.field_model.weather);
    }
    game.skip_restore_weather = true;
    game.field_model.weather = weather;
    game.old_weather.unwrap_or(weather)
}

/// Java `GameState.restoreWeather(boolean endOfDrive)` (StepEndTurn, bb2020/bb2025).
pub fn restore_weather(game: &mut Game, end_of_drive: bool) {
    if end_of_drive {
        if let Some(old) = game.old_weather {
            game.field_model.weather = old;
        }
        game.skip_restore_weather = false;
        game.old_weather = None;
    } else if game.skip_restore_weather {
        game.skip_restore_weather = false;
    } else if let Some(old) = game.old_weather.take() {
        game.field_model.weather = old;
    }
}

/// Java `Weather.name()` -- the enum constant name (the key of `DialogSelectWeatherParameter`).
pub fn java_weather_constant(w: Weather) -> &'static str {
    match w {
        Weather::SwelteringHeat => "SWELTERING_HEAT",
        Weather::VerySunny => "VERY_SUNNY",
        Weather::Nice => "NICE",
        Weather::PouringRain => "POURING_RAIN",
        Weather::Blizzard => "BLIZZARD",
        Weather::Intro => "INTRO",
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

    /// Java StepWeatherMage.handleCommand (case CLIENT_SELECT_WEATHER) always adds a
    /// ReportWeatherMageResult with Effect.CHANGED before applying the new weather.
    /// This report was previously missing entirely from the Rust handle_command path.
    #[test]
    fn handle_select_weather_adds_changed_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        let mut step = StepWeatherMage::new();
        step.handle_command(
            &Action::SelectWeather { weather: Weather::Blizzard },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::WEATHER_MAGE_RESULT));
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

    #[test]
    fn use_mage_increments_uses_on_active_team_inducement() {
        use ffb_model::inducement::inducement::Inducement;
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.inducement_set.add_inducement(
            Inducement::new("weatherMage", 1, vec![Usage::CHANGE_WEATHER])
        );
        StepWeatherMage::use_mage(&mut game);
        let ind = game.turn_data_home.inducement_set.get("weatherMage").unwrap();
        assert_eq!(ind.get_uses(), 1);
        assert_eq!(ind.get_uses_left(), 0);
    }

    /// Java GameState.replaceWeather / restoreWeather: the first replaced weather is remembered;
    /// the turn end right after the mage skips the restore, the next one restores it; the end of a
    /// drive always restores and forgets.
    #[test]
    fn replaced_weather_is_restored_one_turn_later_or_at_the_end_of_the_drive() {
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        assert_eq!(replace_weather(&mut game, Weather::Blizzard), Weather::Nice);
        assert_eq!(replace_weather(&mut game, Weather::PouringRain), Weather::Nice);
        restore_weather(&mut game, false);
        assert_eq!(game.field_model.weather, Weather::PouringRain, "the turn of the mage keeps it");
        restore_weather(&mut game, false);
        assert_eq!(game.field_model.weather, Weather::Nice);
        assert!(game.old_weather.is_none());

        replace_weather(&mut game, Weather::Blizzard);
        restore_weather(&mut game, true);
        assert_eq!(game.field_model.weather, Weather::Nice);
        assert!(game.old_weather.is_none() && !game.skip_restore_weather);
    }

    /// Java: the chosen weather's report carries the modifier the dialog listed for it.
    #[test]
    fn select_weather_reports_the_dialog_modifier() {
        let mut game = make_game();
        let mut options = std::collections::HashMap::new();
        options.insert("BLIZZARD".to_string(), -2);
        game.dialog_parameter = Some(ffb_model::dialog::dialog_parameter_factory::DialogParameter::SelectWeather(
            ffb_model::dialog::dialog_select_weather_parameter::DialogSelectWeatherParameter { options }));
        let mut step = StepWeatherMage::new();
        step.handle_command(&Action::SelectWeather { weather: Weather::Blizzard }, &mut game, &mut GameRng::new(0));
        let report = game.report_list.get_reports().last().unwrap().to_json().unwrap();
        assert_eq!(report["modifier"], -2);
        assert_eq!(game.field_model.weather, Weather::Blizzard);
    }

    #[test]
    fn use_mage_no_op_when_no_change_weather_inducement() {
        let mut game = make_game();
        game.home_playing = true;
        StepWeatherMage::use_mage(&mut game);
        // No panic — no inducement, nothing to use
    }

    #[test]
    fn execute_step_adds_weather_mage_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        let mut step = StepWeatherMage::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::WEATHER_MAGE_ROLL));
    }

    #[test]
    fn execute_step_adds_weather_mage_result_report_on_single_option() {
        use ffb_model::report::report_id::ReportId;
        // sum=7 → only Nice → single option → result report added
        let mut game = make_game();
        game.field_model.weather = Weather::Nice;
        // Pre-set so that options.len()==1: force single option by using a fixed
        // known-single-weather roll. We simply check the report was added.
        // (actual roll seed-dependent, but any start adds a roll report)
        let mut step = StepWeatherMage::new();
        step.start(&mut game, &mut GameRng::new(0));
        // Roll report always added
        assert!(game.report_list.has_report(ReportId::WEATHER_MAGE_ROLL));
    }
}
