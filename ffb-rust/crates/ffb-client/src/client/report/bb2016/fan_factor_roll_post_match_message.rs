use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_fan_factor_roll_post_match::ReportFanFactorRollPostMatch;
use ffb_model::report::report_id::ReportId;

pub struct FanFactorRollPostMatchMessage;

impl ReportMessage for FanFactorRollPostMatchMessage {
    type Report = ReportFanFactorRollPostMatch;

    fn report_id(&self) -> ReportId {
        ReportId::FAN_FACTOR_ROLL_POST_MATCH
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        let mut status = String::new();
        let home_rolls = report.get_fan_factor_roll_home();
        if !home_rolls.is_empty() {
            status.push_str("Fan Factor Roll Home Team ");
            for roll in home_rolls {
                status.push_str(&format!("[ {} ]", roll));
            }
        } else {
            status.push_str("Fan Factor: Concession of Home Team");
        }
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let fan_factor_home = game.team_home.fan_factor;
        let modifier_home = report.get_fan_factor_modifier_home();
        let mut status = format!("FanFactor {}", fan_factor_home);
        if modifier_home < 0 {
            status.push_str(&format!(" - {}", modifier_home.abs()));
        } else {
            status.push_str(&format!(" + {}", modifier_home));
        }
        status.push_str(&format!(" = {}", fan_factor_home + modifier_home));
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        if modifier_home > 0 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " win some new fans.");
        } else if modifier_home < 0 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " lose some fans.");
        } else {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " keep their fans.");
        }

        let mut status = String::new();
        let away_rolls = report.get_fan_factor_roll_away();
        if !away_rolls.is_empty() {
            status.push_str("Fan Factor Roll Away Team ");
            for roll in away_rolls {
                status.push_str(&format!("[ {} ]", roll));
            }
        } else {
            status.push_str("Fan Factor: Concession of Away Team");
        }
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let fan_factor_away = game.team_away.fan_factor;
        let modifier_away = report.get_fan_factor_modifier_away();
        let mut status = format!("FanFactor {}", fan_factor_away);
        if modifier_away < 0 {
            status.push_str(&format!(" - {}", modifier_away.abs()));
        } else {
            status.push_str(&format!(" + {}", modifier_away));
        }
        status.push_str(&format!(" = {}", fan_factor_away + modifier_away));
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        if modifier_away > 0 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " win some new fans.");
        } else if modifier_away < 0 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " lose some fans.");
        } else {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " keep their fans.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, fan_factor: i32) -> Team {
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", 5), make_team("away", 3), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_fan_factor_roll_post_match() {
        assert_eq!(FanFactorRollPostMatchMessage.get_key(), "fanFactorRoll");
    }

    #[test]
    fn positive_modifier_reports_win_some_new_fans() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactorRollPostMatch::new(vec![4], 1, vec![2], -1);
        FanFactorRollPostMatchMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" win some new fans.")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" lose some fans.")));
    }

    #[test]
    fn zero_modifier_keeps_fans() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactorRollPostMatch::new(vec![4], 0, vec![2], 0);
        FanFactorRollPostMatchMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs.iter().filter(|r| r.text.as_deref() == Some(" keep their fans.")).count(), 2);
    }

    #[test]
    fn concession_when_roll_empty() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFanFactorRollPostMatch::new(vec![], 0, vec![2], 0);
        FanFactorRollPostMatchMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Fan Factor: Concession of Home Team"));
    }
}
