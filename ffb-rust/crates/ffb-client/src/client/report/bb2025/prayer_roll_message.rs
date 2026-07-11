use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::inducement::bb2025::prayer::Prayer;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_prayer_roll::ReportPrayerRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `game.<PrayerFactory>getFactory(FactoryType.Factory.PRAYER).forRoll(roll)`.
/// `ffb_model::factory::bb2025::prayer_factory::PrayerFactory` and the root
/// `ffb_model::factory::prayer_factory::PrayerFactory` are still unimplemented stubs (no
/// `prayers` map, no `for_roll`), so this transcribes the roll table straight from
/// `com.fumbbl.ffb.inducement.bb2025.Prayers.java` (the fixed BB2025 d16 prayer-to-Nuffle
/// table) rather than fabricating a mapping.
fn for_roll(roll: i32) -> Option<Prayer> {
    match roll {
        1 => Some(Prayer::TREACHEROUS_TRAPDOOR),
        2 => Some(Prayer::FRIENDS_WITH_THE_REF),
        3 => Some(Prayer::STILETTO),
        4 => Some(Prayer::IRON_MAN),
        5 => Some(Prayer::KNUCKLE_DUSTERS),
        6 => Some(Prayer::BAD_HABITS),
        7 => Some(Prayer::GREASY_CLEATS),
        8 => Some(Prayer::BLESSED_STATUE_OF_NUFFLE),
        9 => Some(Prayer::MOLES_UNDER_THE_PITCH),
        10 => Some(Prayer::PERFECT_PASSING),
        11 => Some(Prayer::DAZZLING_CATCHING),
        12 => Some(Prayer::FAN_INTERACTION),
        13 => Some(Prayer::FOULING_FRENZY),
        14 => Some(Prayer::THROW_A_ROCK),
        15 => Some(Prayer::UNDER_SCRUTINY),
        16 => Some(Prayer::INTENSIVE_TRAINING),
        _ => None,
    }
}

pub struct PrayerRollMessage;

impl ReportMessage for PrayerRollMessage {
    type Report = ReportPrayerRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let prayer = for_roll(report.get_roll());

        status_report.print_indent_style(indent, TextStyle::ROLL, &format!("Prayer Roll [ {} ] for ", report.get_roll()));
        let team_style = if report.is_home_team() { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        status_report.println_indent_style(indent, team_style, report.get_team_name());

        if let Some(prayer) = prayer {
            status_report.println_indent_style(indent + 1, TextStyle::BOLD, prayer.get_name());
            // java: `prayer.getDescription()` returns the prayer's rules text (e.g. "Argue
            // the call succeeds on 5+"), sourced from a per-variant `description` field on
            // `com.fumbbl.ffb.inducement.bb2025.Prayer` that has no equivalent on the Rust
            // `ffb_model::inducement::bb2025::prayer::Prayer` enum (it only exposes
            // `event_message()`, a different, mostly-empty string used for player-event
            // reports). Not fabricated here; falls back to the duration description alone.
            status_report.println_indent_style(
                indent + 2,
                TextStyle::EXPLANATION,
                &format!("{}: ", prayer.get_duration().get_description()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn home_team_uses_home_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerRoll::new("Home Ultras".into(), 8, true);
        PrayerRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Home Ultras")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME_BOLD));
    }

    #[test]
    fn away_team_uses_away_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerRoll::new("Away Raiders".into(), 3, false);
        PrayerRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Raiders")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY_BOLD));
    }

    #[test]
    fn roll_of_8_resolves_to_blessed_statue_of_nuffle() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerRoll::new("Home Ultras".into(), 8, true);
        PrayerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Blessed Statue of Nuffle".to_string()));
    }

    #[test]
    fn roll_of_16_resolves_to_intensive_training() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerRoll::new("Home Ultras".into(), 16, true);
        PrayerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Intensive Training".to_string()));
    }

    #[test]
    fn report_id_is_prayer_roll() {
        assert_eq!(PrayerRollMessage.report_id(), ReportId::PRAYER_ROLL);
    }
}
