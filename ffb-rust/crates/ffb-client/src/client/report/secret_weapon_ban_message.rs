use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::team::Team;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_secret_weapon_ban::ReportSecretWeaponBan;

pub struct SecretWeaponBanMessage;

impl ReportMessage for SecretWeaponBanMessage {
    type Report = ReportSecretWeaponBan;

    fn report_id(&self) -> ReportId {
        ReportId::SECRET_WEAPON_BAN
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        report_secret_weapon_ban(status_report, game, report, &game.team_home);
        report_secret_weapon_ban(status_report, game, report, &game.team_away);
    }
}

fn report_secret_weapon_ban(status_report: &mut StatusReport, game: &Game, report: &ReportSecretWeaponBan, team: &Team) {
    let player_ids = report.get_player_ids();
    if !player_ids.is_empty() {
        let rolls = report.get_rolls();
        let banned = report.get_bans();
        let indent = status_report.get_indent();
        for i in 0..player_ids.len() {
            let player = game.player(&player_ids[i]);
            if let Some(player) = player {
                if team.has_player(&player.id) {
                    if banned[i] {
                        status_report.print_indent(indent, "The ref bans ");
                        print_player(status_report, game, indent, false, Some(player));
                        status_report.println_indent(indent, " for using a Secret Weapon.");
                    } else {
                        status_report.print_indent(indent, "The ref overlooks ");
                        print_player(status_report, game, indent, false, Some(player));
                        status_report.println_indent(indent, " using a Secret Weapon.");
                    }
                    let secret_weapon_value = player.get_skill_int_value(NamedProperties::GETS_SENT_OFF_AT_END_OF_DRIVE);
                    if rolls[i] > 0 {
                        let penalty = format!(
                            "Penalty roll was {}, banned on a {}+",
                            rolls[i], secret_weapon_value
                        );
                        status_report.println_indent_style(indent + 1, TextStyle::NEEDED_ROLL, &penalty);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;

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

    fn add_player(game: &mut Game, home: bool, id: &str, name: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = name.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(SecretWeaponBanMessage.report_id(), ReportId::SECRET_WEAPON_BAN);
    }

    #[test]
    fn banned_player_renders_ban_message_and_penalty() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", "Ripper");
        let mut report = ReportSecretWeaponBan::new();
        report.add("p1".into(), 3, true);
        let mut status_report = StatusReport::new();
        SecretWeaponBanMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("The ref bans ".to_string())));
        assert!(texts.contains(&Some(" for using a Secret Weapon.".to_string())));
        assert!(texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Penalty roll was 3"))));
    }

    #[test]
    fn overlooked_player_renders_overlook_message() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", "Ripper");
        let mut report = ReportSecretWeaponBan::new();
        report.add("p1".into(), 0, false);
        let mut status_report = StatusReport::new();
        SecretWeaponBanMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("The ref overlooks ".to_string())));
        assert!(texts.contains(&Some(" using a Secret Weapon.".to_string())));
        // roll <= 0 means no penalty line
        assert!(!texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Penalty roll was"))));
    }

    #[test]
    fn empty_player_ids_renders_nothing() {
        let game = make_game();
        let report = ReportSecretWeaponBan::new();
        let mut status_report = StatusReport::new();
        SecretWeaponBanMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn player_on_other_team_is_skipped() {
        let mut game = make_game();
        add_player(&mut game, false, "p1", "Away Guy");
        let mut report = ReportSecretWeaponBan::new();
        report.add("p1".into(), 3, true);
        let mut status_report = StatusReport::new();
        // Only reported once (for away team) — home team pass should skip it.
        SecretWeaponBanMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        let ban_count = texts.iter().filter(|t| t.as_deref() == Some("The ref bans ")).count();
        assert_eq!(ban_count, 1);
    }
}
