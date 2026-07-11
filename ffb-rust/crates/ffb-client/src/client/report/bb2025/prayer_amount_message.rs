use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_prayer_amount::ReportPrayerAmount;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool::format_thousands;

/// 1:1 translation of `PrayerAmountMessage.java`.
pub struct PrayerAmountMessage;

impl PrayerAmountMessage {
    fn get_tv_text(tv: i32) -> String {
        format!(" has a TV of {} after buying inducements.", format_thousands(tv as i64))
    }
}

impl ReportMessage for PrayerAmountMessage {
    type Report = ReportPrayerAmount;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_AMOUNT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::BOLD, "Praying to Nuffle");
        status_report.print_indent_style(indent + 1, TextStyle::HOME_BOLD, &game.team_home.name.clone());
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &Self::get_tv_text(report.get_tv_home()));
        status_report.print_indent_style(indent + 1, TextStyle::AWAY_BOLD, &game.team_away.name.clone());
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &Self::get_tv_text(report.get_tv_away()));

        if report.is_home_team_receives_prayers() {
            status_report.print_indent_style(indent + 2, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 2, TextStyle::AWAY, &game.team_away.name.clone());
        }
        let prayers = if report.get_prayer_amount() == 1 { "Prayer" } else { "Prayers" };
        status_report.println_indent_style(
            indent + 2,
            TextStyle::EXPLANATION,
            &format!(" is granted {} additional {} to Nuffle", report.get_prayer_amount(), prayers),
        );
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
    fn home_team_receives_prayers_uses_home_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(1_000_000, 900_000, 2, true);
        PrayerAmountMessage.render(&mut status_report, &game, &report);
        let home_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(home_run.text_style, Some(TextStyle::HOME_BOLD));
    }

    #[test]
    fn tv_text_formats_thousands() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(1_000_000, 900_000, 2, true);
        PrayerAmountMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("1,000,000")));
        assert!(texts.iter().any(|t| t.contains("900,000")));
    }

    #[test]
    fn singular_prayer_when_amount_is_one() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(0, 0, 1, false);
        PrayerAmountMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " is granted 1 additional Prayer to Nuffle"));
    }

    #[test]
    fn plural_prayers_when_amount_is_not_one() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(0, 0, 3, false);
        PrayerAmountMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " is granted 3 additional Prayers to Nuffle"));
        let away_run = status_report
            .rendered_runs
            .iter()
            .rev()
            .find(|r| r.text.as_deref() == Some("Team away"))
            .unwrap();
        assert_eq!(away_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn report_id_is_prayer_amount() {
        assert_eq!(PrayerAmountMessage.report_id(), ReportId::PRAYER_AMOUNT);
    }
}
