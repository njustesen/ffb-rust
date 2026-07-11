use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_blood_lust_roll::ReportBloodLustRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BloodLustRollMessage.java`. Java's `ReportMessageBase<ReportSkillRoll>`
/// generic maps to `ReportBloodLustRoll` here, since only the concrete subclass implements
/// `IReport` in the Rust port (`ReportSkillRoll` is embedded as `ReportBloodLustRoll.base`).
pub struct BloodLustRollMessage;

impl ReportMessage for BloodLustRollMessage {
    type Report = ReportBloodLustRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BLOOD_LUST_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Blood Lust Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 1, false, player);

        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            status_report.println_indent(indent + 1, " resists the Blood Lust.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 1, " gives in to the Blood Lust.");
            status_report.println_indent(
                indent + 1,
                "Player must feed at the end of the action or lose tackle zone (and drop ball if carrying) and suffer a turnover.",
            );
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }

        if let Some(needed_roll) = needed_roll {
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

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let player = Player { id: "p1".into(), name: "Vampire".into(), ..Player::default() };
        let mut game = Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("p1".into()), ..Default::default() };
        game
    }

    #[test]
    fn success_without_re_roll_shows_needed_roll() {
        let game = make_game();
        let report = ReportBloodLustRoll::new(None, true, 4, 2, false, vec![]);
        let mut sr = StatusReport::new();
        BloodLustRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Blood Lust Roll [ 4 ]"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Vampire"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" resists the Blood Lust."));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("Succeeded on a roll of 2+"));
    }

    #[test]
    fn failure_prints_feed_warning_and_needed_roll() {
        let game = make_game();
        let report = ReportBloodLustRoll::new(None, false, 1, 2, false, vec![]);
        let mut sr = StatusReport::new();
        BloodLustRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" gives in to the Blood Lust."));
        assert_eq!(
            sr.rendered_runs[5].text.as_deref(),
            Some("Player must feed at the end of the action or lose tackle zone (and drop ball if carrying) and suffer a turnover.")
        );
        assert_eq!(sr.rendered_runs[7].text.as_deref(), Some("Roll a 2+ to succeed"));
    }

    #[test]
    fn re_rolled_skips_needed_roll() {
        let game = make_game();
        let report = ReportBloodLustRoll::new(None, true, 4, 2, true, vec![]);
        let mut sr = StatusReport::new();
        BloodLustRollMessage.render(&mut sr, &game, &report);
        // roll println (2) + player print (1) + resists println (2) = 5, no needed-roll line.
        assert_eq!(sr.rendered_runs.len(), 5);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BloodLustRollMessage.report_id(), ReportId::BLOOD_LUST_ROLL);
        assert_eq!(BloodLustRollMessage.get_key(), "bloodLustRoll");
    }
}
