use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_show_star_re_roll::ReportShowStarReRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ShowStarReRollMessage.java`.
pub struct ShowStarReRollMessage;

impl ReportMessage for ShowStarReRollMessage {
    type Report = ReportShowStarReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SHOW_STAR_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        print_player(status_report, game, indent, false, player);

        status_report.print_indent(indent, " is the Star of the Show and ");
        // java: `player.getTeam() == game.getTeamHome()` — translated via id-based team
        // membership check since `Player` carries no direct team reference here.
        let is_home = player.is_some_and(|p| game.team_home.has_player(&p.id));
        if is_home {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        status_report.println_indent(indent, " gains a Re-Roll only available for this drive.");
        status_report.println_indent_style(indent, TextStyle::EXPLANATION, "Will be added for the next drive.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Star")]);
        let away = make_team("away", vec![make_player("p2", "OtherStar")]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn home_star_player() {
        let game = make_game();
        let report = ReportShowStarReRoll::new(Some("p1".into()));
        let mut status_report = StatusReport::new();
        ShowStarReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Star"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" is the Star of the Show and "));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Team home"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" gains a Re-Roll only available for this drive."));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("Will be added for the next drive."));
        assert_eq!(status_report.rendered_runs[5].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn away_star_player() {
        let game = make_game();
        let report = ReportShowStarReRoll::new(Some("p2".into()));
        let mut status_report = StatusReport::new();
        ShowStarReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Team away"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn missing_player_still_reports_team_line() {
        let game = make_game();
        let report = ReportShowStarReRoll::new(None);
        let mut status_report = StatusReport::new();
        ShowStarReRollMessage.render(&mut status_report, &game, &report);
        // print_player is a no-op for a missing player, but the rest of the render proceeds.
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some(" is the Star of the Show and "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Team away"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(ShowStarReRollMessage.report_id(), ReportId::SHOW_STAR_RE_ROLL);
        assert_eq!(ShowStarReRollMessage.get_key(), "showStarReRoll");
    }
}
