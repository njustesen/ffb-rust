use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_inducements_bought::ReportInducementsBought;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

pub struct InducementsBoughtMessage;

impl ReportMessage for InducementsBoughtMessage {
    type Report = ReportInducementsBought;

    fn report_id(&self) -> ReportId {
        ReportId::INDUCEMENTS_BOUGHT
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
        if report.get_nr_of_inducements() == 0 && report.get_nr_of_stars() == 0 && report.get_nr_of_mercenaries() == 0 {
            status.push_str("no Inducements.");
        } else {
            let mut item_list: Vec<String> = Vec::new();
            if report.get_nr_of_inducements() > 0 {
                if report.get_nr_of_inducements() == 1 {
                    item_list.push("1 Inducement".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Inducements", &[&report.get_nr_of_inducements().to_string()]));
                }
            }
            if report.get_nr_of_stars() > 0 {
                if report.get_nr_of_stars() == 1 {
                    item_list.push("1 Star".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Stars", &[&report.get_nr_of_stars().to_string()]));
                }
            }
            if report.get_nr_of_mercenaries() > 0 {
                if report.get_nr_of_mercenaries() == 1 {
                    item_list.push("1 Mercenary".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Mercenaries", &[&report.get_nr_of_mercenaries().to_string()]));
                }
            }
            let refs: Vec<&str> = item_list.iter().map(|s| s.as_str()).collect();
            status.push_str(&string_tool::build_enumeration(&refs));
            status.push_str(&format!(" for {} gold total.", string_tool::format_thousands(report.get_gold() as i64)));
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
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_inducements_bought() {
        assert_eq!(InducementsBoughtMessage.get_key(), "inducementsBought");
    }

    #[test]
    fn no_inducements_bought() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducementsBought::new("home".into(), 0, 0, 0, 0);
        InducementsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" buys no Inducements.")));
    }

    #[test]
    fn enumerates_multiple_items() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducementsBought::new("away".into(), 2, 1, 0, 50000);
        InducementsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" buys 2 Inducements and 1 Star for 50,000 gold total.")));
    }

    #[test]
    fn second_call_skips_header() {
        let mut status_report = StatusReport::new();
        status_report.inducements_bought_report_received = true;
        let game = make_game();
        let report = ReportInducementsBought::new("home".into(), 0, 0, 0, 0);
        InducementsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(!status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Buy Inducements")));
    }
}
