/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepPrayers` (BB2020).
///
/// Determines how many Prayers to Nuffle the lower-TV team receives, builds a sequence of
/// StepPrayer steps (one per prayer), and pushes them onto the stack.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepPrayers`.
use ffb_model::events::GameEvent;
use ffb_model::factory::bb2020::prayer_factory::PrayerFactory;
use ffb_model::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id;
use ffb_model::report::mixed::report_prayer_amount::ReportPrayerAmount;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, SequenceStep};
use crate::step::framework::{StepId, StepParameter};

pub struct StepPrayers {
    tv_home: i32,
    tv_away: i32,
}

impl StepPrayers {
    pub fn new() -> Self {
        Self { tv_home: 0, tv_away: 0 }
    }
}

impl Default for StepPrayers {
    fn default() -> Self { Self::new() }
}

impl Step for StepPrayers {
    fn id(&self) -> StepId { StepId::Prayers }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TvHome(v) => { self.tv_home = *v; true }
            StepParameter::TvAway(v) => { self.tv_away = *v; true }
            _ => false,
        }
    }
}

impl StepPrayers {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let max_prayers = game.options.get_int("inducementPrayersMax").unwrap_or(0);
        let prayer_cost = {
            let c = game.options.get_int("inducementPrayersCost").unwrap_or(0);
            if c <= 0 { 50_000 } else { c }
        };

        let mut prayer_amount = (self.tv_home - self.tv_away).unsigned_abs() as i32 / prayer_cost;

        if max_prayers > 0 {
            prayer_amount = prayer_amount.min(max_prayers);
        }

        if prayer_amount > 0 {
            let home_team_receives = self.tv_home < self.tv_away;

            // Java: PrayerFactory prayerFactory = game.getFactory(FactoryType.Factory.PRAYER);
            // Exhibition: rolls 1–8; league (INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE): rolls 1–16.
            let use_league = game.options.is_enabled(game_option_id::INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE);
            let mut factory = PrayerFactory::new();
            factory.initialize(use_league);

            // Java: availablePrayerRolls = prayerFactory.availablePrayerRolls(receivingTeam.getInducementSet(), otherTeam.getInducementSet())
            // — excludes prayers the receiving team already holds, and "affects both teams"
            // prayers the opponent already holds.
            let mut available_rolls: Vec<i32> = if home_team_receives {
                factory.available_prayer_rolls(&game.turn_data_home.inducement_set, &game.turn_data_away.inducement_set)
            } else {
                factory.available_prayer_rolls(&game.turn_data_away.inducement_set, &game.turn_data_home.inducement_set)
            };

            prayer_amount = prayer_amount.min(available_rolls.len() as i32);

            let prayer_amount_event = GameEvent::PrayerAmount {
                tv_home: self.tv_home,
                tv_away: self.tv_away,
                prayer_amount,
                home_team_receives_prayers: home_team_receives,
            };

            let praying_team_id = if home_team_receives {
                game.team_home.id.clone()
            } else {
                game.team_away.id.clone()
            };

            let mut seq: Vec<SequenceStep> = Vec::new();
            for _ in 0..prayer_amount {
                let idx = rng.range(available_rolls.len());
                let roll = available_rolls.remove(idx);
                let mut step = SequenceStep::new(StepId::Prayer);
                step.params.push(StepParameter::PrayerRoll(roll));
                step.params.push(StepParameter::TeamId(praying_team_id.clone()));
                seq.push(step);
            }
            // Stack is LIFO: reverse so first prayer runs first.
            seq.reverse();
            // Java: getResult().addReport(new ReportPrayerAmount(tvHome, tvAway, prayerAmount, homeTeamReceivesPrayers))
            game.report_list.add(ReportPrayerAmount::new(self.tv_home, self.tv_away, prayer_amount, home_team_receives));
            return StepOutcome::next().with_event(prayer_amount_event).push_seq(seq);
        }

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
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_tv_difference_returns_next_with_no_push() {
        let mut game = make_game();
        let mut step = StepPrayers::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn tv_difference_below_cost_no_prayers() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_049_999; // diff = 49_999 < 50_000
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn tv_difference_yields_one_prayer() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_050_000; // diff = 50_000, 1 prayer
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::Prayer);
    }

    #[test]
    fn home_lower_tv_home_team_prays() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 900_000;
        step.tv_away = 950_000; // home is lower, home prays
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let seq = &out.pushes[0];
        assert_eq!(seq.len(), 1);
        let has_team_param = seq[0].params.iter().any(|p| matches!(p, StepParameter::TeamId(id) if id == "home"));
        assert!(has_team_param, "expected TeamId=home_lineman");
    }

    #[test]
    fn away_lower_tv_away_team_prays() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 950_000;
        step.tv_away = 900_000; // away is lower, away prays
        let out = step.start(&mut game, &mut GameRng::new(0));
        let seq = &out.pushes[0];
        let has_team_param = seq[0].params.iter().any(|p| matches!(p, StepParameter::TeamId(id) if id == "away"));
        assert!(has_team_param, "expected TeamId=away_lineman");
    }

    #[test]
    fn max_prayers_caps_count() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        game.options.set("inducementPrayersMax", "2");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_500_000; // diff = 500_000, 10 prayers but capped to 2
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes[0].len(), 2);
    }

    #[test]
    fn set_parameter_tv_home_accepted() {
        let mut step = StepPrayers::new();
        assert!(step.set_parameter(&StepParameter::TvHome(1_000_000)));
        assert_eq!(step.tv_home, 1_000_000);
    }

    #[test]
    fn set_parameter_tv_away_accepted() {
        let mut step = StepPrayers::new();
        assert!(step.set_parameter(&StepParameter::TvAway(900_000)));
        assert_eq!(step.tv_away, 900_000);
    }

    #[test]
    fn default_prayer_cost_used_when_option_absent() {
        let mut game = make_game();
        // No inducementPrayersCost set, defaults to 50_000
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_050_000; // 1 prayer at 50_000 cost
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes[0].len(), 1);
    }

    #[test]
    fn each_prayer_has_prayer_roll_param() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_100_000; // 2 prayers
        let out = step.start(&mut game, &mut GameRng::new(42));
        let seq = &out.pushes[0];
        assert_eq!(seq.len(), 2);
        for s in seq {
            assert!(s.params.iter().any(|p| matches!(p, StepParameter::PrayerRoll(_))));
        }
    }

    #[test]
    fn exhibition_mode_limits_rolls_to_1_8() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        // INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE not set → exhibition → rolls 1..=8 only
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_500_000; // large diff → many prayers, capped by available_rolls (8)
        let out = step.start(&mut game, &mut GameRng::new(7));
        let seq = &out.pushes[0];
        for s in seq {
            let roll = s.params.iter().find_map(|p| if let StepParameter::PrayerRoll(r) = p { Some(*r) } else { None }).unwrap();
            assert!(roll >= 1 && roll <= 8, "exhibition roll {roll} out of range 1-8");
        }
        assert!(seq.len() <= 8);
    }

    #[test]
    fn league_mode_allows_rolls_up_to_16() {
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        game.options.set("inducementPrayersUseLeagueTable", "true");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_900_000; // large diff → 16 prayers
        let out = step.start(&mut game, &mut GameRng::new(7));
        let seq = &out.pushes[0];
        assert_eq!(seq.len(), 16, "league mode should allow up to 16 unique prayers");
    }

    #[test]
    fn already_held_prayer_is_excluded_from_reroll() {
        // Home team already holds "Iron Man" (roll 4 in the exhibition table). If we force
        // exactly 1 prayer to be drawn, the roll must never be 4 since it's already held —
        // before the fix, StepPrayers picked from a raw 1..=8 range ignoring InducementSet
        // state entirely, so roll 4 could be (re-)selected.
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        game.turn_data_home.inducement_set.add_prayer("Iron Man");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_050_000; // 1 prayer, home is underdog and receives it
        // Try many seeds; roll 4 must never appear since Iron Man is already held by home.
        for seed in 0..50u64 {
            let mut g = game.clone();
            let out = step.start(&mut g, &mut GameRng::new(seed));
            let seq = &out.pushes[0];
            let roll = seq[0].params.iter().find_map(|p| if let StepParameter::PrayerRoll(r) = p { Some(*r) } else { None }).unwrap();
            assert_ne!(roll, 4, "already-held prayer (Iron Man, roll 4) must not be re-drawn");
        }
    }

    #[test]
    fn tv_difference_emits_prayer_amount_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.options.set("inducementPrayersCost", "50000");
        let mut step = StepPrayers::new();
        step.tv_home = 1_000_000;
        step.tv_away = 1_050_000; // 1 prayer
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PRAYER_AMOUNT));
    }

    #[test]
    fn no_tv_difference_does_not_emit_prayer_amount_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPrayers::new();
        // tv_home == tv_away -> 0 prayers -> no report
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PRAYER_AMOUNT));
    }
}
