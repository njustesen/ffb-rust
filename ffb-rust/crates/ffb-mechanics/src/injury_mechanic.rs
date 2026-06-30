use ffb_model::enums::{PlayerState, SendToBoxReason};
use ffb_model::enums::PlayerType;
use ffb_model::model::{Game, Player, RosterPosition, Team, TeamResult};
use ffb_model::util::raise_type::RaiseType;
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.InjuryMechanic.
pub trait InjuryMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::INJURY }

    fn raised_by_nurgle_reason(&self) -> SendToBoxReason;
    fn raised_by_nurgle_message(&self) -> String;
    fn can_raise_infected_players(&self, team: &Team, team_result: &TeamResult, attacker: Option<&Player>, dead_player: &Player) -> bool;
    fn infected_goes_to_reserves(&self) -> bool;
    fn can_raise_dead(&self, team: &Team, team_result: &TeamResult, dead_player: &Player) -> bool;
    fn raised_nurgle_type(&self) -> PlayerType;
    fn can_use_apo(&self, game: &Game, defender: &Player, player_state: PlayerState) -> bool;
    fn raise_positions(&self, team: &Team) -> Vec<RosterPosition>;
    fn raise_type(&self, team: &Team) -> RaiseType;
}
