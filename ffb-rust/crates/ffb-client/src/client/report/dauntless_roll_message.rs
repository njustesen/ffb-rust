use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::report_dauntless_roll::ReportDauntlessRoll;
use ffb_model::report::report_id::ReportId;

/// Java `PlayerGender.getSelf()` has no equivalent on the Rust `PlayerGender` enum. This is a
/// small local, directly-traceable translation of the Java enum's "self" constants
/// (himself/herself/themself/itself).
fn self_word(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "himself",
        PlayerGender::Female => "herself",
        PlayerGender::Nonbinary => "themself",
        PlayerGender::Neutral => "itself",
    }
}

/// 1:1 translation of `DauntlessRollMessage.java`.
pub struct DauntlessRollMessage;

impl ReportMessage for DauntlessRollMessage {
    type Report = ReportDauntlessRoll;

    fn report_id(&self) -> ReportId {
        ReportId::DAUNTLESS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Dauntless Roll [ {} ]", report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        let status = if let Some(player) = player {
            if report.base.is_successful() {
                format!(" uses Dauntless to push {} to strength {}", self_word(player.gender), report.get_strength())
            } else {
                format!(" fails to push {} strength", player.gender.genitive())
            }
        } else {
            String::new()
        };
        status_report.print_indent(status_report.get_indent() + 1, &status);
        if let Some(defender_id) = report.get_defender_id() {
            if !defender_id.is_empty() {
                if let Some(defender) = game.player(defender_id) {
                    status_report.print_indent(status_report.get_indent() + 1, " to match ");
                    print_player(status_report, game, status_report.get_indent() + 1, false, Some(defender));
                }
            }
        }
        status_report.println_indent(status_report.get_indent() + 1, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2025);
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Strongman".into();
        player.gender = PlayerGender::Female;
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn successful_push_reports_new_strength() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDauntlessRoll::new(Some("p1".into()), true, 5, 3, false, 5, None);
        DauntlessRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" uses Dauntless to push herself to strength 5"));
        assert!(msg.is_some());
    }

    #[test]
    fn failed_push_reports_genitive_strength() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDauntlessRoll::new(Some("p1".into()), false, 1, 3, false, 3, None);
        DauntlessRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" fails to push her strength"));
        assert!(msg.is_some());
    }

    #[test]
    fn with_defender_prints_to_match_defender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut defender = Player::default();
        defender.id = "d1".into();
        defender.name = "Blocker".into();
        game.team_away.players.push(defender);
        let report = ReportDauntlessRoll::new(Some("p1".into()), true, 6, 3, false, 5, Some("d1".into()));
        DauntlessRollMessage.render(&mut status_report, &game, &report);
        let to_match = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" to match "));
        assert!(to_match.is_some());
        let blocker = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Blocker"));
        assert!(blocker.is_some());
    }

    #[test]
    fn without_defender_skips_to_match_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDauntlessRoll::new(Some("p1".into()), true, 6, 3, false, 5, None);
        DauntlessRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text.as_deref() != Some(" to match ")));
    }

    #[test]
    fn report_id_is_dauntless_roll() {
        assert_eq!(DauntlessRollMessage.report_id(), ReportId::DAUNTLESS_ROLL);
    }
}
