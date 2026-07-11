use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_go_for_it_roll::ReportGoForItRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `GoForItRollMessage.java`. Java's `ReportMessageBase<ReportSkillRoll>`
/// generic maps to `ReportGoForItRoll` here, since only the concrete subclass implements
/// `IReport` in the Rust port (`ReportSkillRoll` is embedded as `ReportGoForItRoll.base`).
pub struct GoForItRollMessage;

impl ReportMessage for GoForItRollMessage {
    type Report = ReportGoForItRoll;

    fn report_id(&self) -> ReportId {
        ReportId::GO_FOR_IT_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let base = &report.base;
        let indent = status_report.get_indent();

        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        let status = format!("Rush Roll [ {} ]", base.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        print_player(status_report, game, indent + 1, false, player);

        let mut needed_roll: Option<String> = None;
        if base.is_successful() {
            status_report.println_indent(indent + 1, " rushes!");
            if !base.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", base.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 1, " trips while rushing.");
            if !base.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", base.get_minimum_roll()));
            }
        }

        if let Some(needed_roll) = needed_roll {
            let needed_roll = format!(
                "{} (Roll{} > {}).",
                needed_roll,
                status_report.format_roll_modifiers(base.get_roll_modifiers()),
                base.get_minimum_roll() - 1
            );
            status_report.println_indent_style(indent + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::acting_player::ActingPlayer;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            players: vec![Player { id: "p1".into(), name: "Grubber".into(), ..Default::default() }],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("p1".into()), ..Default::default() };
        game
    }

    #[test]
    fn renders_success_without_reroll() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        GoForItRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Rush Roll [ 4 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Grubber"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" rushes!"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("Succeeded on a roll of 2+ (Roll > 1)."));
        assert_eq!(sr.rendered_runs[5].text_style, Some(TextStyle::NEEDED_ROLL));
    }

    #[test]
    fn renders_failure_without_reroll() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), false, 1, 3, false, vec!["TackleZone".into()]);
        GoForItRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" trips while rushing."));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("Roll a 3+ to succeed (Roll - TackleZone > 2)."));
    }

    #[test]
    fn skips_needed_roll_when_re_rolled() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), true, 4, 2, true, vec![]);
        GoForItRollMessage.render(&mut sr, &game, &report);
        // roll println (2 runs) + player print (1 run) + " rushes!" println (2 runs); no needed-roll line.
        assert_eq!(sr.rendered_runs.len(), 5);
    }
}
