use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_brilliant_coaching_re_rolls_lost::ReportBrilliantCoachingReRollsLost;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BrilliantCoachingReRollsLostMessage.java`.
pub struct BrilliantCoachingReRollsLostMessage;

impl ReportMessage for BrilliantCoachingReRollsLostMessage {
    type Report = ReportBrilliantCoachingReRollsLost;

    fn report_id(&self) -> ReportId {
        ReportId::BRILLIANT_COACHING_RE_ROLLS_LOST
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let team_id = report.get_team_id().unwrap_or_default();
        let team = game.team_by_id(team_id);
        let is_home = team.is_some_and(|t| t.id == game.team_home.id);
        let team_style = if is_home { TextStyle::HOME } else { TextStyle::AWAY };
        let team_name = team.map(|t| t.name.clone()).unwrap_or_default();

        status_report.print_indent_style(indent + 1, team_style, &team_name);

        let mut builder = String::from(" lose ");
        if report.get_amount() == 1 {
            builder.push_str("1 Brilliant Coaching Re-Roll as it was");
        } else {
            builder.push_str(&report.get_amount().to_string());
            builder.push_str(" Brilliant Coaching Re-Rolls as they were");
        }
        builder.push_str(" not used in this drive.");

        status_report.println_indent_style(indent + 1, TextStyle::NONE, &builder);
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
    fn single_reroll_uses_singular_wording() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportBrilliantCoachingReRollsLost::new(Some("home".into()), 1);
        BrilliantCoachingReRollsLostMessage.render(&mut sr, &game, &report);
        let last_text = sr.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" lose 1 Brilliant Coaching Re-Roll as it was not used in this drive."));
    }

    #[test]
    fn multiple_rerolls_uses_plural_wording() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportBrilliantCoachingReRollsLost::new(Some("away".into()), 3);
        BrilliantCoachingReRollsLostMessage.render(&mut sr, &game, &report);
        let last_text = sr.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" lose 3 Brilliant Coaching Re-Rolls as they were not used in this drive."));
    }

    #[test]
    fn home_team_uses_home_style() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportBrilliantCoachingReRollsLost::new(Some("home".into()), 2);
        BrilliantCoachingReRollsLostMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Home Team"));
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_team_uses_away_style() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportBrilliantCoachingReRollsLost::new(Some("away".into()), 2);
        BrilliantCoachingReRollsLostMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Away Team"));
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BrilliantCoachingReRollsLostMessage.report_id(), ReportId::BRILLIANT_COACHING_RE_ROLLS_LOST);
        assert_eq!(BrilliantCoachingReRollsLostMessage.get_key(), "brilliantCoachingReRoll");
    }
}
