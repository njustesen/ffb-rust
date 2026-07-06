/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.start.StepPrayers` (BB2025).
///
/// Computes how many Prayers to Nuffle each team receives, shuffles the available prayer
/// roll table, picks one entry per prayer without replacement, and pushes a `StepKind::Prayer`
/// step for each picked roll onto the stack.
///
/// Init params (via `set_parameter`): TV_HOME, TV_AWAY, PRAYERS_BOUGHT_HOME, PRAYERS_BOUGHT_AWAY.
///
/// Game options consumed:
/// - `"inducement_prayers_cost"` (int, default 50000) — TV gap per additional prayer.
/// - `"inducement_prayers_available_for_underdog"` (bool, default true) — whether the
///   TV-underdog team receives free prayers.
///
/// Prayer roll table: rolls 1–16 (BB2025 has exactly 16 prayer entries).
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_prayer_amount::ReportPrayerAmount;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome, StepParameter, SequenceStep};

/// Default TV cost per free additional prayer (Java `GameOptionId.INDUCEMENT_PRAYERS_COST`).
const DEFAULT_PRAYERS_COST: i32 = 50_000;

/// BB2025 has 16 entries in its prayer table (rolls 1–16).
const ALL_PRAYER_ROLLS: [i32; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.start.StepPrayers`.
pub struct StepPrayers {
    /// Java: tvHome — set via `StepParameter::TvHome`.
    pub tv_home: i32,
    /// Java: tvAway — set via `StepParameter::TvAway`.
    pub tv_away: i32,
    /// Java: prayersBoughtHome — set via `StepParameter::PrayersBoughtHome`.
    pub prayers_bought_home: i32,
    /// Java: prayersBoughtAway — set via `StepParameter::PrayersBoughtAway`.
    pub prayers_bought_away: i32,
}

impl StepPrayers {
    pub fn new() -> Self {
        Self { tv_home: 0, tv_away: 0, prayers_bought_home: 0, prayers_bought_away: 0 }
    }

    // ── Java start() ───────────────────────────────────────────────────────────

    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: additionalPrayerAmount = |tvAway - tvHome| / INDUCEMENT_PRAYERS_COST
        let prayers_cost = game.options
            .get_int("inducement_prayers_cost")
            .unwrap_or(DEFAULT_PRAYERS_COST);
        let cost = if prayers_cost > 0 { prayers_cost } else { DEFAULT_PRAYERS_COST };
        let additional_prayer_amount_raw = (self.tv_away - self.tv_home).abs() / cost;

        // Java: addPrayersToUnderdog = game option INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG (default true)
        let add_prayers_to_underdog = game.options
            .get("inducement_prayers_available_for_underdog")
            .map(|v| !matches!(v, "false" | "0" | "no"))
            .unwrap_or(true);

        // Java: homeTeamAdditionalReceivesPrayers = tvHome < tvAway
        let home_team_additional_receives_prayers = self.tv_home < self.tv_away;

        let mut prayers_total_home = self.prayers_bought_home;
        let mut prayers_total_away = self.prayers_bought_away;

        // Available prayer rolls = all rolls not already bought by either team.
        // Java: prayerFactory.allPrayerRolls() — full set 1..=16 for BB2025.
        let available_prayer_rolls: Vec<i32> = ALL_PRAYER_ROLLS.to_vec();

        let mut additional_prayer_amount = additional_prayer_amount_raw;
        let mut prayer_amount_event: Option<GameEvent> = None;

        if add_prayers_to_underdog && additional_prayer_amount > 0 {
            // Java: alreadyBoughtPrayers = (homeTeamAdditionalReceivesPrayers ? prayersBoughtHome : prayersBoughtAway)
            let already_bought_prayers = if home_team_additional_receives_prayers {
                self.prayers_bought_home
            } else {
                self.prayers_bought_away
            };
            // Java: additionalPrayerAmount = Math.min(additionalPrayerAmount, availablePrayerRolls.size() - alreadyBoughtPrayers)
            additional_prayer_amount = additional_prayer_amount
                .min((available_prayer_rolls.len() as i32) - already_bought_prayers);

            // Java: getResult().addReport(new ReportPrayerAmount(...))
            game.report_list.add(ReportPrayerAmount::new(
                self.tv_home,
                self.tv_away,
                additional_prayer_amount,
                home_team_additional_receives_prayers,
            ));
            prayer_amount_event = Some(GameEvent::PrayerAmount {
                tv_home: self.tv_home,
                tv_away: self.tv_away,
                prayer_amount: additional_prayer_amount,
                home_team_receives_prayers: home_team_additional_receives_prayers,
            });

            if home_team_additional_receives_prayers {
                prayers_total_home += additional_prayer_amount;
            } else {
                prayers_total_away += additional_prayer_amount;
            }
        }

        // Java: if (prayersTotalAway + prayersTotalHome > 0) { push Sequence }
        if prayers_total_home + prayers_total_away > 0 {
            let home_id = game.team_home.id.clone();
            let away_id = game.team_away.id.clone();

            let mut seq: Vec<SequenceStep> = Vec::new();

            // Java: addPrayerSequences(sequence, game.getTeamHome(), prayersTotalHome, new ArrayList<>(availablePrayerRolls))
            add_prayer_sequence_entries(
                &mut seq,
                &home_id,
                prayers_total_home,
                available_prayer_rolls.clone(),
                rng,
            );
            // Java: addPrayerSequences(sequence, game.getTeamAway(), prayersTotalAway, new ArrayList<>(availablePrayerRolls))
            add_prayer_sequence_entries(
                &mut seq,
                &away_id,
                prayers_total_away,
                available_prayer_rolls.clone(),
                rng,
            );

            let out = StepOutcome::next().push_seq(seq);
            return if let Some(ev) = prayer_amount_event { out.with_event(ev) } else { out };
        }

        if let Some(ev) = prayer_amount_event {
            return StepOutcome::next().with_event(ev);
        }

        StepOutcome::next()
    }
}

/// Java `StepPrayers.addPrayerSequences(Sequence, Team, int, List<Integer>)`.
///
/// For each prayer in `prayer_amount`:
/// 1. Shuffle `available_rolls` (Java `Collections.shuffle`).
/// 2. Remove the first element (Java `availablePrayerRolls.remove(0)`).
/// 3. Push a `Prayer { roll, team_id }` step entry.
fn add_prayer_sequence_entries(
    seq: &mut Vec<SequenceStep>,
    team_id: &str,
    prayer_amount: i32,
    mut available_rolls: Vec<i32>,
    rng: &mut GameRng,
) {
    for _ in 0..prayer_amount {
        if available_rolls.is_empty() { break; }
        // Java: Collections.shuffle(availablePrayerRolls) — use GameRng to shuffle.
        fisher_yates_shuffle(&mut available_rolls, rng);
        // Java: int roll = availablePrayerRolls.remove(0)
        let roll = available_rolls.remove(0);
        // Java: sequence.add(StepId.PRAYER, StepParameter.from(PRAYER_ROLL, roll), StepParameter.from(TEAM_ID, team.getId()))
        seq.push(SequenceStep::with_params(StepId::Prayer, vec![
            StepParameter::PrayerRoll(roll),
            StepParameter::TeamId(team_id.to_owned()),
        ]));
    }
}

/// Fisher-Yates shuffle using `GameRng::die` for index selection.
/// Mirrors Java `Collections.shuffle` driven by the game RNG.
fn fisher_yates_shuffle(v: &mut Vec<i32>, rng: &mut GameRng) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = rng.die((i + 1) as u32) as usize - 1;
        v.swap(i, j);
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
            // Java: case TV_HOME → tvHome = (int) parameter.getValue()
            StepParameter::TvHome(v) => { self.tv_home = *v; true }
            // Java: case TV_AWAY → tvAway = (int) parameter.getValue()
            StepParameter::TvAway(v) => { self.tv_away = *v; true }
            // Java: case PRAYERS_BOUGHT_HOME → prayersBoughtHome = (int) parameter.getValue()
            StepParameter::PrayersBoughtHome(v) => { self.prayers_bought_home = *v; true }
            // Java: case PRAYERS_BOUGHT_AWAY → prayersBoughtAway = (int) parameter.getValue()
            StepParameter::PrayersBoughtAway(v) => { self.prayers_bought_away = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;

    fn prayer_roll_from_entry(entry: &SequenceStep) -> i32 {
        entry.params.iter().find_map(|p| if let StepParameter::PrayerRoll(r) = p { Some(*r) } else { None })
            .expect("PrayerRoll param missing")
    }

    fn prayer_team_from_entry(entry: &SequenceStep) -> &str {
        entry.params.iter().find_map(|p| if let StepParameter::TeamId(t) = p { Some(t.as_str()) } else { None })
            .expect("TeamId param missing")
    }

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn equal_tv_no_prayers_returns_next_step() {
        let mut game = make_game();
        // Both teams 1_000_000 TV, no prayers bought.
        let mut step = StepPrayers { tv_home: 1_000_000, tv_away: 1_000_000, ..StepPrayers::new() };
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty(), "no prayers should push any sequence");
    }

    #[test]
    fn tv_gap_awards_prayers_to_underdog() {
        let mut game = make_game();
        // Home TV 800k, away TV 1000k → gap 200k. Cost 50k → 4 additional.
        // Home is underdog (tvHome < tvAway).
        let mut step = StepPrayers {
            tv_home: 800_000,
            tv_away: 1_000_000,
            prayers_bought_home: 0,
            prayers_bought_away: 0,
        };
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
        // Should push exactly one sequence containing 4 Prayer entries.
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].len(), 4);
        // All pushed steps must be Prayer with the home team id.
        for entry in &out.pushes[0] {
            assert_eq!(entry.step_id, StepId::Prayer);
            assert_eq!(prayer_team_from_entry(entry), game.team_home.id.as_str());
        }
    }

    #[test]
    fn set_parameter_tv_home_accepted() {
        let mut step = StepPrayers::new();
        assert!(step.set_parameter(&StepParameter::TvHome(900_000)));
        assert_eq!(step.tv_home, 900_000);
    }

    #[test]
    fn set_parameter_prayers_bought_away_accepted() {
        let mut step = StepPrayers::new();
        assert!(step.set_parameter(&StepParameter::PrayersBoughtAway(2)));
        assert_eq!(step.prayers_bought_away, 2);
    }

    #[test]
    fn prayers_capped_by_available_rolls() {
        let mut game = make_game();
        // Massive TV gap would give more than 16 prayers, but pool only has 16 rolls.
        // Home already bought 15 → cap = 16 - 15 = 1 additional.
        let mut step = StepPrayers {
            tv_home: 0,
            tv_away: 10_000_000, // huge gap → 200 prayers before cap
            prayers_bought_home: 15,
            prayers_bought_away: 0,
        };
        let out = step.start(&mut game, &mut GameRng::new(7));
        // prayers_total_home = 15 + 1 = 16; prayers_total_away = 0.
        // Sequence has 16 entries (15 bought + 1 extra).
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].len(), 16);
    }

    #[test]
    fn prayer_rolls_are_unique_within_sequence() {
        let mut game = make_game();
        // 4 prayers awarded; rolls must be distinct (each is removed from the pool).
        let mut step = StepPrayers {
            tv_home: 800_000,
            tv_away: 1_000_000,
            prayers_bought_home: 0,
            prayers_bought_away: 0,
        };
        let out = step.start(&mut game, &mut GameRng::new(42));
        let rolls: Vec<i32> = out.pushes[0].iter().map(prayer_roll_from_entry).collect();
        let unique: std::collections::HashSet<i32> = rolls.iter().cloned().collect();
        assert_eq!(rolls.len(), unique.len(), "prayer rolls must be distinct");
    }

    #[test]
    fn tv_gap_adds_prayer_amount_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPrayers {
            tv_home: 800_000,
            tv_away: 1_000_000,
            prayers_bought_home: 0,
            prayers_bought_away: 0,
        };
        step.start(&mut game, &mut GameRng::new(1));
        assert!(game.report_list.has_report(ReportId::PRAYER_AMOUNT),
            "PRAYER_AMOUNT report must be added when TV gap exists");
    }

    #[test]
    fn equal_tv_no_prayer_amount_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPrayers {
            tv_home: 1_000_000,
            tv_away: 1_000_000,
            prayers_bought_home: 0,
            prayers_bought_away: 0,
        };
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PRAYER_AMOUNT),
            "PRAYER_AMOUNT must NOT be added when TV is equal");
    }
}
