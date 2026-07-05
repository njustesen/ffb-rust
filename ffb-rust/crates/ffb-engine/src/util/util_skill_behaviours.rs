/// 1:1 translation of com.fumbbl.ffb.server.util.UtilSkillBehaviours.
///
/// Java: uses Scanner<SkillBehaviour> reflection to find and instantiate all concrete
/// SkillBehaviour subclasses for the active game rules, then calls
/// `behaviour.setSkill(skillFactory.forClass(behaviour.skillClass))` on each one.
///
/// Rust: reflection not available; explicit registration replaces the Scanner.
/// The function returns the list of concrete behaviours for the given rules edition.
/// DEFERRED: `setSkill` wiring requires `SkillFactory.for_class()` — not yet ported.
use ffb_model::enums::Rules;
use crate::skill_behaviour::SkillBehaviour;

pub struct UtilSkillBehaviours;

impl UtilSkillBehaviours {
    pub fn new() -> Self { Self }

    /// Java: registerBehaviours(Game, DebugLog).
    /// Returns all concrete SkillBehaviour instances for the given rules edition.
    /// Caller is responsible for calling `setSkill` (DEFERRED: SkillFactory.for_class).
    pub fn register_behaviours(rules: Rules) -> Vec<Box<dyn SkillBehaviour>> {
        use crate::skill_behaviour::{bb2016, bb2020, bb2025, common, mixed};
        let mut behaviours: Vec<Box<dyn SkillBehaviour>> = Vec::new();
        match rules {
            Rules::Bb2016 => {
                behaviours.push(Box::new(bb2016::AgilityIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2016::AnimosityBehaviour::new()));
                behaviours.push(Box::new(bb2016::ArmourIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2016::BloodLustBehaviour::new()));
                behaviours.push(Box::new(bb2016::BombardierBehaviour::new()));
                behaviours.push(Box::new(bb2016::BoneHeadBehaviour::new()));
                behaviours.push(Box::new(bb2016::CatchBehaviour::new()));
                behaviours.push(Box::new(bb2016::DauntlessBehaviour::new()));
                behaviours.push(Box::new(bb2016::DivingTackleBehaviour::new()));
                behaviours.push(Box::new(bb2016::DodgeBehaviour::new()));
                behaviours.push(Box::new(bb2016::DumpOffBehaviour::new()));
                behaviours.push(Box::new(bb2016::FoulAppearanceBehaviour::new()));
                behaviours.push(Box::new(bb2016::GrabBehaviour::new()));
                behaviours.push(Box::new(bb2016::JumpUpBehaviour::new()));
                behaviours.push(Box::new(bb2016::LeapBehaviour::new()));
                behaviours.push(Box::new(bb2016::MonstrousMouthBehaviour::new()));
                behaviours.push(Box::new(bb2016::MovementIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2016::PassBehaviour::new()));
                behaviours.push(Box::new(bb2016::PilingOnBehaviour::new()));
                behaviours.push(Box::new(bb2016::ReallyStupidBehaviour::new()));
                behaviours.push(Box::new(bb2016::SafeThrowBehaviour::new()));
                behaviours.push(Box::new(bb2016::ShadowingBehaviour::new()));
                behaviours.push(Box::new(bb2016::SideStepBehaviour::new()));
                behaviours.push(Box::new(bb2016::SneakyGitBehaviour::new()));
                behaviours.push(Box::new(bb2016::StabBehaviour::new()));
                behaviours.push(Box::new(bb2016::StandFirmBehaviour::new()));
                behaviours.push(Box::new(bb2016::StrengthIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2016::SwarmingBehaviour::new()));
                behaviours.push(Box::new(bb2016::SwoopBehaviour::new()));
                behaviours.push(Box::new(bb2016::TakeRootBehaviour::new()));
                behaviours.push(Box::new(bb2016::TentaclesBehaviour::new()));
                behaviours.push(Box::new(bb2016::ThrowTeamMateBehaviour::new()));
                behaviours.push(Box::new(bb2016::WildAnimalBehaviour::new()));
                behaviours.push(Box::new(bb2016::WrestleBehaviour::new()));
                // Mixed behaviours not present in bb2016 module (shared BB2016 + BB2020)
                behaviours.push(Box::new(mixed::BlindRageBehaviour::new()));
                behaviours.push(Box::new(mixed::CrushingBlowBehaviour::new()));
                behaviours.push(Box::new(mixed::IndomitableBehaviour::new()));
                behaviours.push(Box::new(mixed::JuggernautBehaviour::new()));
                behaviours.push(Box::new(mixed::OldProBehaviour::new()));
                behaviours.push(Box::new(mixed::RamBehaviour::new()));
                behaviours.push(Box::new(mixed::SavageMaulingBehaviour::new()));
                behaviours.push(Box::new(mixed::WatchOutBehaviour::new()));
            }
            Rules::Bb2020 => {
                // AbstractPassBehaviour is abstract in Java — not registered
                behaviours.push(Box::new(bb2020::AgilityIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2020::AnimalSavageryBehaviour::new()));
                behaviours.push(Box::new(bb2020::AnimosityBehaviour::new()));
                behaviours.push(Box::new(bb2020::BloodLustBehaviour::new()));
                behaviours.push(Box::new(bb2020::BombardierBehaviour::new()));
                behaviours.push(Box::new(bb2020::BoneHeadBehaviour::new()));
                behaviours.push(Box::new(bb2020::BrutalBlockBehaviour::new()));
                behaviours.push(Box::new(bb2020::CatchBehaviour::new()));
                behaviours.push(Box::new(bb2020::CloudBursterBehaviour::new()));
                behaviours.push(Box::new(bb2020::DivingTackleBehaviour::new()));
                behaviours.push(Box::new(bb2020::DodgeBehaviour::new()));
                behaviours.push(Box::new(bb2020::DumpOffBehaviour::new()));
                behaviours.push(Box::new(bb2020::DwarfenScourgeBehaviour::new()));
                behaviours.push(Box::new(bb2020::FoulAppearanceBehaviour::new()));
                behaviours.push(Box::new(bb2020::GhostlyFlamesBehaviour::new()));
                behaviours.push(Box::new(bb2020::GrabBehaviour::new()));
                behaviours.push(Box::new(bb2020::MasterAssassinBehaviour::new()));
                behaviours.push(Box::new(bb2020::MonstrousMouthBehaviour::new()));
                behaviours.push(Box::new(bb2020::PassBehaviour::new()));
                behaviours.push(Box::new(bb2020::PassingIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2020::PilingOnBehaviour::new()));
                behaviours.push(Box::new(bb2020::ReallyStupidBehaviour::new()));
                behaviours.push(Box::new(bb2020::ShadowingBehaviour::new()));
                behaviours.push(Box::new(bb2020::SideStepBehaviour::new()));
                behaviours.push(Box::new(bb2020::SlayerBehaviour::new()));
                behaviours.push(Box::new(bb2020::SneakyGitBehaviour::new()));
                behaviours.push(Box::new(bb2020::StabBehaviour::new()));
                behaviours.push(Box::new(bb2020::StandFirmBehaviour::new()));
                behaviours.push(Box::new(bb2020::StrengthIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2020::SwarmingBehaviour::new()));
                behaviours.push(Box::new(bb2020::SwoopBehaviour::new()));
                behaviours.push(Box::new(bb2020::TakeRootBehaviour::new()));
                behaviours.push(Box::new(bb2020::TentaclesBehaviour::new()));
                behaviours.push(Box::new(bb2020::TheBallistaBehaviour::new()));
                behaviours.push(Box::new(bb2020::ThrowTeamMateBehaviour::new()));
                behaviours.push(Box::new(bb2020::ToxinConnoisseurBehaviour::new()));
                behaviours.push(Box::new(bb2020::UnchannelledFuryBehaviour::new()));
                behaviours.push(Box::new(bb2020::WrestleBehaviour::new()));
                // Mixed behaviours not present in bb2020 module (shared BB2016 + BB2020)
                behaviours.push(Box::new(mixed::BlindRageBehaviour::new()));
                behaviours.push(Box::new(mixed::CrushingBlowBehaviour::new()));
                behaviours.push(Box::new(mixed::IndomitableBehaviour::new()));
                behaviours.push(Box::new(mixed::JuggernautBehaviour::new()));
                behaviours.push(Box::new(mixed::OldProBehaviour::new()));
                behaviours.push(Box::new(mixed::RamBehaviour::new()));
                behaviours.push(Box::new(mixed::SavageMaulingBehaviour::new()));
                behaviours.push(Box::new(mixed::WatchOutBehaviour::new()));
            }
            Rules::Bb2025 | _ => {
                // AbstractPassBehaviour is abstract in Java — not registered
                behaviours.push(Box::new(bb2025::AgilityIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2025::AnimalSavageryBehaviour::new()));
                behaviours.push(Box::new(bb2025::AnimosityBehaviour::new()));
                behaviours.push(Box::new(bb2025::BloodLustBehaviour::new()));
                behaviours.push(Box::new(bb2025::BombardierBehaviour::new()));
                behaviours.push(Box::new(bb2025::BoneHeadBehaviour::new()));
                behaviours.push(Box::new(bb2025::BullseyeBehaviour::new()));
                behaviours.push(Box::new(bb2025::CatchBehaviour::new()));
                behaviours.push(Box::new(bb2025::DivingTackleBehaviour::new()));
                behaviours.push(Box::new(bb2025::DodgeBehaviour::new()));
                behaviours.push(Box::new(bb2025::DumpOffBehaviour::new()));
                behaviours.push(Box::new(bb2025::DwarvenScourgeBehaviour::new()));
                behaviours.push(Box::new(bb2025::EyeGougeBehaviour::new()));
                behaviours.push(Box::new(bb2025::FoulAppearanceBehaviour::new()));
                behaviours.push(Box::new(bb2025::GrabBehaviour::new()));
                behaviours.push(Box::new(bb2025::JuggernautBehaviour::new()));
                behaviours.push(Box::new(bb2025::KrumpAndSmashBehaviour::new()));
                behaviours.push(Box::new(bb2025::LoneFoulerBehaviour::new()));
                behaviours.push(Box::new(bb2025::MasterAssassinBehaviour::new()));
                behaviours.push(Box::new(bb2025::MonstrousMouthBehaviour::new()));
                behaviours.push(Box::new(bb2025::PassBehaviour::new()));
                behaviours.push(Box::new(bb2025::PassingIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2025::ReallyStupidBehaviour::new()));
                behaviours.push(Box::new(bb2025::SaboteurBehaviour::new()));
                behaviours.push(Box::new(bb2025::ShadowingBehaviour::new()));
                behaviours.push(Box::new(bb2025::SidestepBehaviour::new()));
                behaviours.push(Box::new(bb2025::SlayerBehaviour::new()));
                behaviours.push(Box::new(bb2025::SneakyGitBehaviour::new()));
                behaviours.push(Box::new(bb2025::StabBehaviour::new()));
                behaviours.push(Box::new(bb2025::StandFirmBehaviour::new()));
                behaviours.push(Box::new(bb2025::StrengthIncreaseBehaviour::new()));
                behaviours.push(Box::new(bb2025::SwoopBehaviour::new()));
                behaviours.push(Box::new(bb2025::TakeRootBehaviour::new()));
                behaviours.push(Box::new(bb2025::TentaclesBehaviour::new()));
                behaviours.push(Box::new(bb2025::TheBallistaBehaviour::new()));
                behaviours.push(Box::new(bb2025::ThrowTeamMateBehaviour::new()));
                behaviours.push(Box::new(bb2025::ToxinConnoisseurBehaviour::new()));
                behaviours.push(Box::new(bb2025::UnchannelledFuryBehaviour::new()));
                behaviours.push(Box::new(bb2025::WrestleBehaviour::new()));
            }
        }
        // HornsBehaviour is shared across all editions
        behaviours.push(Box::new(common::HornsBehaviour::new()));
        behaviours
    }
}

impl Default for UtilSkillBehaviours {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_can_be_created() {
        let _ = UtilSkillBehaviours::new();
    }

    #[test]
    fn bb2025_returns_non_empty_list() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2025);
        assert!(!behaviours.is_empty());
    }

    #[test]
    fn bb2025_includes_horns_and_dodge() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2025);
        let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"DodgeBehaviour"), "missing DodgeBehaviour");
        assert!(names.contains(&"HornsBehaviour"), "missing HornsBehaviour");
    }

    #[test]
    fn bb2016_includes_leap_not_in_bb2025() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2016);
        let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"LeapBehaviour"), "bb2016 missing LeapBehaviour");
    }

    #[test]
    fn bb2020_includes_brutal_block() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2020);
        let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"BrutalBlockBehaviour"));
    }

    #[test]
    fn bb2016_includes_mixed_blind_rage() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2016);
        let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"BlindRageBehaviour"));
    }

    #[test]
    fn bb2025_count_is_at_least_40() {
        let behaviours = UtilSkillBehaviours::register_behaviours(Rules::Bb2025);
        assert!(behaviours.len() >= 40, "expected >= 40, got {}", behaviours.len());
    }

    #[test]
    fn no_abstract_pass_behaviour_registered() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let behaviours = UtilSkillBehaviours::register_behaviours(rules);
            let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
            assert!(!names.contains(&"AbstractPassBehaviour"), "AbstractPassBehaviour should not be registered for {:?}", rules);
        }
    }

    #[test]
    fn all_editions_include_horns() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let behaviours = UtilSkillBehaviours::register_behaviours(rules);
            let names: Vec<&str> = behaviours.iter().map(|b| b.name()).collect();
            assert!(names.contains(&"HornsBehaviour"), "missing HornsBehaviour for {:?}", rules);
        }
    }
}
