use ffb_model::enums::{PlayerState, PlayerType, SendToBoxReason};
use ffb_model::model::{Game, Player, RosterPosition, Team, TeamResult};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::raise_type::RaiseType;
use crate::mechanic::{Mechanic, MechanicType};
use crate::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.InjuryMechanic.
pub struct InjuryMechanic;

impl Default for InjuryMechanic {
    fn default() -> Self { InjuryMechanic }
}

impl Mechanic for InjuryMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::INJURY }
}

impl InjuryMechanicTrait for InjuryMechanic {
    fn raised_by_nurgle_reason(&self) -> SendToBoxReason {
        SendToBoxReason::NurglesRot
    }

    fn raised_by_nurgle_message(&self) -> String {
        " has been infected with Nurgle's Rot and will join team ".into()
    }

    fn can_raise_infected_players(
        &self,
        _team: &Team,
        _team_result: &TeamResult,
        attacker: Option<&Player>,
        dead_player: &Player,
    ) -> bool {
        // attacker.hasSkillProperty(allowsRaisingLineman) && dead_player.strength <= 4
        // && !dead_player.hasSkillProperty(preventRaiseFromDead)
        // && !dead_player.hasSkillProperty(requiresSecondCasualtyRoll)
        // TODO: has_skill_property is a stub until SkillFactory is translated
        attacker.is_some()
            && attacker.unwrap().has_skill_property(NamedProperties::ALLOWS_RAISING_LINEMAN)
            && dead_player.strength_with_modifiers() <= 4
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
            && !dead_player.has_skill_property(NamedProperties::REQUIRES_SECOND_CASUALTY_ROLL)
    }

    fn infected_goes_to_reserves(&self) -> bool { false }

    fn can_raise_dead(
        &self,
        team: &Team,
        team_result: &TeamResult,
        dead_player: &Player,
    ) -> bool {
        // (roster.hasNecromancer() || roster.hasVampireLord())
        // && raisedDead == 0 && strength <= 4 && !preventRaiseFromDead
        // TODO: Roster.has_necromancer/has_vampire_lord not yet translated
        let _ = team;
        team_result.raised_dead == 0
            && dead_player.strength_with_modifiers() <= 4
            && !dead_player.has_skill_property(NamedProperties::PREVENT_RAISE_FROM_DEAD)
    }

    fn raised_nurgle_type(&self) -> PlayerType { PlayerType::RaisedFromDead }

    fn can_use_apo(&self, game: &Game, defender: &Player, _player_state: PlayerState) -> bool {
        defender.player_type != PlayerType::Star
            && ((game.team_home.has_player(&defender.id) && game.turn_data_home.apothecaries > 0)
                || (game.team_away.has_player(&defender.id) && game.turn_data_away.apothecaries > 0))
    }

    fn raise_positions(&self, _team: &Team) -> Vec<RosterPosition> { vec![] }

    fn raise_type(&self, team: &Team) -> RaiseType {
        // if has_necromancer → ZOMBIE; if has_vampire_lord → THRALL; else ROTTER
        // TODO: Roster.has_necromancer/has_vampire_lord not yet translated
        let _ = team;
        RaiseType::ROTTER
    }
}

impl InjuryMechanic {
    pub fn new() -> Self { InjuryMechanic }
}
