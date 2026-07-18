use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::factory::bb2025::prayer_factory::PrayerFactory as Bb2025PrayerFactory;
use ffb_model::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;
use ffb_model::inducement::bb2025::prayer::Prayer;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_prayer_roll::ReportPrayerRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `game.<PrayerFactory>getFactory(FactoryType.Factory.PRAYER).forRoll(roll)`.
/// `ffb_model::factory::bb2025::prayer_factory::PrayerFactory` is now a real translation
/// (no game-option dependency for BB2025, unlike BB2020's league-table branch), so this
/// constructs and initializes it directly rather than duplicating its roll table.
fn for_roll(roll: i32) -> Option<Prayer> {
    let mut factory = Bb2025PrayerFactory::new();
    factory.initialize();
    factory.for_roll(roll)
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
            // java: `prayer.getDuration().getDescription() + ": " + prayer.getDescription()`
            status_report.println_indent_style(
                indent + 2,
                TextStyle::EXPLANATION,
                &format!("{}: {}", prayer.get_duration().get_description(), prayer.get_description()),
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

    #[test]
    fn prayer_description_is_rendered_alongside_duration() {
        // java: `println(getIndent() + 2, TextStyle.EXPLANATION,
        // prayer.getDuration().getDescription() + ": " + prayer.getDescription());` — the
        // prayer's actual rules text must be printed, not just the duration label.
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerRoll::new("Home Ultras".into(), 2, true);
        PrayerRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("Argue the call succeeds on 5+")));
    }
}
