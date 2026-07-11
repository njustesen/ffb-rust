use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_animal_savagery::ReportAnimalSavagery;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

/// 1:1 translation of `AnimalSavageryMessage.java`.
pub struct AnimalSavageryMessage;

impl ReportMessage for AnimalSavageryMessage {
    type Report = ReportAnimalSavagery;

    fn report_id(&self) -> ReportId {
        ReportId::ANIMAL_SAVAGERY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let attacker = report.get_attacker_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, attacker);
        if string_tool::is_provided(report.get_defender_id()) {
            let defender = report.get_defender_id().and_then(|id| game.player(id));
            status_report.print_indent_style(indent, TextStyle::NONE, " lashes out against ");
            print_player(status_report, game, indent, false, defender);
            status_report.println_indent_style(indent, TextStyle::NONE, ".");
        } else if let Some(attacker) = attacker {
            status_report.println_indent_style(
                indent,
                TextStyle::NONE,
                &format!(
                    " has no one to lash out against and thus loses {} action.",
                    attacker.gender.genitive()
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
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
            make_team("home", vec![make_player("attacker", PlayerGender::Male)]),
            make_team("away", vec![make_player("defender", PlayerGender::Female)]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn lashes_out_against_defender() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportAnimalSavagery::new(Some("attacker".into()), Some("defender".into()));
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Player attacker"));
        assert!(texts.iter().any(|t| t.contains(" lashes out against ")));
        assert!(texts.contains(&"Player defender"));
        assert!(texts.contains(&"."));
    }

    #[test]
    fn no_defender_loses_genitive_action() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportAnimalSavagery::new(Some("attacker".into()), None);
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("loses his action")));
    }

    #[test]
    fn no_defender_uses_female_genitive() {
        let game = Game::new(
            make_team("home", vec![make_player("attacker", PlayerGender::Female)]),
            make_team("away", vec![]),
            Rules::Bb2020,
        );
        let mut status_report = StatusReport::new();
        let report = ReportAnimalSavagery::new(Some("attacker".into()), None);
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("loses her action")));
    }

    #[test]
    fn empty_defender_id_is_treated_as_absent() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportAnimalSavagery::new(Some("attacker".into()), Some("".into()));
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("has no one to lash out against")));
    }
}
