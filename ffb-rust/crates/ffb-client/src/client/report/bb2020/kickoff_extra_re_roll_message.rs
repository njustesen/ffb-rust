use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_kickoff_extra_re_roll::ReportKickoffExtraReRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffExtraReRollMessage.java`.
pub struct KickoffExtraReRollMessage;

impl ReportMessage for KickoffExtraReRollMessage {
    type Report = ReportKickoffExtraReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_EXTRA_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let home_banned = game.turn_data_home.coach_banned;
        let away_banned = game.turn_data_away.coach_banned;

        let home_part = game.turn_data_home.inducement_set.value(Usage::ADD_COACH);
        let away_part = game.turn_data_away.inducement_set.value(Usage::ADD_COACH);

        let mut status = format!("Brilliant Coaching Roll Home Team [ {} ]", report.get_roll_home());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_home =
            report.get_roll_home() + game.team_home.assistant_coaches - if home_banned { 1 } else { 0 } + home_part;
        status = format!("Rolled {}", report.get_roll_home());
        status.push_str(&format!(" + {} Assistant Coaches", game.team_home.assistant_coaches));
        if home_part > 0 {
            status.push_str(&format!(" + {} Part-time Assistant Coaches", home_part));
        }
        status.push(' ');
        status.push_str(if home_banned { "- 1 Banned" } else { " + 0 Head" });
        status.push_str(" Coach");
        status.push_str(&format!(" = {}.", total_home));
        status_report.println_indent(indent + 1, &status);

        status = format!("Brilliant Coaching Roll Away Team [ {} ]", report.get_roll_away());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_away =
            report.get_roll_away() + game.team_away.assistant_coaches - if away_banned { 1 } else { 0 } + away_part;
        status = format!("Rolled {}", report.get_roll_away());
        status.push_str(&format!(" + {} Assistant Coaches", game.team_away.assistant_coaches));
        if away_part > 0 {
            status.push_str(&format!(" + {} Part-time Assistant Coaches", away_part));
        }
        status.push(' ');
        status.push_str(if away_banned { "- 1 Banned" } else { " + 0 Head" });
        status.push_str(" Coach");
        status.push_str(&format!(" = {}.", total_away));
        status_report.println_indent(indent + 1, &status);

        match report.get_team_id() {
            None => {
                status_report.println_indent(indent, "Neither team gains a Re-Roll.");
            }
            Some(team_id) => {
                if team_id == game.team_home.id {
                    status_report.print_indent(indent, "Team ");
                    status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
                } else {
                    status_report.print_indent(indent, "Team ");
                    status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
                }
                status_report.println_indent(indent, " gains a Re-Roll only available for this drive.");
            }
        }
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
            assistant_coaches: 2,
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
        assert_eq!(KickoffExtraReRollMessage.report_id(), ReportId::KICKOFF_EXTRA_RE_ROLL);
    }

    #[test]
    fn no_team_gains_reroll() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffExtraReRoll::new(3, 2, None);
        KickoffExtraReRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Neither team gains a Re-Roll.")));
    }

    #[test]
    fn home_team_gains_reroll() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffExtraReRoll::new(3, 2, Some("home".to_string()));
        KickoffExtraReRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team ")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" gains a Re-Roll only available for this drive.")));
    }

    #[test]
    fn coach_banned_uses_minus_one() {
        let mut sr = StatusReport::new();
        let mut game = make_game();
        game.turn_data_home.coach_banned = true;
        let report = ReportKickoffExtraReRoll::new(3, 2, None);
        KickoffExtraReRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("- 1 Banned Coach"))));
    }
}
