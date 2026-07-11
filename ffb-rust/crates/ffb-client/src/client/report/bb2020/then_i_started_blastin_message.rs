use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_then_i_started_blastin::ReportThenIStartedBlastin;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

/// java: `PlayerGender.getSelf()` — not yet exposed on the Rust `PlayerGender` enum.
fn gender_self(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "himself",
        PlayerGender::Female => "herself",
        PlayerGender::Nonbinary => "themself",
        PlayerGender::Neutral => "itself",
    }
}

/// 1:1 translation of `ThenIStartedBlastinMessage.java`.
pub struct ThenIStartedBlastinMessage;

impl ReportMessage for ThenIStartedBlastinMessage {
    type Report = ReportThenIStartedBlastin;

    fn report_id(&self) -> ReportId {
        ReportId::THEN_I_STARTED_BLASTIN
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        if report.get_roll() > 0 {
            status_report.println_indent_style(
                indent,
                TextStyle::ROLL,
                &format!("\"Then I Started Blastin'!\" Roll [ {} ]", report.get_roll()),
            );
        }
        let thrower = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent, false, thrower);
        if string_tool::is_provided(report.get_target_player_id()) {
            status_report.print_indent_style(indent, TextStyle::NONE, " hits ");
            if report.is_fumble() {
                let text = thrower.map(|p| gender_self(p.gender)).unwrap_or("");
                status_report.print_indent_style(indent, TextStyle::NONE, text);
            } else if report.is_success() {
                let target = report.get_target_player_id().and_then(|id| game.player(id));
                print_player(status_report, game, indent, false, target);
            } else {
                status_report.print_indent_style(indent, TextStyle::NONE, "a player chosen by the opposing coach");
            }
        } else {
            status_report.print_indent_style(indent, TextStyle::NONE, " starts blastin' ");
        }
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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

    fn make_game() -> Game {
        Game::new(
            make_team("home", vec![make_player("thrower", PlayerGender::Male)]),
            make_team("away", vec![make_player("target", PlayerGender::Female)]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn fumble_prints_thrower_gender_self() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 5, false, true);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"himself"));
    }

    #[test]
    fn success_prints_target_player() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 5, true, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("Player target")));
    }

    #[test]
    fn opponent_chosen_when_not_success_or_fumble() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), Some("target".into()), 5, false, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"a player chosen by the opposing coach"));
    }

    #[test]
    fn no_target_starts_blastin() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThenIStartedBlastin::new(Some("thrower".into()), None, 0, false, false);
        ThenIStartedBlastinMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" starts blastin' "));
        // roll <= 0, so no roll-line should be present.
        assert!(!texts.iter().any(|t| t.contains("Roll [")));
    }
}
