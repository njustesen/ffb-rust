use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_always_hungry_roll::ReportAlwaysHungryRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `AlwaysHungryMessage.java`.
pub struct AlwaysHungryMessage;

impl ReportMessage for AlwaysHungryMessage {
    type Report = ReportAlwaysHungryRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ALWAYS_HUNGRY_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Always Hungry Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, thrower);
        let status = if report.is_successful() {
            " resists the hunger.".to_string()
        } else {
            format!(" tries to eat {} team-mate.", thrower.map(|p| p.gender.genitive()).unwrap_or(""))
        };
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NONE, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
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
        assert_eq!(AlwaysHungryMessage.report_id(), ReportId::ALWAYS_HUNGRY_ROLL);
    }

    #[test]
    fn renders_success() {
        let mut game = make_game();
        let mut thrower = Player::default();
        thrower.id = "p1".into();
        thrower.name = "Thrower".into();
        thrower.gender = PlayerGender::Male;
        game.team_home.players.push(thrower);
        game.acting_player.player_id = Some("p1".into());

        let report = ReportAlwaysHungryRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        let mut status_report = StatusReport::new();
        AlwaysHungryMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Always Hungry Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Thrower"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" resists the hunger."));
    }

    #[test]
    fn renders_failure_uses_genitive() {
        let mut game = make_game();
        let mut thrower = Player::default();
        thrower.id = "p1".into();
        thrower.name = "Thrower".into();
        thrower.gender = PlayerGender::Female;
        game.team_home.players.push(thrower);
        game.acting_player.player_id = Some("p1".into());

        let report = ReportAlwaysHungryRoll::new(Some("p1".into()), false, 2, 4, false, vec![]);
        let mut status_report = StatusReport::new();
        AlwaysHungryMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" tries to eat her team-mate."));
    }

    #[test]
    fn renders_away_thrower() {
        let mut game = make_game();
        let mut thrower = Player::default();
        thrower.id = "p2".into();
        thrower.name = "AwayThrower".into();
        thrower.gender = PlayerGender::Neutral;
        game.team_away.players.push(thrower);
        game.acting_player.player_id = Some("p2".into());

        let report = ReportAlwaysHungryRoll::new(Some("p2".into()), false, 2, 4, false, vec![]);
        let mut status_report = StatusReport::new();
        AlwaysHungryMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" tries to eat its team-mate."));
    }
}
