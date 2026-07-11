use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_dedicated_fans::ReportDedicatedFans;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `DedicatedFansMessage.java`.
pub struct DedicatedFansMessage;

impl DedicatedFansMessage {
    /// Java: private `printTeamRoll(int, int, String, TextStyle, boolean)`.
    fn print_team_roll(
        status_report: &mut StatusReport,
        roll: i32,
        modifier: i32,
        team_name: &str,
        team_style: TextStyle,
        conceded: bool,
    ) {
        let indent = status_report.get_indent();

        if roll > 0 {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Dedicated Fans Roll [ {roll} ]"));
        }

        status_report.print_indent_style(indent + 1, team_style, team_name);

        let mut text = String::new();

        if modifier > 0 {
            text.push_str(" gain ");
            text.push_str(&modifier.to_string());
        } else if modifier < 0 {
            text.push_str(" lose ");
            text.push_str(&modifier.abs().to_string());
        } else {
            text.push_str(" keep their");
        }

        text.push_str(" Dedicated Fan");

        if modifier.abs() != 1 {
            text.push('s');
        }

        if conceded && modifier != 0 {
            text.push_str(" due to conceding");
        }
        text.push('.');

        status_report.println_indent_style(indent + 1, TextStyle::NONE, &text);
    }
}

impl ReportMessage for DedicatedFansMessage {
    type Report = ReportDedicatedFans;

    fn report_id(&self) -> ReportId {
        ReportId::DEDICATED_FANS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let home_conceded =
            report.is_conceded() && report.get_conceded_team() == Some(game.team_home.id.as_str());
        Self::print_team_roll(
            status_report,
            report.get_roll_home(),
            report.get_modifier_home(),
            &game.team_home.name,
            TextStyle::HOME_BOLD,
            home_conceded,
        );

        let away_conceded =
            report.is_conceded() && report.get_conceded_team() == Some(game.team_away.id.as_str());
        Self::print_team_roll(
            status_report,
            report.get_roll_away(),
            report.get_modifier_away(),
            &game.team_away.name,
            TextStyle::AWAY_BOLD,
            away_conceded,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
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
    fn renders_gain_for_positive_modifier_home() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDedicatedFans::new(3, 1, 0, 0, None, false);
        DedicatedFansMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Dedicated Fans Roll [ 3 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Team home"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" gain 1 Dedicated Fan."));
    }

    #[test]
    fn renders_lose_for_negative_modifier_and_plural() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDedicatedFans::new(0, 0, 5, -2, None, false);
        DedicatedFansMessage.render(&mut sr, &game, &report);
        // home roll is 0 -> no roll line, only team name + text (2 runs).
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Team home"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" keep their Dedicated Fans."));
        // away roll is 5 -> roll line emitted.
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Dedicated Fans Roll [ 5 ]"));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some(" lose 2 Dedicated Fans."));
    }

    #[test]
    fn renders_conceding_suffix_when_conceded_and_nonzero_modifier() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDedicatedFans::new(2, -1, 0, 0, Some("home".into()), true);
        DedicatedFansMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" lose 1 Dedicated Fan due to conceding."));
    }

    #[test]
    fn no_conceding_suffix_when_modifier_zero_even_if_conceded() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDedicatedFans::new(0, 0, 0, 0, Some("home".into()), true);
        DedicatedFansMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" keep their Dedicated Fans."));
    }
}
