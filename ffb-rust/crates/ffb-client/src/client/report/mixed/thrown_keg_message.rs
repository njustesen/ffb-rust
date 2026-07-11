use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::gender_self;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_thrown_keg::ReportThrownKeg;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ThrownKegMessage.java`.
pub struct ThrownKegMessage;

impl ReportMessage for ThrownKegMessage {
    type Report = ReportThrownKeg;

    fn report_id(&self) -> ReportId {
        ReportId::THROWN_KEG
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Beer Barrel Bash Roll [ {} ]", report.get_roll()));
        let thrower = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent, false, thrower);
        status_report.print_indent_style(indent, TextStyle::NONE, " hits ");
        if report.is_fumble() {
            // java: `thrower.getPlayerGender().getSelf()` — no null-guard needed by Java
            // here either since a fumble implies the thrower exists.
            if let Some(thrower) = thrower {
                status_report.print_indent_style(indent, TextStyle::NONE, gender_self(thrower.gender));
            }
        } else if report.is_success() {
            let target = report.get_target_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, indent, false, target);
        } else {
            status_report.print_indent_style(indent, TextStyle::NONE, "no one");
        }
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: format!("Coach {id}"),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("thrower", "Thrower", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("target", "Target", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn success_hits_target() {
        let game = make_game();
        let report = ReportThrownKeg::new(Some("thrower".into()), Some("target".into()), 5, true, false);
        let mut status_report = StatusReport::new();
        ThrownKegMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Beer Barrel Bash Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Thrower"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" hits "));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some("Target"));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("."));
    }

    #[test]
    fn fumble_hits_self() {
        let game = make_game();
        let report = ReportThrownKeg::new(Some("thrower".into()), None, 1, false, true);
        let mut status_report = StatusReport::new();
        ThrownKegMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some("himself"));
    }

    #[test]
    fn miss_hits_no_one() {
        let game = make_game();
        let report = ReportThrownKeg::new(Some("thrower".into()), None, 2, false, false);
        let mut status_report = StatusReport::new();
        ThrownKegMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some("no one"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(ThrownKegMessage.report_id(), ReportId::THROWN_KEG);
        assert_eq!(ThrownKegMessage.get_key(), "thrownKeg");
    }
}
