use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_cards_bought::ReportCardsBought;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

pub struct CardsBoughtMessage;

impl ReportMessage for CardsBoughtMessage {
    type Report = ReportCardsBought;

    fn report_id(&self) -> ReportId {
        ReportId::CARDS_BOUGHT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        if !status_report.cards_bought_report_received {
            status_report.cards_bought_report_received = true;
            status_report.println_indent_style(indent, TextStyle::BOLD, "Buy Cards");
        }
        status_report.print_indent(indent + 1, "Team ");
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        }
        let mut status = String::from(" buys ");
        if report.get_nr_of_cards() == 0 {
            status.push_str("no Cards.");
        } else {
            if report.get_nr_of_cards() == 1 {
                status.push_str("1 Card");
            } else {
                status.push_str(&format!("{} Cards", report.get_nr_of_cards()));
            }
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
    fn get_key_is_cards_bought() {
        assert_eq!(CardsBoughtMessage.get_key(), "cardsBought");
    }

    #[test]
    fn first_report_prints_header_and_sets_flag() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCardsBought::new("home".into(), 2, 20000);
        CardsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(status_report.cards_bought_report_received);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Buy Cards"));
    }

    #[test]
    fn second_report_skips_header() {
        let mut status_report = StatusReport::new();
        status_report.cards_bought_report_received = true;
        let game = make_game();
        let report = ReportCardsBought::new("away".into(), 0, 0);
        CardsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(!status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Buy Cards")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" buys no Cards.")));
    }

    #[test]
    fn single_card_uses_singular() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCardsBought::new("home".into(), 1, 10000);
        CardsBoughtMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" buys 1 Card for 10,000 gold total.")));
    }
}
