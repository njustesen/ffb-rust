use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::report_bribes_roll::ReportBribesRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `PlayerGender.getVerbForm(String, String)`. The Rust `PlayerGender` enum (shared,
/// generated 1:1 from `ffb-common/PlayerGender.java`) does not carry the verb-form field, so
/// this small local helper reproduces the singular/plural branch faithfully: only
/// `Nonbinary` uses the plural verb form (Java: `VerbForm.plural`), everything else singular.
fn verb_form(gender: PlayerGender, singular_form: &'static str, plural_form: &'static str) -> &'static str {
    match gender {
        PlayerGender::Nonbinary => plural_form,
        _ => singular_form,
    }
}

/// 1:1 translation of `BribesRollMessage.java`.
pub struct BribesRollMessage;

impl ReportMessage for BribesRollMessage {
    type Report = ReportBribesRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BRIBES_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_player_id());
        let status = format!("Bribes Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        if report.is_successful() {
            let gender = player.map(|p| p.gender).unwrap_or(PlayerGender::Neutral);
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::NONE, "The ref refrains from penalizing ");
            print_player(status_report, game, status_report.get_indent() + 1, false, player);
            let status = format!(" and {} {} in the game.", gender.nominative(), verb_form(gender, "remains", "remain"));
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NONE, &status);
        } else {
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::NONE, "The ref appears to be unimpressed and ");
            print_player(status_report, game, status_report.get_indent() + 1, false, player);
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NONE, " must leave the game.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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
        assert_eq!(BribesRollMessage.report_id(), ReportId::BRIBES_ROLL);
    }

    #[test]
    fn successful_singular_gender_remains() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Bribed".into();
        player.gender = PlayerGender::Male;
        game.team_home.players.push(player);

        let report = ReportBribesRoll::new("p1".into(), true, 5);
        let mut status_report = StatusReport::new();
        BribesRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Bribes Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("The ref refrains from penalizing "));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("Bribed"));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" and he remains in the game."));
    }

    #[test]
    fn successful_nonbinary_gender_remain_plural() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Bribed".into();
        player.gender = PlayerGender::Nonbinary;
        game.team_home.players.push(player);

        let report = ReportBribesRoll::new("p1".into(), true, 5);
        let mut status_report = StatusReport::new();
        BribesRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" and they remain in the game."));
    }

    #[test]
    fn unsuccessful_player_must_leave() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p2".into();
        player.name = "Penalized".into();
        game.team_away.players.push(player);

        let report = ReportBribesRoll::new("p2".into(), false, 1);
        let mut status_report = StatusReport::new();
        BribesRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("The ref appears to be unimpressed and "));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("Penalized"));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" must leave the game."));
    }
}
