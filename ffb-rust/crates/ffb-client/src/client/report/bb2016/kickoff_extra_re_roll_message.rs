use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2016::report_kickoff_extra_re_roll::ReportKickoffExtraReRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::util_player::UtilPlayer;

pub struct KickoffExtraReRollMessage;

impl ReportMessage for KickoffExtraReRollMessage {
    type Report = ReportKickoffExtraReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_EXTRA_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let game_result = &game.game_result;
        let kickoff_result = report.get_kickoff_result();
        let fan_favourites_home = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_home, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;
        let fan_favourites_away = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_away, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;

        if kickoff_result.is_fan_reroll() {
            let status = format!("Cheering Fans Roll Home Team [ {} ]", report.get_roll_home());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let total_home = report.get_roll_home() + game_result.home.fame + fan_favourites_home + game.team_home.cheerleaders;
            let status = format!(
                "Rolled {} + {} FAME + {} Fan Favourites + {} Cheerleaders = {}.",
                report.get_roll_home(), game_result.home.fame, fan_favourites_home, game.team_home.cheerleaders, total_home
            );
            status_report.println_indent(indent + 1, &status);
            let status = format!("Cheering Fans Roll Away Team [ {} ]", report.get_roll_away());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let total_away = report.get_roll_away() + game_result.away.fame + fan_favourites_away + game.team_away.cheerleaders;
            let status = format!(
                "Rolled {} + {} FAME + {} Fan Favourites + {} Cheerleaders = {}.",
                report.get_roll_away(), game_result.away.fame, fan_favourites_away, game.team_away.cheerleaders, total_away
            );
            status_report.println_indent(indent + 1, &status);
        }
        if kickoff_result.is_coach_reroll() {
            let home_banned = game.turn_data_home.coach_banned;
            let away_banned = game.turn_data_away.coach_banned;

            let status = format!("Brilliant Coaching Roll Home Team [ {} ]", report.get_roll_home());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let total_home = report.get_roll_home() + game_result.home.fame + fan_favourites_home + game.team_home.assistant_coaches - if home_banned { 1 } else { 0 };
            let status = format!(
                "Rolled {} + {} FAME + {} Fan Favourites + {} Assistant Coaches {} Coach = {}.",
                report.get_roll_home(), game_result.home.fame, fan_favourites_home, game.team_home.assistant_coaches,
                if home_banned { "- 1 Banned" } else { " + 0 Head" }, total_home
            );
            status_report.println_indent(indent + 1, &status);
            let status = format!("Brilliant Coaching Roll Away Team [ {} ]", report.get_roll_away());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let total_away = report.get_roll_away() + game_result.away.fame + fan_favourites_away + game.team_away.assistant_coaches - if away_banned { 1 } else { 0 };
            let status = format!(
                "Rolled {} + {} FAME + {} Fan Favourites + {} Assistant Coaches {} Coach = {}.",
                report.get_roll_away(), game_result.away.fame, fan_favourites_away, game.team_away.assistant_coaches,
                if away_banned { "- 1 Banned" } else { " + 0 Head" }, total_away
            );
            status_report.println_indent(indent + 1, &status);
        }
        if report.is_home_gains_re_roll() {
            status_report.print_indent(indent, "Team ");
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
            status_report.println_indent(indent, " gains a Re-Roll.");
        }
        if report.is_away_gains_re_roll() {
            status_report.print_indent(indent, "Team ");
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
            status_report.println_indent(indent, " gains a Re-Roll.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{KickoffResult, Rules};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_kickoff_extra_re_roll() {
        assert_eq!(KickoffExtraReRollMessage.get_key(), "extraReRoll");
    }

    #[test]
    fn home_gains_re_roll_reports_team_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffExtraReRoll::new(KickoffResult::WeatherChange, 0, true, 0, false);
        KickoffExtraReRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" gains a Re-Roll.")));
    }

    #[test]
    fn fan_reroll_kickoff_reports_cheering_fans_rolls() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffExtraReRoll::new(KickoffResult::CheeringFans, 4, false, 5, false);
        KickoffExtraReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Cheering Fans Roll Home Team [ 4 ]"));
    }

    #[test]
    fn coach_reroll_kickoff_reports_brilliant_coaching_rolls() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffExtraReRoll::new(KickoffResult::BrilliantCoaching, 3, false, 2, false);
        KickoffExtraReRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Brilliant Coaching Roll Home Team [ 3 ]")));
    }
}
