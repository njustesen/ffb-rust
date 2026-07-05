/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepPrayers` (BB2020).
///
/// Determines how many Prayers to Nuffle the lower-TV team receives, builds a sequence of
/// StepPrayer steps (one per prayer), and pushes them onto the stack.
///
/// DEFERRED items:
///  - PrayerFactory.availablePrayerRolls() not translated → uses fixed rolls 1..=16.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepPrayers`.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
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

            // DEFERRED(prayers): PrayerFactory.availablePrayerRolls() not translated.
            // Use a fixed pool of rolls 1..=16.
            let mut available_rolls: Vec<i32> = (1..=16).collect();

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
}
