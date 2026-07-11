use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_prayers_and_inducements_bought::ReportPrayersAndInducementsBought;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool::{bind, build_enumeration, format_thousands};

/// 1:1 translation of `PrayersAndInducementsBoughtMessage.java`.
pub struct PrayersAndInducementsBoughtMessage;

impl ReportMessage for PrayersAndInducementsBoughtMessage {
    type Report = ReportPrayersAndInducementsBought;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        if !status_report.inducements_bought_report_received {
            status_report.inducements_bought_report_received = true;
            status_report.println_indent_style(indent, TextStyle::BOLD, "Buy Inducements");
        }
        status_report.print_indent(indent + 1, "Team ");
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        }

        let mut status = String::from(" buys ");
        let bought_items = report.get_inducements() + report.get_stars() + report.get_mercenaries();
        if bought_items == 0 {
            status.push_str("no Inducements.");
        } else {
            let mut item_list: Vec<String> = Vec::new();
            if report.get_inducements() > 0 {
                if report.get_inducements() == 1 {
                    item_list.push("1 Inducement".to_string());
                } else {
                    item_list.push(bind("$1 Inducements", &[&report.get_inducements().to_string()]));
                }
            }
            if report.get_stars() > 0 {
                if report.get_stars() == 1 {
                    item_list.push("1 Star".to_string());
                } else {
                    item_list.push(bind("$1 Stars", &[&report.get_stars().to_string()]));
                }
            }
            if report.get_mercenaries() > 0 {
                if report.get_mercenaries() == 1 {
                    item_list.push("1 Mercenary".to_string());
                } else {
                    item_list.push(bind("$1 Mercenaries", &[&report.get_mercenaries().to_string()]));
                }
            }
            let item_refs: Vec<&str> = item_list.iter().map(String::as_str).collect();
            status.push_str(&build_enumeration(&item_refs));
            status.push_str(&format!(
                " for {} gold total increasing their Team Value to {}",
                format_thousands(report.get_gold() as i64),
                format_thousands(report.get_new_tv() as i64)
            ));
        }
        status_report.println_indent(indent + 1, &status);
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
    fn first_call_prints_buy_inducements_header_once() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayersAndInducementsBought::new("home".into(), 0, 0, 0, 0, 0);
        PrayersAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&"Buy Inducements".to_string()));
        assert!(status_report.inducements_bought_report_received);
    }

    #[test]
    fn second_call_does_not_reprint_header() {
        let mut status_report = StatusReport::new();
        status_report.inducements_bought_report_received = true;
        let game = make_game();
        let report = ReportPrayersAndInducementsBought::new("home".into(), 0, 0, 0, 0, 0);
        PrayersAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.contains(&"Buy Inducements".to_string()));
    }

    #[test]
    fn no_items_bought_prints_no_inducements() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayersAndInducementsBought::new("home".into(), 0, 0, 0, 0, 0);
        PrayersAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " buys no Inducements."));
    }

    #[test]
    fn multiple_items_use_enumeration_and_gold_formatting() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayersAndInducementsBought::new("away".into(), 2, 1, 0, 150000, 1100000);
        PrayersAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("2 Inducements and 1 Star")));
        assert!(texts.iter().any(|t| t.contains("150,000 gold")));
        assert!(texts.iter().any(|t| t.contains("1,100,000")));
    }

    #[test]
    fn single_item_singular_wording() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayersAndInducementsBought::new("home".into(), 1, 0, 1, 50000, 900000);
        PrayersAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("1 Inducement and 1 Mercenary")));
    }

    #[test]
    fn report_id_is_prayers_and_inducements_bought() {
        assert_eq!(PrayersAndInducementsBoughtMessage.report_id(), ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT);
    }
}
