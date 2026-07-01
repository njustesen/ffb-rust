use ffb_model::enums::{PlayerState, PlayerType, SendToBoxReason};
use ffb_model::model::{Game, Player, RosterPosition, SpecialRule, Team, TeamResult};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::raise_type::RaiseType;
use crate::mechanic::{Mechanic, MechanicType};
use crate::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.InjuryMechanic.
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
        _team: &Team,
        _team_result: &TeamResult,
        attacker: Option<&Player>,
        dead_player: &Player,
    ) -> bool {
        // attacker.hasSkillProperty(allowsRaisingLineman) && !dead_player.hasSkillProperty(BIG_GUY keyword)
        // && !UtilCards.hasSkillToCancelProperty(...)
        // TODO: Keyword.BIG_GUY on RosterPosition not yet translated
        attacker.is_some()
            && attacker.unwrap().has_skill_property(NamedProperties::ALLOWS_RAISING_LINEMAN)
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
    }

    fn infected_goes_to_reserves(&self) -> bool { true }

    fn can_raise_dead(
        &self,
        team: &Team,
        team_result: &TeamResult,
        dead_player: &Player,
    ) -> bool {
        team.special_rules.iter().any(|r| r == SpecialRule::MASTERS_OF_UNDEATH.get_rule_name())
            && team_result.raised_dead == 0
            && dead_player.strength_with_modifiers() <= 4
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
    }

    fn raised_nurgle_type(&self) -> PlayerType { PlayerType::PlagueRidden }

    fn can_use_apo(&self, _game: &Game, _defender: &Player, _player_state: PlayerState) -> bool {
        // TODO: ApothecaryType.forPlayer(game, defender, playerState).isEmpty()
        false
    }

    fn raise_positions(&self, _team: &Team) -> Vec<RosterPosition> {
        // TODO: pos.get_keywords().contains(Keyword.LINEMAN) — needs Keyword support on RosterPosition
        vec![]
    }

    fn raise_type(&self, team: &Team) -> RaiseType {
        if team.special_rules.iter().any(|r| r == SpecialRule::MASTERS_OF_UNDEATH.get_rule_name()) {
            RaiseType::ZOMBIE
        } else {
            RaiseType::ROTTER
        }
    }
}

impl InjuryMechanic {
    pub fn new() -> Self { InjuryMechanic }
}
