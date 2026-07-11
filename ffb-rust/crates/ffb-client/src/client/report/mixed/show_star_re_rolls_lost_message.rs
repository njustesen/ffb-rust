use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_show_star_re_rolls_lost::ReportShowStarReRollsLost;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ShowStarReRollsLostMessage.java`.
pub struct ShowStarReRollsLostMessage;

impl ReportMessage for ShowStarReRollsLostMessage {
    type Report = ReportShowStarReRollsLost;

    fn report_id(&self) -> ReportId {
        ReportId::SHOW_STAR_RE_ROLLS_LOST
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let team = report.get_team_id().and_then(|id| game.team_by_id(id));
        let indent = status_report.get_indent() + 1;
        let is_home = team.is_some_and(|t| t.id == game.team_home.id);
        let team_style = if is_home { TextStyle::HOME } else { TextStyle::AWAY };

        if let Some(team) = team {
            status_report.print_indent_style(indent, team_style, &team.name.clone());
        }

        let amount = report.get_amount();
        let mut status = String::from(" lose ");
        if amount == 1 {
            status.push_str("1 Star of the Show Re-Roll as it was");
        } else {
            status.push_str(&format!("{amount} Star of the Show Re-Rolls as they were"));
        }
        status.push_str(" not used in this drive.");

        status_report.println_indent_style(indent, TextStyle::NONE, &status);
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
            coach: format!("Coach {id}"),
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
    fn singular_amount_home_team() {
        let game = make_game();
        let report = ReportShowStarReRollsLost::new(Some("home".into()), 1);
        let mut status_report = StatusReport::new();
        ShowStarReRollsLostMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team home"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME));
        assert_eq!(
            status_report.rendered_runs[1].text.as_deref(),
            Some(" lose 1 Star of the Show Re-Roll as it was not used in this drive.")
        );
    }

    #[test]
    fn plural_amount_away_team() {
        let game = make_game();
        let report = ReportShowStarReRollsLost::new(Some("away".into()), 3);
        let mut status_report = StatusReport::new();
        ShowStarReRollsLostMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team away"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY));
        assert_eq!(
            status_report.rendered_runs[1].text.as_deref(),
            Some(" lose 3 Star of the Show Re-Rolls as they were not used in this drive.")
        );
    }

    #[test]
    fn missing_team_still_reports_amount_line() {
        let game = make_game();
        let report = ReportShowStarReRollsLost::new(None, 2);
        let mut status_report = StatusReport::new();
        ShowStarReRollsLostMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs.len(), 2); // text run + println terminator
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some(" lose 2 Star of the Show Re-Rolls as they were not used in this drive.")
        );
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(ShowStarReRollsLostMessage.report_id(), ReportId::SHOW_STAR_RE_ROLLS_LOST);
        assert_eq!(ShowStarReRollsLostMessage.get_key(), "showStarReRollLost");
    }
}
