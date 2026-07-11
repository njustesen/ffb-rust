use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::report_card_effect_roll::ReportCardEffectRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CardEffectRollMessage.java`.
///
/// Java: `report.getCard().cardReport(report.getCardEffect(), report.getRoll())` looks up a
/// per-card-subclass roll-table description via `Card.cardReport(CardEffect, int)`. The base
/// `Card` class returns `Optional.empty()` and only specific card subclasses override it with
/// real text; no such per-card override registry exists in the Rust model (`ReportCardEffectRoll`
/// only carries the card's name string, and `ffb-model`/`ffb-mechanics` have no `cardReport`
/// lookup). Since we cannot resolve which override (if any) applies without fabricating data,
/// this renders nothing — the same behavior as the un-overridden base-class default.
pub struct CardEffectRollMessage;

impl ReportMessage for CardEffectRollMessage {
    type Report = ReportCardEffectRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CARD_EFFECT_ROLL
    }

    fn render(&self, _status_report: &mut StatusReport, _game: &Game, _report: &Self::Report) {
        // java: report.getCard().cardReport(report.getCardEffect(), report.getRoll()) -- no
        // reachable per-card override registry in the Rust model; base Card.cardReport()
        // returns Optional.empty(), so nothing is printed (see doc comment above).
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn renders_nothing_when_no_card_effect() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCardEffectRoll::new("DISTRACT".into(), 3);
        CardEffectRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn renders_nothing_when_card_effect_present() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let mut report = ReportCardEffectRoll::new("MADCAP_MUSHROOM_POTION".into(), 5);
        report.set_card_effect(Some("MadCapMushroomPotion".into()));
        CardEffectRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_is_card_effect_roll() {
        assert_eq!(CardEffectRollMessage.report_id(), ReportId::CARD_EFFECT_ROLL);
    }
}
