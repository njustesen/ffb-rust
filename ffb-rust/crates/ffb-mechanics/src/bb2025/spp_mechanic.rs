use std::collections::HashSet;
use ffb_model::model::{PlayerResult, SpecialRule, Team};
use crate::mechanic::{Mechanic, MechanicType};
use crate::spp_mechanic::SppMechanic as SppMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.SppMechanic.
pub struct SppMechanic;

impl Default for SppMechanic {
    fn default() -> Self { SppMechanic }
}

impl Mechanic for SppMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::SPP }
}

impl SppMechanicTrait for SppMechanic {
    fn mvp_spp(&self) -> i32 { 4 }

    fn touchdown_spp(&self, team: &Team) -> i32 {
        if has_brawlin_brutes(team) { 2 } else { 3 }
    }

    fn casualty_spp(&self, team: &Team) -> i32 {
        if has_brawlin_brutes(team) { 3 } else { 2 }
    }

    fn completion_spp(&self) -> i32 { 1 }
    fn interception_spp(&self) -> i32 { 2 }
    fn deflection_spp(&self) -> i32 { 1 }
    fn catch_spp(&self) -> i32 { 1 }
    fn landing_spp(&self) -> i32 { 1 }
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

    fn add_catch(&self, additional_catch_spp_teams: &HashSet<String>, pr: &mut PlayerResult, team_id: &str) {
        if additional_catch_spp_teams.contains(team_id) {
            pr.catches_with_additional_spp += 1;
        }
    }

    fn add_landing(&self, pr: &mut PlayerResult) {
        pr.landings += 1;
    }
}

impl SppMechanic {
    pub fn new() -> Self { SppMechanic }
}

fn has_brawlin_brutes(team: &Team) -> bool {
    team.special_rules.iter().any(|r| r == SpecialRule::BRAWLIN_BRUTES.get_rule_name())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::model::{PlayerResult, Team};

    fn bare_team() -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "C".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
        }
    }

    fn brutes_team() -> Team {
        let mut t = bare_team();
        t.special_rules.push(SpecialRule::BRAWLIN_BRUTES.get_rule_name().to_string());
        t
    }

    #[test]
    fn touchdown_spp_normal_team_is_3() {
        assert_eq!(SppMechanic.touchdown_spp(&bare_team()), 3);
    }

    #[test]
    fn touchdown_spp_brutes_team_is_2() {
        assert_eq!(SppMechanic.touchdown_spp(&brutes_team()), 2);
    }

    #[test]
    fn casualty_spp_brutes_team_is_3() {
        assert_eq!(SppMechanic.casualty_spp(&brutes_team()), 3);
    }

    #[test]
    fn add_landing_increments_landings() {
        let mut pr = PlayerResult::default();
        SppMechanic.add_landing(&mut pr);
        assert_eq!(pr.landings, 1);
    }

    #[test]
    fn add_catch_with_extra_team_increments_extra_catches() {
        let mut pr = PlayerResult::default();
        let mut extra: HashSet<String> = HashSet::new();
        extra.insert("t".to_string());
        SppMechanic.add_catch(&extra, &mut pr, "t");
        assert_eq!(pr.catches_with_additional_spp, 1);
    }
}
