use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_kickoff_pitch_invasion::ReportKickoffPitchInvasion;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffPitchInvasionMessage.java`.
pub struct KickoffPitchInvasionMessage;

impl ReportMessage for KickoffPitchInvasionMessage {
    type Report = ReportKickoffPitchInvasion;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_PITCH_INVASION
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let game_result = &game.game_result;

        if report.get_amount() > 0 {
            let status = format!("Pitch Invasion Roll [ {} ]", report.get_amount());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);

            let status = format!(
                "Affected Teams will have {} player{} stunned.",
                report.get_amount(),
                if report.get_amount() > 1 { "s" } else { "" }
            );
            status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &status);
        }

        let status = format!("Pitch Invasion Roll Home Team [ {} ]", report.get_roll_home());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_home = report.get_roll_home() + game_result.home.fan_factor;
        let status = format!(
            "Rolled {} + {} Fan Factor = {}.",
            report.get_roll_home(),
            game_result.home.fan_factor,
            total_home
        );
        status_report.println_indent(indent + 1, &status);

        let status = format!("Pitch Invasion Roll Away Team [ {} ]", report.get_roll_away());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_away = report.get_roll_away() + game_result.away.fan_factor;
        let status = format!(
            "Rolled {} + {} Fan Factor = {}.",
            report.get_roll_away(),
            game_result.away.fan_factor,
            total_away
        );
        status_report.println_indent(indent + 1, &status);

        for player_id in report.get_affected_players() {
            let player = game.player(player_id);
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " is stunned");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut game = Game::new(
            empty_team("home", vec![Player { id: "p1".into(), name: "Stunned Guy".into(), ..Default::default() }]),
            empty_team("away", vec![]),
            Rules::Bb2020,
        );
        game.game_result.home.fan_factor = 3;
        game.game_result.away.fan_factor = 2;
        game
    }

    #[test]
    fn renders_pitch_invasion_roll_when_amount_positive_plural() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(3, 4, 2, vec![]);
        KickoffPitchInvasionMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Pitch Invasion Roll [ 2 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Affected Teams will have 2 players stunned."));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some("Pitch Invasion Roll Home Team [ 3 ]"));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some("Rolled 3 + 3 Fan Factor = 6."));
        assert_eq!(sr.rendered_runs[8].text.as_deref(), Some("Pitch Invasion Roll Away Team [ 4 ]"));
        assert_eq!(sr.rendered_runs[10].text.as_deref(), Some("Rolled 4 + 2 Fan Factor = 6."));
    }

    #[test]
    fn skips_amount_lines_when_amount_zero() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(1, 2, 0, vec![]);
        KickoffPitchInvasionMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Pitch Invasion Roll Home Team [ 1 ]"));
    }

    #[test]
    fn renders_singular_player_stunned_text_for_amount_one() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(1, 2, 1, vec![]);
        KickoffPitchInvasionMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Affected Teams will have 1 player stunned."));
    }

    #[test]
    fn renders_affected_players() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(1, 2, 0, vec!["p1".into()]);
        KickoffPitchInvasionMessage.render(&mut sr, &game, &report);
        // last two runs: player name + " is stunned" line.
        let len = sr.rendered_runs.len();
        assert_eq!(sr.rendered_runs[len - 3].text.as_deref(), Some("Stunned Guy"));
        assert_eq!(sr.rendered_runs[len - 2].text.as_deref(), Some(" is stunned"));
    }
}
