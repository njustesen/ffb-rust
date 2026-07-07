use std::collections::HashSet;
use ffb_model::model::{PlayerResult, Team};
use crate::mechanic::{Mechanic, MechanicType};
use crate::spp_mechanic::SppMechanic as SppMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.SppMechanic.
pub struct SppMechanic;

impl Default for SppMechanic {
    fn default() -> Self { SppMechanic }
}

impl Mechanic for SppMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SPP }
}

impl SppMechanicTrait for SppMechanic {
    fn mvp_spp(&self) -> i32 { 5 }
    fn touchdown_spp(&self, _team: &Team) -> i32 { 3 }
    fn casualty_spp(&self, _team: &Team) -> i32 { 2 }
    fn completion_spp(&self) -> i32 { 1 }
    fn interception_spp(&self) -> i32 { 2 }
    fn deflection_spp(&self) -> i32 { 1 }
    fn catch_spp(&self) -> i32 { 1 }
    fn landing_spp(&self) -> i32 { 0 }
    fn additional_completion_spp(&self) -> i32 { 0 }
    fn additional_casualty_spp(&self) -> i32 { 0 }
    fn additional_catch_spp(&self) -> i32 { 0 }

    fn add_completion(&self, _additional_completion_spp_teams: &HashSet<String>, pr: &mut PlayerResult, _team_id: &str) {
        pr.completions += 1;
    }

    fn add_casualty(&self, _additional_casualty_spp_teams: &HashSet<String>, pr: &mut PlayerResult, _team_id: &str) {
        pr.casualties += 1;
    }

    fn add_catch(&self, _additional_catch_spp_teams: &HashSet<String>, _pr: &mut PlayerResult, _team_id: &str) {}

    fn add_landing(&self, _pr: &mut PlayerResult) {}
}

impl SppMechanic {
    pub fn new() -> Self { SppMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mvp_spp_is_5() {
        assert_eq!(SppMechanic.mvp_spp(), 5);
    }

    #[test]
    fn touchdown_spp_is_3() {
        use ffb_model::enums::Rules;
        use ffb_model::model::Team;
        let team = Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "C".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
        };
        assert_eq!(SppMechanic.touchdown_spp(&team), 3);
    }

    #[test]
    fn add_completion_increments_completions() {
        use ffb_model::model::PlayerResult;
        let mut pr = PlayerResult::default();
        SppMechanic.add_completion(&Default::default(), &mut pr, "t");
        assert_eq!(pr.completions, 1);
    }

    #[test]
    fn add_casualty_increments_casualties() {
        use ffb_model::model::PlayerResult;
        let mut pr = PlayerResult::default();
        SppMechanic.add_casualty(&Default::default(), &mut pr, "t");
        assert_eq!(pr.casualties, 1);
    }
}
