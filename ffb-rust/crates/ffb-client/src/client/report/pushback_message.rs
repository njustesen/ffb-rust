use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_pushback::ReportPushback;

/// 1:1 translation of `PushbackMessage.java`.
pub struct PushbackMessage;

impl ReportMessage for PushbackMessage {
    type Report = ReportPushback;

    fn report_id(&self) -> ReportId {
        ReportId::PUSHBACK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent() + 1;
        let defender = game.player(report.get_defender_id());
        if report.get_pushback_mode() == PushbackMode::SIDE_STEP {
            print_player(status_report, game, indent, false, defender);
            let status = " uses Side Step to avoid being pushed.".to_string();
            status_report.println_indent(indent, &status);
        }
        if report.get_pushback_mode() == PushbackMode::GRAB {
            let acting_player = game.player(game.acting_player.player_id.as_deref().unwrap_or(""));
            print_player(status_report, game, indent, false, acting_player);
            let status = format!(
                " uses Grab to place {} opponent.",
                acting_player.map(|p| p.gender.genitive()).unwrap_or("")
            );
            status_report.println_indent(indent, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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

    #[test]
    fn report_id_matches() {
        assert_eq!(PushbackMessage.report_id(), ReportId::PUSHBACK);
    }

    #[test]
    fn render_side_step_prints_defender_message() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut defender = Player::default();
        defender.id = "d1".into();
        defender.name = "Defender".into();
        game.team_away.players.push(defender);
        let report = ReportPushback::new("d1".into(), PushbackMode::SIDE_STEP);
        PushbackMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Defender"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" uses Side Step to avoid being pushed."));
    }

    #[test]
    fn render_grab_uses_acting_player_gender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut actor = Player::default();
        actor.id = "a1".into();
        actor.name = "Actor".into();
        actor.gender = PlayerGender::Female;
        game.team_home.players.push(actor);
        game.acting_player.player_id = Some("a1".into());
        let report = ReportPushback::new("d1".into(), PushbackMode::GRAB);
        PushbackMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Actor"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" uses Grab to place her opponent."));
    }

    #[test]
    fn render_regular_mode_prints_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPushback::new("d1".into(), PushbackMode::REGULAR);
        PushbackMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
