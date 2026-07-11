use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::report_chainsaw_roll::ReportChainsawRoll;
use ffb_model::report::report_id::ReportId;

/// Java `PlayerGender.getDative()`/`getSelf()` have no equivalent method on the Rust
/// `PlayerGender` enum (only `nominative()`/`genitive()` are implemented there), so this is
/// a small local, directly-traceable translation of the Java enum's dative/self constants
/// (him/her/them/it, himself/herself/themself/itself).
fn dative(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "him",
        PlayerGender::Female => "her",
        PlayerGender::Nonbinary => "them",
        PlayerGender::Neutral => "it",
    }
}

/// 1:1 translation of `ChainsawRollMessage.java`.
pub struct ChainsawRollMessage;

impl ReportMessage for ChainsawRollMessage {
    type Report = ReportChainsawRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CHAINSAW_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Chainsaw Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        let status = if let Some(player) = player {
            if report.is_successful() {
                format!(" uses {} Chainsaw", player.gender.genitive())
            } else {
                format!("'s Chainsaw kicks back to hurt {}", dative(player.gender))
            }
        } else {
            String::new()
        };
        status_report.print_indent(status_report.get_indent() + 1, &status);

        if let Some(defender_id) = report.get_defender_id() {
            if !defender_id.is_empty() {
                status_report.print_indent(status_report.get_indent() + 1, " against ");
                let defender = game.player(defender_id);
                if defender.is_some() {
                    print_player(status_report, game, status_report.get_indent() + 1, false, defender);
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
        player.name = "Chopper".into();
        player.gender = PlayerGender::Male;
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn successful_use_prints_gender_genitive() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChainsawRoll::new(Some("p1".into()), true, 5, 2, false, vec![], None);
        ChainsawRollMessage.render(&mut status_report, &game, &report);
        let uses = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" uses his Chainsaw"));
        assert!(uses.is_some());
    }

    #[test]
    fn failed_use_prints_kickback_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChainsawRoll::new(Some("p1".into()), false, 1, 2, false, vec![], None);
        ChainsawRollMessage.render(&mut status_report, &game, &report);
        let kickback = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("'s Chainsaw kicks back to hurt him"));
        assert!(kickback.is_some());
    }

    #[test]
    fn with_defender_prints_against_defender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut defender = Player::default();
        defender.id = "d1".into();
        defender.name = "Target".into();
        game.team_away.players.push(defender);
        let report = ReportChainsawRoll::new(Some("p1".into()), true, 6, 2, false, vec![], Some("d1".into()));
        ChainsawRollMessage.render(&mut status_report, &game, &report);
        let against = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" against "));
        assert!(against.is_some());
        let target = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Target"));
        assert!(target.is_some());
    }

    #[test]
    fn without_defender_skips_against_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChainsawRoll::new(Some("p1".into()), true, 6, 2, false, vec![], None);
        ChainsawRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text.as_deref() != Some(" against ")));
    }

    #[test]
    fn report_id_is_chainsaw_roll() {
        assert_eq!(ChainsawRollMessage.report_id(), ReportId::CHAINSAW_ROLL);
    }
}
