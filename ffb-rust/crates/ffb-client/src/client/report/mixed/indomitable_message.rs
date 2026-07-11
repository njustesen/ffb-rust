use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_indomitable::ReportIndomitable;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `IndomitableMessage.java`.
pub struct IndomitableMessage;

impl ReportMessage for IndomitableMessage {
    type Report = ReportIndomitable;

    fn report_id(&self) -> ReportId {
        ReportId::INDOMITABLE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, false, player);

        status_report.print_indent(indent + 1, " uses Indomitable to push ");
        // java: `player.getPlayerGender().getGenitive()` assumes the player is non-null.
        if let Some(player) = player {
            status_report.print_indent(indent + 1, player.gender.genitive());
        }
        status_report.print_indent(indent + 1, " strength to the double of ");

        let defender = report.get_defender_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent, false, defender);

        status_report.print_indent(indent + 1, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str, players: Vec<Player>) -> Team {
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
        Game::new(
            empty_team(
                "home",
                vec![Player { id: "p1".into(), name: "Bruiser".into(), gender: PlayerGender::Male, ..Default::default() }],
            ),
            empty_team(
                "away",
                vec![Player { id: "d1".into(), name: "Target".into(), gender: PlayerGender::Female, ..Default::default() }],
            ),
            Rules::Bb2020,
        )
    }

    #[test]
    fn renders_player_gender_and_defender() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportIndomitable::new(Some("p1".into()), Some("d1".into()));
        IndomitableMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Bruiser"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" uses Indomitable to push "));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some(PlayerGender::Male.genitive()));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" strength to the double of "));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some("Target"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("."));
    }

    #[test]
    fn skips_gender_run_when_player_missing() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportIndomitable::new(Some("missing".into()), Some("d1".into()));
        IndomitableMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some(" uses Indomitable to push "));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" strength to the double of "));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Target"));
    }

    #[test]
    fn skips_defender_run_when_defender_missing() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportIndomitable::new(Some("p1".into()), Some("missing".into()));
        IndomitableMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Bruiser"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" strength to the double of "));
        assert_eq!(sr.rendered_runs[4].text.as_deref(), Some("."));
    }
}
