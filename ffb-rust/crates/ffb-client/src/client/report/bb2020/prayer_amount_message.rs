use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_prayer_amount::ReportPrayerAmount;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool::format_thousands;

/// 1:1 translation of `PrayerAmountMessage.java`.
pub struct PrayerAmountMessage;

impl ReportMessage for PrayerAmountMessage {
    type Report = ReportPrayerAmount;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_AMOUNT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::BOLD, "Praying to Nuffle");
        status_report.print_indent_style(indent + 1, TextStyle::HOME_BOLD, &game.team_home.name.clone());
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &get_tv_text(report.get_tv_home()));
        status_report.print_indent_style(indent + 1, TextStyle::AWAY_BOLD, &game.team_away.name.clone());
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &get_tv_text(report.get_tv_away()));
        if report.is_home_team_receives_prayers() {
            status_report.print_indent_style(indent + 2, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 2, TextStyle::AWAY, &game.team_away.name.clone());
        }
        let prayers = if report.get_prayer_amount() == 1 { "Prayer" } else { "Prayers" };
        status_report.println_indent_style(
            indent + 2,
            TextStyle::EXPLANATION,
            &format!(" is granted {} {} to Nuffle", report.get_prayer_amount(), prayers),
        );
    }
}

fn get_tv_text(tv: i32) -> String {
    format!(" has a TV of {} after buying inducements.", format_thousands(tv as i64))
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(PrayerAmountMessage.report_id(), ReportId::PRAYER_AMOUNT);
    }

    #[test]
    fn renders_tv_text_for_both_teams() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(1000, 2000, 1, true);
        PrayerAmountMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" has a TV of 1,000 after buying inducements.")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" has a TV of 2,000 after buying inducements.")));
    }

    #[test]
    fn singular_prayer_text_for_amount_one() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(500, 500, 1, true);
        PrayerAmountMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is granted 1 Prayer to Nuffle")));
    }

    #[test]
    fn plural_prayers_text_for_amount_other_than_one() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(500, 500, 3, false);
        PrayerAmountMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is granted 3 Prayers to Nuffle")));
    }

    #[test]
    fn away_team_receives_prayers_when_flag_false() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerAmount::new(500, 500, 1, false);
        PrayerAmountMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text_style == Some(TextStyle::AWAY) && r.text.as_deref() == Some("Team away")));
    }
}
