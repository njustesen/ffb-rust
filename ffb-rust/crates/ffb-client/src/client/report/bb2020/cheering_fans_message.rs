use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_cheering_fans::ReportCheeringFans;
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

        if report.get_team_id().is_empty() {
            status_report.println_indent(indent, "Neither team gains a Prayer to Nuffle.");
        } else {
            if report.get_team_id() == game.team_home.id {
                status_report.print_indent(indent, "Team ");
                status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
            } else {
                status_report.print_indent(indent, "Team ");
                status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
            }
            if report.is_prayer_available() {
                status_report.println_indent(indent, " gains a Prayer to Nuffle.");
            } else {
                status_report.println_indent(indent, " would gain a Prayer to Nuffle but all are in effect.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str, cheerleaders: i32) -> Team {
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
            cheerleaders,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: Vec::<Player>::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(
            make_team("home", "Home Team", 2),
            make_team("away", "Away Team", 1),
            Rules::Bb2020,
        )
    }

    #[test]
    fn no_team_gains_prayer_when_team_id_empty() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCheeringFans::new(String::new(), false, 4, 3);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Neither team gains a Prayer to Nuffle."));
    }

    #[test]
    fn home_team_gains_prayer_when_available() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCheeringFans::new("home".into(), true, 4, 3);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Home Team"));
        assert!(texts.iter().any(|t| t.contains(" gains a Prayer to Nuffle.")));
    }

    #[test]
    fn away_team_prayer_not_available_when_all_in_effect() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCheeringFans::new("away".into(), false, 4, 3);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Away Team"));
        assert!(texts.iter().any(|t| t.contains(" would gain a Prayer to Nuffle but all are in effect.")));
    }

    #[test]
    fn totals_include_cheerleaders_from_team() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportCheeringFans::new("home".into(), true, 4, 3);
        CheeringFansMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        // roll 4 + 2 cheerleaders (home) = 6
        assert!(texts.iter().any(|t| t.contains("Rolled 4 + 2 Cheerleaders = 6.")));
        // roll 3 + 1 cheerleader (away) = 4
        assert!(texts.iter().any(|t| t.contains("Rolled 3 + 1 Cheerleaders = 4.")));
    }
}
