use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BriberyAndCorruptionReRollMessage.java`.
pub struct BriberyAndCorruptionReRollMessage;

impl ReportMessage for BriberyAndCorruptionReRollMessage {
    type Report = ReportBriberyAndCorruptionReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let team_id = report.get_team_id().unwrap_or_default();
        let team = game.team_by_id(team_id);
        let is_home = team.is_some_and(|t| t.id == game.team_home.id);
        let team_style = if is_home { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        let team_name = team.map(|t| t.name.clone()).unwrap_or_default();

        status_report.print_indent_style(indent, team_style, &team_name);

        // java: BriberyAndCorruptionAction enum constants: USED, ADDED, WASTED (verified
        // against ffb-java BriberyAndCorruptionAction.java).
        match report.get_action() {
            "USED" => {
                status_report.println_indent(indent, " use Bribery and Corruption to re-roll their Argue the Call roll.");
            }
            "ADDED" => {
                status_report.println_indent(
                    indent,
                    " may re-roll a natural 1 on an Argue the Call roll once in this game due to Bribery and Corruption.",
                );
            }
            "WASTED" => {
                status_report.println_indent(
                    indent,
                    " have no use for their Bribery and Corruption as the coach was banned for more than one argue.",
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", "Home Team"), make_team("away", "Away Team"), Rules::Bb2020)
    }

    #[test]
    fn used_action_prints_used_message() {
        let game = make_game();
        let report = ReportBriberyAndCorruptionReRoll::new(Some("home".into()), "USED".into());
        let mut sr = StatusReport::new();
        BriberyAndCorruptionReRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Home Team"));
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" use Bribery and Corruption to re-roll their Argue the Call roll."));
    }

    #[test]
    fn added_action_prints_added_message() {
        let game = make_game();
        let report = ReportBriberyAndCorruptionReRoll::new(Some("away".into()), "ADDED".into());
        let mut sr = StatusReport::new();
        BriberyAndCorruptionReRollMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::AWAY_BOLD));
        assert_eq!(
            sr.rendered_runs[1].text.as_deref(),
            Some(" may re-roll a natural 1 on an Argue the Call roll once in this game due to Bribery and Corruption.")
        );
    }

    #[test]
    fn wasted_action_prints_wasted_message() {
        let game = make_game();
        let report = ReportBriberyAndCorruptionReRoll::new(Some("home".into()), "WASTED".into());
        let mut sr = StatusReport::new();
        BriberyAndCorruptionReRollMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[1].text.as_deref(),
            Some(" have no use for their Bribery and Corruption as the coach was banned for more than one argue.")
        );
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BriberyAndCorruptionReRollMessage.report_id(), ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL);
        assert_eq!(BriberyAndCorruptionReRollMessage.get_key(), "briberyAndCorruptionReRoll");
    }
}
