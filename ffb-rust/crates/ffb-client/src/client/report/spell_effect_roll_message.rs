use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_special_effect_roll::ReportSpecialEffectRoll;

pub struct SpellEffectRollMessage;

impl ReportMessage for SpellEffectRollMessage {
    type Report = ReportSpecialEffectRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SPELL_EFFECT_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let mut status = String::new();
        match report.get_special_effect() {
            SpecialEffect::LIGHTNING => {
                status.push_str(&format!("Lightning Spell Effect Roll [ {} ]", report.get_roll()));
            }
            SpecialEffect::ZAP => {
                status.push_str(&format!("Zap! Spell Effect Roll [ {} ]", report.get_roll()));
            }
            SpecialEffect::FIREBALL => {
                status.push_str(&format!("Fireball Spell Effect Roll [ {} ]", report.get_roll()));
            }
            SpecialEffect::BOMB => {
                status.push_str("Bomb Effect Roll [ ");
                if report.get_roll() > 0 {
                    status.push_str(&report.get_roll().to_string());
                } else {
                    status.push_str("automatic success");
                }
                status.push_str(" ]");
            }
        }
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, false, player);
        if report.is_successful() {
            if report.get_special_effect().is_wizard_spell() {
                status_report.println_indent(indent + 1, " is hit by the spell.");
            } else {
                status_report.println_indent(indent + 1, " is hit by the explosion.");
            }
        } else if report.get_special_effect().is_wizard_spell() {
            status_report.println_indent(indent + 1, " escapes the spell effect.");
        } else {
            status_report.println_indent(indent + 1, " escapes the explosion.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

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

    fn add_player(game: &mut Game, id: &str, name: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = name.to_string();
        game.team_home.players.push(player);
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(SpellEffectRollMessage.report_id(), ReportId::SPELL_EFFECT_ROLL);
    }

    #[test]
    fn lightning_success_hit_by_spell() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Zappy");
        let report = ReportSpecialEffectRoll::new(SpecialEffect::LIGHTNING, Some("p1".into()), 4, true);
        let mut status_report = StatusReport::new();
        SpellEffectRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Lightning Spell Effect Roll [ 4 ]".to_string())));
        assert!(texts.contains(&Some(" is hit by the spell.".to_string())));
    }

    #[test]
    fn bomb_failure_escapes_explosion() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Bommy");
        let report = ReportSpecialEffectRoll::new(SpecialEffect::BOMB, Some("p1".into()), 2, false);
        let mut status_report = StatusReport::new();
        SpellEffectRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Bomb Effect Roll [ 2 ]".to_string())));
        assert!(texts.contains(&Some(" escapes the explosion.".to_string())));
    }

    #[test]
    fn bomb_zero_roll_is_automatic_success() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Bommy");
        let report = ReportSpecialEffectRoll::new(SpecialEffect::BOMB, Some("p1".into()), 0, true);
        let mut status_report = StatusReport::new();
        SpellEffectRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Bomb Effect Roll [ automatic success ]".to_string())));
    }

    #[test]
    fn zap_failure_escapes_spell_effect() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Zappy");
        let report = ReportSpecialEffectRoll::new(SpecialEffect::ZAP, Some("p1".into()), 1, false);
        let mut status_report = StatusReport::new();
        SpellEffectRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Zap! Spell Effect Roll [ 1 ]".to_string())));
        assert!(texts.contains(&Some(" escapes the spell effect.".to_string())));
    }

    #[test]
    fn fireball_success_hit_by_explosion() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Fiery");
        let report = ReportSpecialEffectRoll::new(SpecialEffect::FIREBALL, Some("p1".into()), 6, true);
        let mut status_report = StatusReport::new();
        SpellEffectRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Fireball Spell Effect Roll [ 6 ]".to_string())));
        assert!(texts.contains(&Some(" is hit by the spell.".to_string())));
    }
}
