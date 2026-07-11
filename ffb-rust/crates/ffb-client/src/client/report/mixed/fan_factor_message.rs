use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_fan_factor::ReportFanFactor;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `FanFactorMessage.java`.
pub struct FanFactorMessage;

impl ReportMessage for FanFactorMessage {
    type Report = ReportFanFactor;

    fn report_id(&self) -> ReportId {
        ReportId::FAN_FACTOR
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Fan Factor Roll [{}]", report.get_roll()));

        status_report.print_indent(indent + 1, "Team ");
        if Some(game.team_home.id.as_str()) == report.get_team_id() {
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        }

        let status = format!(
            " has {}k fans behind them ({}k Dedicated Fans and {}k fair-weather fans)",
            report.get_result(),
            report.get_dedicated_fans(),
            report.get_roll()
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
    fn renders_home_team_fan_factor() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactor::new(3, 2, Some("home".into()));
        FanFactorMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Fan Factor Roll [3]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Team "));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team home"));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::HOME));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some(" has 5k fans behind them (2k Dedicated Fans and 3k fair-weather fans)"));
    }

    #[test]
    fn renders_away_team_fan_factor() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactor::new(1, 0, Some("away".into()));
        FanFactorMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team away"));
        assert_eq!(sr.rendered_runs[3].text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn falls_back_to_away_when_team_id_missing() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactor::new(4, 1, None);
        FanFactorMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Team away"));
    }
}
