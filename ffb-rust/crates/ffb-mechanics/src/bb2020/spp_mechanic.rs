use std::collections::HashSet;
use ffb_model::model::{PlayerResult, Team};
use crate::mechanic::{Mechanic, MechanicType};
use crate::spp_mechanic::SppMechanic as SppMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.SppMechanic.
pub struct SppMechanic;

impl Default for SppMechanic {
    fn default() -> Self { SppMechanic }
}

impl Mechanic for SppMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SPP }
}

impl SppMechanicTrait for SppMechanic {
    fn mvp_spp(&self) -> i32 { 4 }
    fn touchdown_spp(&self, _team: &Team) -> i32 { 3 }
    fn casualty_spp(&self, _team: &Team) -> i32 { 2 }
    fn completion_spp(&self) -> i32 { 1 }
    fn interception_spp(&self) -> i32 { 2 }
    fn deflection_spp(&self) -> i32 { 1 }
    fn catch_spp(&self) -> i32 { 1 }
    fn landing_spp(&self) -> i32 { 0 }
    fn additional_completion_spp(&self) -> i32 { 1 }
    fn additional_casualty_spp(&self) -> i32 { 1 }
    fn additional_catch_spp(&self) -> i32 { 1 }

    fn add_completion(&self, additional_completion_spp_teams: &HashSet<String>, pr: &mut PlayerResult, team_id: &str) {
        pr.completions += 1;
        if additional_completion_spp_teams.contains(team_id) {
            pr.completions_with_additional_spp += 1;
        }
    }

    fn add_casualty(&self, additional_casualty_spp_teams: &HashSet<String>, pr: &mut PlayerResult, team_id: &str) {
        pr.casualties += 1;
        if additional_casualty_spp_teams.contains(team_id) {
            pr.casualties_with_additional_spp += 1;
        }
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
    use std::collections::HashSet;
    use ffb_model::model::PlayerResult;

    #[test]
    fn mvp_spp_is_4() {
        assert_eq!(SppMechanic.mvp_spp(), 4);
    }

    #[test]
    fn add_completion_with_extra_team_sets_extra_flag() {
        let mut pr = PlayerResult::default();
        let mut extra = HashSet::new();
        extra.insert("t1".to_string());
        SppMechanic.add_completion(&extra, &mut pr, "t1");
        assert_eq!(pr.completions, 1);
        assert_eq!(pr.completions_with_additional_spp, 1);
    }

    #[test]
    fn add_completion_without_extra_team_no_extra_flag() {
        let mut pr = PlayerResult::default();
        SppMechanic.add_completion(&Default::default(), &mut pr, "t1");
        assert_eq!(pr.completions, 1);
        assert_eq!(pr.completions_with_additional_spp, 0);
    }

    #[test]
    fn add_casualty_with_extra_team_sets_extra_flag() {
        let mut pr = PlayerResult::default();
        let mut extra = HashSet::new();
        extra.insert("t1".to_string());
        SppMechanic.add_casualty(&extra, &mut pr, "t1");
        assert_eq!(pr.casualties, 1);
        assert_eq!(pr.casualties_with_additional_spp, 1);
    }

    #[test]
    fn deflection_spp_is_1() {
        assert_eq!(SppMechanic.deflection_spp(), 1);
    }
}
