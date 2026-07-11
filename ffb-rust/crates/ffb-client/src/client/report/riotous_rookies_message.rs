use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_riotous_rookies::ReportRiotousRookies;

/// 1:1 translation of `RiotousRookiesMessage.java`.
pub struct RiotousRookiesMessage;

impl ReportMessage for RiotousRookiesMessage {
    type Report = ReportRiotousRookies;

    fn report_id(&self) -> ReportId {
        ReportId::RIOTOUS_ROOKIES
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let roll = report.get_roll();
        let text = format!(
            "Riotous Rookies Roll [ {} ][ {} ] + 1",
            roll.first().copied().unwrap_or(0),
            roll.get(1).copied().unwrap_or(0)
        );
        status_report.println_indent_style(0, TextStyle::ROLL, &text);
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(1, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(1, TextStyle::AWAY, &game.team_away.name.clone());
        }
        let text = format!(" hires {} Riotous Rookies for this game", report.get_amount());
        status_report.println_indent_style(1, TextStyle::NONE, &text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: format!("{id}-name"), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(RiotousRookiesMessage.report_id(), ReportId::RIOTOUS_ROOKIES);
    }

    #[test]
    fn render_home_team_prints_home_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRiotousRookies::new(vec![2, 3], 1, "home".into());
        RiotousRookiesMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Riotous Rookies Roll [ 2 ][ 3 ] + 1"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("home-name"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" hires 1 Riotous Rookies for this game"));
    }

    #[test]
    fn render_away_team_prints_away_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRiotousRookies::new(vec![5, 6], 2, "away".into());
        RiotousRookiesMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("away-name"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" hires 2 Riotous Rookies for this game"));
    }

    #[test]
    fn render_missing_roll_entries_default_to_zero() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRiotousRookies::new(vec![], 0, "home".into());
        RiotousRookiesMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Riotous Rookies Roll [ 0 ][ 0 ] + 1"));
    }
}
