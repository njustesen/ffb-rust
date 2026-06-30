use std::collections::HashSet;
use ffb_model::model::{PlayerResult, Team};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.SppMechanic.
/// Note: add_* methods take an explicit team_id because Rust PlayerResult has no Player reference.
pub trait SppMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SPP }

    fn mvp_spp(&self) -> i32;
    fn touchdown_spp(&self, team: &Team) -> i32;
    fn casualty_spp(&self, team: &Team) -> i32;
    fn completion_spp(&self) -> i32;
    fn interception_spp(&self) -> i32;
    fn deflection_spp(&self) -> i32;
    fn catch_spp(&self) -> i32;
    fn landing_spp(&self) -> i32;
    fn additional_completion_spp(&self) -> i32;
    fn additional_casualty_spp(&self) -> i32;
    fn additional_catch_spp(&self) -> i32;
    fn add_completion(&self, additional_completion_spp_teams: &HashSet<String>, player_result: &mut PlayerResult, team_id: &str);
    fn add_casualty(&self, additional_casualty_spp_teams: &HashSet<String>, player_result: &mut PlayerResult, team_id: &str);
    fn add_catch(&self, additional_catch_spp_teams: &HashSet<String>, player_result: &mut PlayerResult, team_id: &str);
    fn add_landing(&self, player_result: &mut PlayerResult);
}
