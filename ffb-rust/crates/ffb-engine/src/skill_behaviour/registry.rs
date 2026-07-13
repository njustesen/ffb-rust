/// 1:1 translation of com.fumbbl.ffb.server.model.SkillFactory (skill registry aspect).
///
/// Java: SkillFactory holds all skill instances; each skill has a getSkillBehaviour()
/// that returns its ISkillBehaviour. The behaviour holds registered StepModifiers.
///
/// Rust: SkillRegistry maps SkillId → SkillBehaviourContainer (which holds StepModifiers).
/// One registry per rules edition, lazily initialized via OnceLock.
///
/// NOTE: Only skills with `register_into` implemented are registered here. As each
/// edition-specific behaviour's `register_into` is ported, add the corresponding call
/// to the relevant build_* function(s).
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

use ffb_model::enums::{Rules, SkillId};
use crate::model::skill_behaviour::SkillBehaviour as SkillBehaviourContainer;

// ── Edition-specific static registries ───────────────────────────────────────

static REGISTRY_BB2025: Lazy<Arc<SkillRegistry>> =
    Lazy::new(|| Arc::new(SkillRegistry::build_bb2025()));

static REGISTRY_BB2020: Lazy<Arc<SkillRegistry>> =
    Lazy::new(|| Arc::new(SkillRegistry::build_bb2020()));

static REGISTRY_BB2016: Lazy<Arc<SkillRegistry>> =
    Lazy::new(|| Arc::new(SkillRegistry::build_bb2016()));

/// Return the singleton registry for a given edition.
/// Java: game.getRules().getSkillFactory()
pub fn registry_for(rules: Rules) -> Arc<SkillRegistry> {
    match rules {
        Rules::Bb2025 | Rules::Common => REGISTRY_BB2025.clone(),
        Rules::Bb2020 => REGISTRY_BB2020.clone(),
        Rules::Bb2016 => REGISTRY_BB2016.clone(),
    }
}

// ── SkillRegistry ─────────────────────────────────────────────────────────────

/// Maps SkillId → SkillBehaviourContainer for one rules edition.
/// Java: the per-skill registration done by SkillBehaviour<T> constructor calls.
pub struct SkillRegistry {
    behaviours: HashMap<SkillId, SkillBehaviourContainer>,
}

impl SkillRegistry {
    /// Empty registry — used for testing and as the base for `build_*` constructors.
    pub fn empty() -> Self {
        Self { behaviours: HashMap::new() }
    }

    /// Register one skill's behaviour container.
    pub fn register(&mut self, skill_id: SkillId, sb: SkillBehaviourContainer) {
        self.behaviours.insert(skill_id, sb);
    }

    /// Look up a skill's behaviour container.
    /// Java: skillFactory.getSkill(id).getSkillBehaviour()
    pub fn get(&self, skill_id: SkillId) -> Option<&SkillBehaviourContainer> {
        self.behaviours.get(&skill_id)
    }

    /// Iterate over all (SkillId, container) pairs.
    pub fn behaviours_iter(&self) -> impl Iterator<Item=(&SkillId, &SkillBehaviourContainer)> {
        self.behaviours.iter()
    }

    /// Number of registered skills.
    pub fn len(&self) -> usize { self.behaviours.len() }

    /// True if no skills are registered.
    pub fn is_empty(&self) -> bool { self.behaviours.is_empty() }

    // ── Edition builders ──────────────────────────────────────────────────────

    /// Build the BB2025 registry.
    /// Java: SkillFactory initialisation for BB2025 rules.
    fn build_bb2025() -> Self {
        use crate::skill_behaviour::common::horns_behaviour::HornsBehaviour;
        use crate::skill_behaviour::bb2025::{
            animal_savagery_behaviour::AnimalSavageryBehaviour,
            animosity_behaviour::AnimosityBehaviour,
            blood_lust_behaviour::BloodLustBehaviour,
            bombardier_behaviour::BombardierBehaviour,
            bone_head_behaviour::BoneHeadBehaviour,
            bullseye_behaviour::BullseyeBehaviour,
            catch_behaviour::CatchBehaviour,
            diving_tackle_behaviour::DivingTackleBehaviour,
            dump_off_behaviour::DumpOffBehaviour,
            eye_gouge_behaviour::EyeGougeBehaviour,
            foul_appearance_behaviour::FoulAppearanceBehaviour,
            grab_behaviour::GrabBehaviour,
            juggernaut_behaviour::JuggernautBehaviour,
            jump_up_behaviour::JumpUpBehaviour,
            monstrous_mouth_behaviour::MonstrousMouthBehaviour,
            pass_behaviour::PassBehaviour,
            really_stupid_behaviour::ReallyStupidBehaviour,
            saboteur_behaviour::SaboteurBehaviour,
            shadowing_behaviour::ShadowingBehaviour,
            sidestep_behaviour::SidestepBehaviour,
            sneaky_git_behaviour::SneakyGitBehaviour,
            stab_behaviour::StabBehaviour,
            stand_firm_behaviour::StandFirmBehaviour,
            swoop_behaviour::SwoopBehaviour,
            take_root_behaviour::TakeRootBehaviour,
            tentacles_behaviour::TentaclesBehaviour,
            the_ballista_behaviour::TheBallistaBehaviour,
            throw_team_mate_behaviour::ThrowTeamMateBehaviour,
            unchannelled_fury_behaviour::UnchannelledFuryBehaviour,
            wild_animal_behaviour::WildAnimalBehaviour,
            wrestle_behaviour::WrestleBehaviour,
        };
        use crate::skill_behaviour::mixed::dauntless_behaviour::DauntlessBehaviour;
        use crate::skill_behaviour::mixed::abstract_dodging_behaviour::AbstractDodgingBehaviour;
        let mut reg = Self::empty();
        // Common (all editions)
        HornsBehaviour::register_into(&mut reg);
        // Dodge-family (mixed AbstractDodgingBehaviour, priority/requireUnusedSkill per Java ctor)
        AbstractDodgingBehaviour::register_into(&mut reg, SkillId::Dodge, 1, false);
        AbstractDodgingBehaviour::register_into(&mut reg, SkillId::WatchOut, 2, true);
        // BB2025 skills
        AnimalSavageryBehaviour::register_into(&mut reg);
        AnimosityBehaviour::register_into(&mut reg);
        BloodLustBehaviour::register_into(&mut reg);
        BombardierBehaviour::register_into(&mut reg);
        BoneHeadBehaviour::register_into(&mut reg);
        BullseyeBehaviour::register_into(&mut reg);
        CatchBehaviour::register_into(&mut reg);
        DivingTackleBehaviour::register_into(&mut reg);
        DumpOffBehaviour::register_into(&mut reg);
        EyeGougeBehaviour::register_into(&mut reg);
        FoulAppearanceBehaviour::register_into(&mut reg);
        GrabBehaviour::register_into(&mut reg);
        JuggernautBehaviour::register_into(&mut reg);
        JumpUpBehaviour::register_into(&mut reg);
        MonstrousMouthBehaviour::register_into(&mut reg);
        PassBehaviour::register_into(&mut reg);
        ReallyStupidBehaviour::register_into(&mut reg);
        SaboteurBehaviour::register_into(&mut reg);
        ShadowingBehaviour::register_into(&mut reg);
        SidestepBehaviour::register_into(&mut reg);
        SneakyGitBehaviour::register_into(&mut reg);
        StabBehaviour::register_into(&mut reg);
        StandFirmBehaviour::register_into(&mut reg);
        SwoopBehaviour::register_into(&mut reg);
        TakeRootBehaviour::register_into(&mut reg);
        TentaclesBehaviour::register_into(&mut reg);
        TheBallistaBehaviour::register_into(&mut reg);
        ThrowTeamMateBehaviour::register_into(&mut reg);
        UnchannelledFuryBehaviour::register_into(&mut reg);
        WildAnimalBehaviour::register_into(&mut reg);
        WrestleBehaviour::register_into(&mut reg);
        // Mixed (shared logic across editions)
        DauntlessBehaviour::register_into(&mut reg);
        reg
    }

    /// Build the BB2020 registry.
    /// Uses BB2020-specific implementations where they differ from BB2025;
    /// falls back to BB2025 impls where the logic is identical.
    fn build_bb2020() -> Self {
        use crate::skill_behaviour::common::horns_behaviour::HornsBehaviour;
        // BB2025 impls reused unchanged for BB2020
        use crate::skill_behaviour::bb2025::{
            animosity_behaviour::AnimosityBehaviour,
            blood_lust_behaviour::BloodLustBehaviour,
            bombardier_behaviour::BombardierBehaviour,
            dump_off_behaviour::DumpOffBehaviour,
            foul_appearance_behaviour::FoulAppearanceBehaviour,
            jump_up_behaviour::JumpUpBehaviour,
            shadowing_behaviour::ShadowingBehaviour,
            stab_behaviour::StabBehaviour,
            take_root_behaviour::TakeRootBehaviour,
            tentacles_behaviour::TentaclesBehaviour,
            wrestle_behaviour::WrestleBehaviour,
        };
        // BB2020-specific implementations
        use crate::skill_behaviour::bb2020::{
            bone_head_behaviour::BoneHeadBehaviour,
            grab_behaviour::GrabBehaviour,
            piling_on_behaviour::PilingOnBehaviour,
            really_stupid_behaviour::ReallyStupidBehaviour,
            side_step_behaviour::SideStepBehaviour,
            stand_firm_behaviour::StandFirmBehaviour,
        };
        use crate::skill_behaviour::mixed::dauntless_behaviour::DauntlessBehaviour;
        use crate::skill_behaviour::mixed::abstract_dodging_behaviour::AbstractDodgingBehaviour;
        let mut reg = Self::empty();
        // Common (all editions)
        HornsBehaviour::register_into(&mut reg);
        // Dodge-family (mixed AbstractDodgingBehaviour, priority/requireUnusedSkill per Java ctor)
        AbstractDodgingBehaviour::register_into(&mut reg, SkillId::Dodge, 1, false);
        AbstractDodgingBehaviour::register_into(&mut reg, SkillId::WatchOut, 2, true);
        // BB2025 impls (identical behaviour in BB2020)
        AnimosityBehaviour::register_into(&mut reg);
        BloodLustBehaviour::register_into(&mut reg);
        BombardierBehaviour::register_into(&mut reg);
        DumpOffBehaviour::register_into(&mut reg);
        FoulAppearanceBehaviour::register_into(&mut reg);
        JumpUpBehaviour::register_into(&mut reg);
        ShadowingBehaviour::register_into(&mut reg);
        StabBehaviour::register_into(&mut reg);
        TakeRootBehaviour::register_into(&mut reg);
        TentaclesBehaviour::register_into(&mut reg);
        WrestleBehaviour::register_into(&mut reg);
        // BB2020-specific behaviours
        BoneHeadBehaviour::register_into(&mut reg);
        GrabBehaviour::register_into(&mut reg);
        PilingOnBehaviour::register_into(&mut reg);
        ReallyStupidBehaviour::register_into(&mut reg);
        SideStepBehaviour::register_into(&mut reg);
        StandFirmBehaviour::register_into(&mut reg);
        // Mixed (shared logic across editions)
        DauntlessBehaviour::register_into(&mut reg);
        reg
    }

    /// Build the BB2016 registry.
    /// Uses BB2016-specific implementations where they differ from BB2025;
    /// falls back to BB2025 impls where the logic is identical.
    fn build_bb2016() -> Self {
        use crate::skill_behaviour::common::horns_behaviour::HornsBehaviour;
        // BB2025 impls reused unchanged for BB2016
        use crate::skill_behaviour::bb2025::{
            animosity_behaviour::AnimosityBehaviour,
            blood_lust_behaviour::BloodLustBehaviour,
            bombardier_behaviour::BombardierBehaviour,
            dump_off_behaviour::DumpOffBehaviour,
            foul_appearance_behaviour::FoulAppearanceBehaviour,
            jump_up_behaviour::JumpUpBehaviour,
            shadowing_behaviour::ShadowingBehaviour,
            stab_behaviour::StabBehaviour,
            take_root_behaviour::TakeRootBehaviour,
            tentacles_behaviour::TentaclesBehaviour,
            wrestle_behaviour::WrestleBehaviour,
        };
        // BB2016-specific implementations
        use crate::skill_behaviour::bb2016::{
            bone_head_behaviour::BoneHeadBehaviour,
            grab_behaviour::GrabBehaviour,
            piling_on_behaviour::PilingOnBehaviour,
            really_stupid_behaviour::ReallyStupidBehaviour,
            side_step_behaviour::SideStepBehaviour,
            stand_firm_behaviour::StandFirmBehaviour,
            wild_animal_behaviour::WildAnimalBehaviour,
        };
        use crate::skill_behaviour::mixed::dauntless_behaviour::DauntlessBehaviour;
        let mut reg = Self::empty();
        // Common (all editions)
        HornsBehaviour::register_into(&mut reg);
        // BB2025 impls (identical behaviour in BB2016)
        AnimosityBehaviour::register_into(&mut reg);
        BloodLustBehaviour::register_into(&mut reg);
        BombardierBehaviour::register_into(&mut reg);
        DumpOffBehaviour::register_into(&mut reg);
        FoulAppearanceBehaviour::register_into(&mut reg);
        JumpUpBehaviour::register_into(&mut reg);
        ShadowingBehaviour::register_into(&mut reg);
        StabBehaviour::register_into(&mut reg);
        TakeRootBehaviour::register_into(&mut reg);
        TentaclesBehaviour::register_into(&mut reg);
        WrestleBehaviour::register_into(&mut reg);
        // BB2016-specific behaviours
        BoneHeadBehaviour::register_into(&mut reg);
        GrabBehaviour::register_into(&mut reg);
        PilingOnBehaviour::register_into(&mut reg);
        ReallyStupidBehaviour::register_into(&mut reg);
        SideStepBehaviour::register_into(&mut reg);
        StandFirmBehaviour::register_into(&mut reg);
        WildAnimalBehaviour::register_into(&mut reg);
        // Mixed (shared logic across editions)
        DauntlessBehaviour::register_into(&mut reg);
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_has_no_entries() {
        let reg = SkillRegistry::empty();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn register_adds_skill() {
        let mut reg = SkillRegistry::empty();
        let sb = SkillBehaviourContainer::new();
        reg.register(SkillId::Horns, sb);
        assert_eq!(reg.len(), 1);
        assert!(reg.get(SkillId::Horns).is_some());
    }

    #[test]
    fn get_missing_skill_returns_none() {
        let reg = SkillRegistry::empty();
        assert!(reg.get(SkillId::Horns).is_none());
    }

    #[test]
    fn behaviours_iter_yields_all_registered() {
        let mut reg = SkillRegistry::empty();
        reg.register(SkillId::Horns, SkillBehaviourContainer::new());
        reg.register(SkillId::Wrestle, SkillBehaviourContainer::new());
        assert_eq!(reg.behaviours_iter().count(), 2);
    }

    #[test]
    fn build_bb2025_is_not_empty() {
        let reg = SkillRegistry::build_bb2025();
        assert!(!reg.is_empty());
    }

    #[test]
    fn build_bb2020_is_not_empty() {
        let reg = SkillRegistry::build_bb2020();
        assert!(!reg.is_empty());
    }

    #[test]
    fn build_bb2016_is_not_empty() {
        let reg = SkillRegistry::build_bb2016();
        assert!(!reg.is_empty());
    }

    #[test]
    fn registry_for_bb2025_returns_same_arc() {
        let a = registry_for(Rules::Bb2025);
        let b = registry_for(Rules::Bb2025);
        assert!(Arc::ptr_eq(&a, &b), "should return the same Arc");
    }

    #[test]
    fn bb2025_registry_contains_horns() {
        let reg = SkillRegistry::build_bb2025();
        assert!(reg.get(SkillId::Horns).is_some());
    }

    #[test]
    fn bb2025_registry_contains_wrestle() {
        let reg = SkillRegistry::build_bb2025();
        assert!(reg.get(SkillId::Wrestle).is_some());
    }

    #[test]
    fn bb2025_registry_contains_bone_head() {
        let reg = SkillRegistry::build_bb2025();
        assert!(reg.get(SkillId::BoneHead).is_some());
    }

    #[test]
    fn bb2025_registry_contains_tentacles() {
        let reg = SkillRegistry::build_bb2025();
        assert!(reg.get(SkillId::Tentacles).is_some());
    }

    #[test]
    fn bb2025_registry_has_thirty_five_entries() {
        let reg = SkillRegistry::build_bb2025();
        assert_eq!(reg.len(), 35, "BB2025 registry should have 35 registered skill entries (33 + Dodge + WatchOut)");
    }

    #[test]
    fn bb2016_registry_has_twenty_entries() {
        let reg = SkillRegistry::build_bb2016();
        assert_eq!(reg.len(), 20, "BB2016 registry should have 20 registered skill entries (17 + Grab + SideStep + StandFirm)");
    }

    #[test]
    fn bb2020_registry_has_twenty_one_entries() {
        let reg = SkillRegistry::build_bb2020();
        assert_eq!(reg.len(), 21, "BB2020 registry should have 21 registered skill entries (18 + Grab + SideStep + StandFirm)");
    }

    #[test]
    fn bb2025_registry_contains_dodge_with_step_modifier() {
        let reg = SkillRegistry::build_bb2025();
        let sb = reg.get(SkillId::Dodge).expect("Dodge must be in BB2025 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2025_registry_contains_watch_out_with_step_modifier() {
        let reg = SkillRegistry::build_bb2025();
        let sb = reg.get(SkillId::WatchOut).expect("WatchOut must be in BB2025 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2020_registry_contains_dodge_and_watch_out() {
        let reg = SkillRegistry::build_bb2020();
        assert!(reg.get(SkillId::Dodge).is_some());
        assert!(reg.get(SkillId::WatchOut).is_some());
    }

    #[test]
    fn bb2020_registry_contains_bone_head_with_step_modifier() {
        let reg = SkillRegistry::build_bb2020();
        let sb = reg.get(SkillId::BoneHead).expect("BoneHead must be in BB2020 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2020_registry_contains_really_stupid_with_step_modifier() {
        let reg = SkillRegistry::build_bb2020();
        let sb = reg.get(SkillId::ReallyStupid).expect("ReallyStupid must be in BB2020 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2020_registry_contains_piling_on() {
        let reg = SkillRegistry::build_bb2020();
        assert!(reg.get(SkillId::PilingOn).is_some(), "PilingOn must be in BB2020 registry");
    }

    #[test]
    fn bb2020_registry_contains_grab_with_step_modifier() {
        let reg = SkillRegistry::build_bb2020();
        let sb = reg.get(SkillId::Grab).expect("Grab must be in BB2020 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2020_registry_contains_side_step_with_step_modifier() {
        let reg = SkillRegistry::build_bb2020();
        let sb = reg.get(SkillId::SideStep).expect("SideStep must be in BB2020 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2020_registry_contains_stand_firm_with_step_modifier() {
        let reg = SkillRegistry::build_bb2020();
        let sb = reg.get(SkillId::StandFirm).expect("StandFirm must be in BB2020 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2016_registry_contains_grab_with_step_modifier() {
        let reg = SkillRegistry::build_bb2016();
        let sb = reg.get(SkillId::Grab).expect("Grab must be in BB2016 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2016_registry_contains_side_step_with_step_modifier() {
        let reg = SkillRegistry::build_bb2016();
        let sb = reg.get(SkillId::SideStep).expect("SideStep must be in BB2016 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn bb2016_registry_contains_stand_firm_with_step_modifier() {
        let reg = SkillRegistry::build_bb2016();
        let sb = reg.get(SkillId::StandFirm).expect("StandFirm must be in BB2016 registry");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }
}
