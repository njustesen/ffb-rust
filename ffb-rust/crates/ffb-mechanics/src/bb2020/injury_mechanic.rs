use ffb_model::enums::{PlayerState, PlayerType, SendToBoxReason};
use ffb_model::model::{Game, Player, RosterPosition, SpecialRule, Team, TeamResult};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::raise_type::RaiseType;
use crate::mechanic::{Mechanic, MechanicType};
use crate::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.InjuryMechanic.
pub struct InjuryMechanic;

impl Default for InjuryMechanic {
    fn default() -> Self { InjuryMechanic }
}

impl Mechanic for InjuryMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::INJURY }
}

impl InjuryMechanicTrait for InjuryMechanic {
    fn raised_by_nurgle_reason(&self) -> SendToBoxReason {
        SendToBoxReason::PlagueRidden
    }

    fn raised_by_nurgle_message(&self) -> String {
        " is now Plague Ridden and will join team ".into()
    }

    fn can_raise_infected_players(
        &self,
        team: &Team,
        team_result: &TeamResult,
        attacker: Option<&Player>,
        dead_player: &Player,
    ) -> bool {
        team.special_rules.iter().any(|r| r == SpecialRule::FAVOURED_OF_NURGLE.get_rule_name())
            && team_result.raised_dead == 0
            && attacker.is_some()
            && attacker.unwrap().has_skill_property(NamedProperties::ALLOWS_RAISING_LINEMAN)
            && dead_player.strength_with_modifiers() <= 4
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
            && !dead_player.has_skill_property(NamedProperties::REQUIRES_SECOND_CASUALTY_ROLL)
    }

    fn infected_goes_to_reserves(&self) -> bool { true }

    fn can_raise_dead(
        &self,
        team: &Team,
        team_result: &TeamResult,
        dead_player: &Player,
    ) -> bool {
        // (MASTERS_OF_UNDEATH || roster.hasVampireLord()) && raisedDead == 0
        // && strength <= 4 && !preventRaiseFromDead
        let masters_of_undeath = team.special_rules.iter()
            .any(|r| r == SpecialRule::MASTERS_OF_UNDEATH.get_rule_name());
        (masters_of_undeath || team.vampire_lord)
            && team_result.raised_dead == 0
            && dead_player.strength_with_modifiers() <= 4
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
    }

    fn raised_nurgle_type(&self) -> PlayerType { PlayerType::PlagueRidden }

    fn can_use_apo(&self, game: &Game, defender: &Player, player_state: PlayerState) -> bool {
        use crate::apothecary_mechanic::ApothecaryMechanic as ApoTrait;
        !super::apothecary_mechanic::ApothecaryMechanic::new()
            .apothecary_types(game, defender, player_state)
            .is_empty()
    }

    fn raise_positions(&self, _team: &Team) -> Vec<RosterPosition> { vec![] }

    fn raise_type(&self, team: &Team) -> RaiseType {
        if team.special_rules.iter().any(|r| r == SpecialRule::MASTERS_OF_UNDEATH.get_rule_name()) {
            return RaiseType::ZOMBIE;
        }
        // Java: team.getRoster().hasVampireLord() → THRALL; stored on Team.vampire_lord
        if team.vampire_lord {
            return RaiseType::THRALL;
        }
        RaiseType::ROTTER
    }
}

impl InjuryMechanic {
    pub fn new() -> Self { InjuryMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;

    fn empty_team(special_rules: Vec<String>) -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules, players: vec![],
            vampire_lord: false,
        }
    }

    fn empty_team_result() -> TeamResult {
        TeamResult::default()
    }

    fn bare_player() -> Player {
        Player::default()
    }

    #[test]
    fn can_raise_infected_requires_favoured_of_nurgle() {
        let m = InjuryMechanic::new();
        let team = empty_team(vec![]);
        let tr = empty_team_result();
        let attacker = bare_player();
        let dead = bare_player();
        assert!(!m.can_raise_infected_players(&team, &tr, Some(&attacker), &dead));
    }

    #[test]
    fn can_raise_infected_blocked_if_raised_dead_nonzero() {
        let m = InjuryMechanic::new();
        let team = empty_team(vec![SpecialRule::FAVOURED_OF_NURGLE.get_rule_name().into()]);
        let mut tr = empty_team_result();
        tr.raised_dead = 1;
        let dead = bare_player();
        assert!(!m.can_raise_infected_players(&team, &tr, None, &dead));
    }

    #[test]
    fn infected_goes_to_reserves_true() {
        assert!(InjuryMechanic::new().infected_goes_to_reserves());
    }

    #[test]
    fn raise_type_is_zombie_for_masters_of_undeath() {
        let m = InjuryMechanic::new();
        let team = empty_team(vec![SpecialRule::MASTERS_OF_UNDEATH.get_rule_name().into()]);
        assert_eq!(m.raise_type(&team), RaiseType::ZOMBIE);
    }

    #[test]
    fn raise_type_is_rotter_without_special_rule() {
        let m = InjuryMechanic::new();
        let team = empty_team(vec![]);
        assert_eq!(m.raise_type(&team), RaiseType::ROTTER);
    }

    #[test]
    fn can_use_apo_false_when_no_apothecaries() {
        use ffb_model::enums::{PS_SERIOUS_INJURY, Rules};
        let mut home = empty_team(vec![]);
        let p = bare_player();
        home.players.push(p.clone());
        let game = ffb_model::model::Game::new(home, empty_team(vec![]), Rules::Bb2020);
        assert!(!InjuryMechanic::new().can_use_apo(&game, &p, PlayerState(PS_SERIOUS_INJURY)));
    }

    #[test]
    fn can_use_apo_true_when_team_has_apo() {
        use ffb_model::enums::{PS_SERIOUS_INJURY, Rules};
        let mut home = empty_team(vec![]);
        let p = bare_player();
        home.players.push(p.clone());
        let mut game = ffb_model::model::Game::new(home, empty_team(vec![]), Rules::Bb2020);
        game.turn_data_home.apothecaries = 1;
        assert!(InjuryMechanic::new().can_use_apo(&game, &p, PlayerState(PS_SERIOUS_INJURY)));
    }
}
