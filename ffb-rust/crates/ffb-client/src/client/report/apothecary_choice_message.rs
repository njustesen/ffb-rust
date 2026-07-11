use crate::client::report::report_message_base::print_player;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PS_RESERVE;
use ffb_model::model::game::Game;
use ffb_model::report::report_apothecary_choice::ReportApothecaryChoice;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ApothecaryChoiceMessage.java`.
pub struct ApothecaryChoiceMessage;

impl ReportMessage for ApothecaryChoiceMessage {
    type Report = ReportApothecaryChoice;

    fn report_id(&self) -> ReportId {
        ReportId::APOTHECARY_CHOICE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let game_result = &game.game_result;
        let player = game.player(report.get_player_id());
        if report.get_player_state().base() == PS_RESERVE {
            status_report.print_indent_style(status_report.get_indent(), TextStyle::BOLD, "The apothecary patches ");
            print_player(status_report, game, status_report.get_indent(), true, player);
            let nominative = player.map(|p| p.gender.nominative()).unwrap_or("");
            status_report.println_indent_style(
                status_report.get_indent(),
                TextStyle::BOLD,
                &format!(" up so {nominative} is able to play again."),
            );
        } else {
            status_report.print_indent(status_report.get_indent(), "Coach ");
            if let Some(player) = player {
                if game.team_home.has_player(&player.id) {
                    status_report.print_indent_style(status_report.get_indent(), TextStyle::HOME, &game.team_home.coach.clone());
                } else {
                    status_report.print_indent_style(status_report.get_indent(), TextStyle::AWAY, &game.team_away.coach.clone());
                }
            }
            let player_state_old = player.and_then(|p| game.field_model.player_state(&p.id));
            let serious_injury_old = player
                .and_then(|p| game_result.team_result(game.team_home.has_player(&p.id)).player_result(&p.id))
                .and_then(|r| r.serious_injury);
            let serious_injury_old_name = serious_injury_old.map(|k| k.name());
            if Some(*report.get_player_state()) != player_state_old || report.get_serious_injury() != serious_injury_old_name {
                status_report.println_indent(status_report.get_indent(), " chooses the new injury result.");
            } else {
                status_report.println_indent(status_report.get_indent(), " keeps the old injury result.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerState, Rules, SeriousInjuryKind};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: format!("coach_{id}"), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
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
        assert_eq!(ApothecaryChoiceMessage.report_id(), ReportId::APOTHECARY_CHOICE);
    }

    #[test]
    fn reserve_state_patches_player() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Patched".into();
        player.gender = PlayerGender::Male;
        game.team_home.players.push(player);

        let report = ReportApothecaryChoice::new("p1".into(), PlayerState::new(PS_RESERVE), None);
        let mut status_report = StatusReport::new();
        ApothecaryChoiceMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("The apothecary patches "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Patched"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" up so he is able to play again."));
    }

    #[test]
    fn keeps_old_injury_when_state_and_injury_unchanged() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Injured".into();
        game.team_home.players.push(player);
        let old_state = PlayerState::new(0);
        game.field_model.set_player_state("p1", old_state);
        game.game_result.team_result_mut(true).player_result_mut("p1").serious_injury = None;

        let report = ReportApothecaryChoice::new("p1".into(), old_state, None);
        let mut status_report = StatusReport::new();
        ApothecaryChoiceMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Coach "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("coach_home"));
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" keeps the old injury result."));
    }

    #[test]
    fn chooses_new_injury_when_serious_injury_differs() {
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Injured".into();
        game.team_away.players.push(player);
        let old_state = PlayerState::new(0);
        game.field_model.set_player_state("p1", old_state);
        game.game_result.team_result_mut(false).player_result_mut("p1").serious_injury = None;

        let report = ReportApothecaryChoice::new("p1".into(), old_state, Some(SeriousInjuryKind::BrokenRibs.name().to_string()));
        let mut status_report = StatusReport::new();
        ApothecaryChoiceMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" chooses the new injury result."));
    }
}
