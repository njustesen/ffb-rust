use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_cards_and_inducements_bought::ReportCardsAndInducementsBought;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

/// 1:1 translation of `CardsAndInducementsBoughtMessage.java`.
pub struct CardsAndInducementsBoughtMessage;

impl ReportMessage for CardsAndInducementsBoughtMessage {
    type Report = ReportCardsAndInducementsBought;

    fn report_id(&self) -> ReportId {
        ReportId::CARDS_AND_INDUCEMENTS_BOUGHT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if !status_report.inducements_bought_report_received {
            status_report.inducements_bought_report_received = true;
            status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, "Buy Inducements");
        }
        let indent = status_report.get_indent() + 1;
        status_report.print_indent(indent, "Team ");
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }

        let mut status = String::from(" buys ");
        let bought_items = report.get_cards() + report.get_inducements() + report.get_stars() + report.get_mercenaries();
        if bought_items == 0 {
            status.push_str("no Inducements.");
        } else {
            let mut item_list: Vec<String> = Vec::new();
            if report.get_cards() > 0 {
                if report.get_cards() == 1 {
                    item_list.push("1 Card".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Cards", &[&report.get_cards().to_string()]));
                }
            }
            if report.get_inducements() > 0 {
                if report.get_inducements() == 1 {
                    item_list.push("1 Inducement".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Inducements", &[&report.get_inducements().to_string()]));
                }
            }
            if report.get_stars() > 0 {
                if report.get_stars() == 1 {
                    item_list.push("1 Star".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Stars", &[&report.get_stars().to_string()]));
                }
            }
            if report.get_mercenaries() > 0 {
                if report.get_mercenaries() == 1 {
                    item_list.push("1 Mercenary".to_string());
                } else {
                    item_list.push(string_tool::bind("$1 Mercenaries", &[&report.get_mercenaries().to_string()]));
                }
            }
            let item_refs: Vec<&str> = item_list.iter().map(|s| s.as_str()).collect();
            status.push_str(&string_tool::build_enumeration(&item_refs));
            status.push_str(" for ");
            status.push_str(&string_tool::format_thousands(report.get_gold() as i64));
            status.push_str(" gold total increasing their Team Value to ");
            status.push_str(&string_tool::format_thousands(report.get_new_tv() as i64));
        }
        status_report.println_indent(indent, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(
            make_team("home", "Home Team", vec![]),
            make_team("away", "Away Team", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn first_report_prints_buy_inducements_header() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCardsAndInducementsBought::new("home".into(), 0, 0, 0, 0, 0, 1_000_000);
        CardsAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Buy Inducements"));
        assert!(status_report.inducements_bought_report_received);
    }

    #[test]
    fn second_report_does_not_repeat_header() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        status_report.inducements_bought_report_received = true;
        let report = ReportCardsAndInducementsBought::new("away".into(), 1, 0, 0, 0, 100_000, 1_100_000);
        CardsAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.contains(&"Buy Inducements"));
    }

    #[test]
    fn no_items_bought_prints_no_inducements() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCardsAndInducementsBought::new("home".into(), 0, 0, 0, 0, 0, 1_000_000);
        CardsAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("no Inducements.")));
    }

    #[test]
    fn single_card_and_multiple_stars_use_correct_pluralization() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCardsAndInducementsBought::new("home".into(), 1, 0, 2, 0, 100_000, 1_200_000);
        CardsAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("1 Card") && t.contains("2 Stars")));
        assert!(texts.iter().any(|t| t.contains("100,000 gold total increasing their Team Value to 1,200,000")));
    }

    #[test]
    fn away_team_uses_away_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCardsAndInducementsBought::new("away".into(), 0, 1, 0, 1, 50_000, 900_000);
        CardsAndInducementsBoughtMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Team")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("1 Inducement") && t.contains("1 Mercenary")));
    }
}
