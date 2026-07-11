use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::mixed::report_animal_savagery::ReportAnimalSavagery;
use ffb_model::report::report_id::ReportId;

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

        if report.get_defender_id().is_some_and(|s| !s.is_empty()) {
            let defender = report.get_defender_id().and_then(|id| game.player(id));
            status_report.print_indent_style(indent, TextStyle::NONE, " lashes out against ");
            print_player(status_report, game, indent, false, defender);
            if is_option_enabled(game, ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION) {
                let genitive = attacker.map(|p| p.gender.genitive()).unwrap_or("");
                status_report.print_indent_style(indent, TextStyle::NONE, &format!(" but still loses {genitive} action"));
            }
            status_report.println_indent_style(indent, TextStyle::NONE, ".");
        } else {
            let genitive = attacker.map(|p| p.gender.genitive()).unwrap_or("");
            status_report.println_indent_style(
                indent,
                TextStyle::NONE,
                &format!(" has no one to lash out against and thus loses {genitive} action."),
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

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

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
        let home = make_team("home", vec![make_player("a1", "Attacker", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("d1", "Defender", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn lashes_out_against_defender() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportAnimalSavagery::new(Some("a1".into()), Some("d1".into()));
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&" lashes out against ".to_string()));
        assert!(texts.iter().any(|t| t == "."));
    }

    #[test]
    fn no_defender_loses_action() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportAnimalSavagery::new(Some("a1".into()), None);
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("has no one to lash out against")));
        assert!(texts.iter().any(|t| t.contains("his action")));
    }

    #[test]
    fn lash_out_ends_activation_option_appends_text() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.options.set(ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION, "true");
        let report = ReportAnimalSavagery::new(Some("a1".into()), Some("d1".into()));
        AnimalSavageryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("but still loses his action")));
    }

    #[test]
    fn report_id_is_animal_savagery() {
        assert_eq!(AnimalSavageryMessage.report_id(), ReportId::ANIMAL_SAVAGERY);
    }
}
