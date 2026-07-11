use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2016::report_kickoff_pitch_invasion::ReportKickoffPitchInvasion;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::util_player::UtilPlayer;

pub struct KickoffPitchInvasionMessage;

impl ReportMessage for KickoffPitchInvasionMessage {
    type Report = ReportKickoffPitchInvasion;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_PITCH_INVASION
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let game_result = &game.game_result;
        let fan_favourites_home = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_home, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;
        let fan_favourites_away = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_away, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;

        let rolls_home = report.get_rolls_home();
        let players_affected_home = report.get_players_affected_home();
        for (i, home_player) in game.team_home.players.iter().enumerate() {
            if rolls_home.get(i).copied().unwrap_or(0) > 0 {
                let status = format!("Pitch Invasion Roll [ {} ]", rolls_home[i]);
                status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                print_player(status_report, game, indent + 1, false, Some(home_player));
                let affected_text = if players_affected_home.get(i).copied().unwrap_or(false) { " has been stunned." } else { " is unaffected." };
                let total = rolls_home[i] + game_result.away.fame + fan_favourites_away;
                let status = format!(
                    "{} (Roll {} + {} opposing FAME + {} opposing Fan Favourites = {} Total)",
                    affected_text, rolls_home[i], game_result.away.fame, fan_favourites_away, total
                );
                status_report.println_indent(indent + 1, &status);
            }
        }
        let rolls_away = report.get_rolls_away();
        let players_affected_away = report.get_players_affected_away();
        for (i, away_player) in game.team_away.players.iter().enumerate() {
            if rolls_away.get(i).copied().unwrap_or(0) > 0 {
                let status = format!("Pitch Invasion Roll [ {} ]", rolls_away[i]);
                status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                print_player(status_report, game, indent + 1, false, Some(away_player));
                let affected_text = if players_affected_away.get(i).copied().unwrap_or(false) { " has been stunned." } else { " is unaffected." };
                let total = rolls_away[i] + game_result.home.fame + fan_favourites_home;
                let status = format!(
                    "{} (Roll {} + {} opposing FAME  + {} opposing Fan Favourites = {} Total)",
                    affected_text, rolls_away[i], game_result.home.fame, fan_favourites_home, total
                );
                status_report.println_indent(indent + 1, &status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(), name: format!("Player {id}"), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(make_player("h1"));
        let mut away = make_team("away");
        away.players.push(make_player("a1"));
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn get_key_is_kickoff_pitch_invasion() {
        assert_eq!(KickoffPitchInvasionMessage.get_key(), "kickoffPitchInvasion");
    }

    #[test]
    fn stunned_home_player_reports_roll_and_effect() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(vec![5], vec![true], vec![0], vec![false]);
        KickoffPitchInvasionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Pitch Invasion Roll [ 5 ]"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref().unwrap_or("").contains("has been stunned")));
    }

    #[test]
    fn zero_roll_player_produces_no_output() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(vec![0], vec![false], vec![0], vec![false]);
        KickoffPitchInvasionMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn unaffected_away_player_reports_unaffected() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffPitchInvasion::new(vec![0], vec![false], vec![3], vec![false]);
        KickoffPitchInvasionMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref().unwrap_or("").contains("is unaffected")));
    }
}
