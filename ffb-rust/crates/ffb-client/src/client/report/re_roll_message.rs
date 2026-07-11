use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_re_roll::ReportReRoll;

/// 1:1 translation of `ReRollMessage.java`.
pub struct ReRollMessage;

impl ReportMessage for ReRollMessage {
    type Report = ReportReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        if report.get_re_roll_source().name == "Loner" {
            let status = format!("Loner Roll [ {} ]", report.get_roll());
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &status);
            print_player(status_report, game, status_report.get_indent() + 2, false, player);
            if report.is_successful() {
                status_report.println_indent(status_report.get_indent() + 2, " may use a Team Re-Roll.");
            } else {
                status_report.println_indent(status_report.get_indent() + 2, " wastes a Team Re-Roll.");
            }
        } else if report.get_re_roll_source().name == "Pro" {
            let status = format!("Pro Roll [ {} ]", report.get_roll());
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &status);
            print_player(status_report, game, status_report.get_indent() + 2, false, player);
            let status = if report.is_successful() {
                format!(
                    "'s Pro skill allows {} to re-roll the action.",
                    player.map(|p| p.gender.dative()).unwrap_or("")
                )
            } else {
                format!("'s Pro skill does not help {}.", player.map(|p| p.gender.dative()).unwrap_or(""))
            };
            status_report.println_indent(status_report.get_indent() + 2, &status);
        } else {
            // java: report.getReRollSource().getName(game) resolves the source name via
            // game.getRules().getSkillFactory() when a matching skill exists; the Rust Game
            // model has no skill-factory hookup here, so we fall back to the raw source name
            // (the same fallback Java's getName(game) uses when no skill matches).
            let status = format!("Re-Roll using {}", report.get_re_roll_source().name.to_uppercase());
            status_report.println_indent(status_report.get_indent() + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ReRollSource, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::player_gender::PlayerGender;
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

    fn add_player(game: &mut Game, gender: PlayerGender) {
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Grombrindal".into();
        player.gender = gender;
        game.team_home.players.push(player);
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(ReRollMessage.report_id(), ReportId::RE_ROLL);
    }

    #[test]
    fn render_loner_successful() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, PlayerGender::Male);
        let report = ReportReRoll::new(Some("p1".into()), ReRollSource::new("Loner"), true, 4);
        ReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Loner Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" may use a Team Re-Roll."));
    }

    #[test]
    fn render_loner_failed() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, PlayerGender::Male);
        let report = ReportReRoll::new(Some("p1".into()), ReRollSource::new("Loner"), false, 1);
        ReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" wastes a Team Re-Roll."));
    }

    #[test]
    fn render_pro_successful_uses_dative() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, PlayerGender::Female);
        let report = ReportReRoll::new(Some("p1".into()), ReRollSource::new("Pro"), true, 4);
        ReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Pro Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("'s Pro skill allows her to re-roll the action."));
    }

    #[test]
    fn render_pro_failed_uses_dative() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, PlayerGender::Nonbinary);
        let report = ReportReRoll::new(Some("p1".into()), ReRollSource::new("Pro"), false, 1);
        ReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("'s Pro skill does not help them."));
    }

    #[test]
    fn render_other_source_prints_uppercased_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReRoll::new(None, ReRollSource::new("Team ReRoll"), true, 3);
        ReRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Re-Roll using TEAM REROLL"));
    }
}
