use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::inducement::inducement::Inducement;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_cheering_fans::ReportCheeringFans;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CheeringFansMessage.java`.
pub struct CheeringFansMessage;

impl ReportMessage for CheeringFansMessage {
    type Report = ReportCheeringFans;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_CHEERING_FANS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let home_temps = game.turn_data_home.inducement_set.value(Usage::ADD_CHEERLEADER);
        let away_temps = game.turn_data_away.inducement_set.value(Usage::ADD_CHEERLEADER);

        let indent = status_report.get_indent();

        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Cheering Fans Roll Home Team [ {} ]", report.get_roll_home()),
        );
        let total_home = report.get_roll_home() + game.team_home.cheerleaders + home_temps;
        let mut status = format!("Rolled {}", report.get_roll_home());
        status.push_str(&format!(" + {} Cheerleaders", game.team_home.cheerleaders));
        if home_temps > 0 {
            status.push_str(&format!(" + {home_temps} Temp Agency Cheerleaders"));
        }
        status.push_str(&format!(" = {total_home}."));
        status_report.println_indent(indent + 1, &status);
        if report.get_rerolled().contains(&game.team_home.id) {
            print_team_name(status_report, game, false, &game.team_home.id.clone());
            status_report.println_indent(indent + 1, " rerolled a natural 1 using their Team Mascot");
        }
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Cheering Fans Roll Away Team [ {} ]", report.get_roll_away()),
        );
        let total_away = report.get_roll_away() + game.team_away.cheerleaders + away_temps;
        let mut status = format!("Rolled {}", report.get_roll_away());
        status.push_str(&format!(" + {} Cheerleaders", game.team_away.cheerleaders));
        if away_temps > 0 {
            status.push_str(&format!(" + {away_temps} Temp Agency Cheerleaders"));
        }
        status.push_str(&format!(" = {total_away}."));
        status_report.println_indent(indent + 1, &status);
        if report.get_rerolled().contains(&game.team_away.id) {
            print_team_name(status_report, game, false, &game.team_away.id.clone());
            status_report.println_indent(indent + 1, " rerolled a natural 1 using their Team Mascot");
        }

        for team_id in report.get_team_ids() {
            if *team_id == game.team_home.id {
                status_report.print_indent(indent, "Team ");
                status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
            } else {
                status_report.print_indent(indent, "Team ");
                status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
            }
            status_report.println_indent(indent, " gain an additional offensive assist for the first block action in their next turn.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, cheerleaders: i32) -> Team {
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
            cheerleaders,
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
        Game::new(make_team("home", 2), make_team("away", 1), Rules::Bb2025)
    }

    #[test]
    fn basic_rolls_and_totals() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCheeringFans::new(vec![], 4, 2, vec![]);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Cheering Fans Roll Home Team [ 4 ]"));
        assert!(texts.iter().any(|t| t.contains("Rolled 4 + 2 Cheerleaders = 6.")));
        assert!(texts.iter().any(|t| t.contains("Rolled 2 + 1 Cheerleaders = 3.")));
    }

    #[test]
    fn temp_agency_cheerleaders_included() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.turn_data_home.inducement_set.add_inducement(Inducement::new("TEMP_CHEERLEADER", 3, vec![Usage::ADD_CHEERLEADER]));
        let report = ReportCheeringFans::new(vec![], 1, 1, vec![]);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("Temp Agency Cheerleaders")));
    }

    #[test]
    fn rerolled_prints_mascot_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCheeringFans::new(vec![], 1, 1, vec!["home".into()]);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("rerolled a natural 1 using their Team Mascot")));
    }

    #[test]
    fn team_ids_gain_additional_assist() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCheeringFans::new(vec!["home".into(), "away".into()], 3, 3, vec![]);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(texts.iter().filter(|t| t.contains("gain an additional offensive assist")).count(), 2);
    }

    #[test]
    fn report_id_is_kickoff_cheering_fans() {
        assert_eq!(CheeringFansMessage.report_id(), ReportId::KICKOFF_CHEERING_FANS);
    }
}
