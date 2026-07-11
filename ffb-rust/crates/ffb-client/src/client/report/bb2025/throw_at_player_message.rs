use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_throw_at_player::ReportThrowAtPlayer;
use ffb_model::report::report_id::ReportId;

// java: PlayerGender.getDative() — not yet ported to the shared enum, so translated locally here.
fn player_gender_dative(gender: ffb_model::enums::PlayerGender) -> &'static str {
    use ffb_model::enums::PlayerGender::*;
    match gender {
        Male => "him",
        Female => "her",
        Nonbinary => "them",
        Neutral => "it",
    }
}

/// 1:1 translation of `ThrowAtPlayerMessage.java`.
pub struct ThrowAtPlayerMessage;

impl ReportMessage for ThrowAtPlayerMessage {
    type Report = ReportThrowAtPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::THROW_AT_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.player(report.get_player_id());

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Throw a Rock Roll [ {} ]", report.get_roll()));

        status_report.print_indent(indent + 1, "Fans throw a rock at ");
        print_player(status_report, game, indent + 1, true, player);
        let dative = player.map(|p| player_gender_dative(p.gender)).unwrap_or("");
        let message = if report.is_successful() {
            format!(" knocking {dative} down.")
        } else {
            format!(" but miss {dative}.")
        };
        status_report.println_indent(indent + 1, &message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), gender, ..Default::default() }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Victim", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("p2", "Target", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_throw_at_player() {
        assert_eq!(ThrowAtPlayerMessage.report_id(), ReportId::THROW_AT_PLAYER);
    }

    #[test]
    fn successful_hit_knocks_him_down() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtPlayer::new("p1".into(), 6, true);
        ThrowAtPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Throw a Rock Roll [ 6 ]"));
        assert!(texts.iter().any(|t| t == " knocking him down."));
    }

    #[test]
    fn miss_uses_dative_pronoun() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtPlayer::new("p2".into(), 1, false);
        ThrowAtPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " but miss her."));
    }

    #[test]
    fn prints_player_bold() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtPlayer::new("p1".into(), 4, true);
        ThrowAtPlayerMessage.render(&mut status_report, &game, &report);
        let run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Victim")).unwrap();
        assert_eq!(run.text_style, Some(TextStyle::HOME_BOLD));
    }
}
