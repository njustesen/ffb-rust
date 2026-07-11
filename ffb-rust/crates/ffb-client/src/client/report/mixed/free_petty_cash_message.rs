use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_free_petty_cash::ReportFreePettyCash;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `FreePettyCashMessage.java`.
pub struct FreePettyCashMessage;

// java: StringTool.formatThousands not yet ported to ffb-model — mirrors
// `com.fumbbl.ffb.util.StringTool.formatThousands(long)` exactly (inserts a comma every 3
// digits from the right; leftover digits < 3 are emitted first, unseparated).
fn format_thousands(number: i64) -> String {
    let number_string = number.to_string();
    let len = number_string.len();
    let mut result = String::new();
    let mut pos = 0;
    let remainder = len % 3;
    if remainder > 0 {
        result.push_str(&number_string[0..remainder]);
        pos += remainder;
    }
    while pos < len {
        if pos > 0 {
            result.push(',');
        }
        result.push_str(&number_string[pos..pos + 3]);
        pos += 3;
    }
    result
}

impl ReportMessage for FreePettyCashMessage {
    type Report = ReportFreePettyCash;

    fn report_id(&self) -> ReportId {
        ReportId::FREE_PETTY_CASH
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        status_report.println_indent_style(indent, TextStyle::BOLD, "Assigning Petty Cash");

        status_report.print_indent(indent + 1, "Team ");
        if Some(game.team_home.id.as_str()) == report.get_team_id() {
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        }

        let status = format!(
            " receives {} gold as petty cash from being the underdog before adding inducements.",
            format_thousands(report.get_gold() as i64)
        );
        status_report.println_indent(indent + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
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
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn format_thousands_matches_java_examples() {
        assert_eq!(format_thousands(2130000), "2,130,000");
        assert_eq!(format_thousands(50000), "50,000");
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(100), "100");
    }

    #[test]
    fn renders_home_team_petty_cash() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFreePettyCash::new(Some("home".into()), 50000);
        FreePettyCashMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Assigning Petty Cash"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Team "));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team home"));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::HOME));
        assert_eq!(
            sr.rendered_runs[4].text.as_deref(),
            Some(" receives 50,000 gold as petty cash from being the underdog before adding inducements.")
        );
    }

    #[test]
    fn renders_away_team_petty_cash() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFreePettyCash::new(Some("away".into()), 120000);
        FreePettyCashMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team away"));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::AWAY));
        assert!(sr.rendered_runs[4].text.as_deref().unwrap().contains("120,000"));
    }

    #[test]
    fn falls_back_to_away_when_team_id_missing() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFreePettyCash::new(None, 1000);
        FreePettyCashMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team away"));
    }
}
