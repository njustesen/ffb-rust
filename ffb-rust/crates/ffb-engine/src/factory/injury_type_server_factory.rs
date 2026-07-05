/// Translation of com.fumbbl.ffb.server.factory.InjuryTypeServerFactory.
///
/// Java: uses reflection (InjuryTypeConstants field scan + Scanner<InjuryTypeServer>)
/// to build name→InjuryType and Class→InjuryTypeServer maps at startup.
///
/// Rust: explicit factory-function registry mapping the Java simple class name
/// (e.g. "Block", "Foul", "Chainsaw") to a constructor for `Box<dyn InjuryTypeServer>`.
///
/// The registry is populated in `initialize()` with all concrete InjuryTypeServer impls.
/// `InjuryTypeCrowd` is abstract and not registered; `InjuryTypeThrowARockStalling` is Rust-only.
use std::collections::HashMap;
use crate::injury::InjuryTypeServer;

type InjuryTypeConstructor = fn() -> Box<dyn InjuryTypeServer>;

pub struct InjuryTypeServerFactory {
    /// Java: Map<String, InjuryType> injuryTypes (keyed by lower-case name)
    registry: HashMap<&'static str, InjuryTypeConstructor>,
}

impl InjuryTypeServerFactory {
    pub fn new() -> Self { Self { registry: HashMap::new() } }

    /// Register a constructor for the given Java simple class name (case-sensitive).
    pub fn register(&mut self, name: &'static str, ctor: InjuryTypeConstructor) {
        self.registry.insert(name, ctor);
    }

    /// Java: forName(String name) — case-insensitive lookup by injury type name.
    /// Returns None if the name is not registered.
    pub fn for_name(&self, name: &str) -> Option<Box<dyn InjuryTypeServer>> {
        self.registry.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, ctor)| ctor())
    }

    /// Java: initialize(Game game) — Scanner populates serverTypes.
    /// Rust: explicit registration using the Java InjuryType.getName() as the key.
    /// The factory is a case-insensitive lookup; keys match Java's lowercase names.
    pub fn initialize(&mut self) {
        use crate::injury::injuryType::{
            injury_type_ball_and_chain::InjuryTypeBallAndChain,
            injury_type_bitten::InjuryTypeBitten,
            injury_type_block::{InjuryTypeBlock, BlockMode},
            injury_type_block_prone::InjuryTypeBlockProne,
            injury_type_block_prone_for_spp::InjuryTypeBlockProneForSpp,
            injury_type_block_stunned::InjuryTypeBlockStunned,
            injury_type_block_stunned_for_spp::InjuryTypeBlockStunnedForSpp,
            injury_type_bomb::InjuryTypeBomb,
            injury_type_bomb_with_modifier::InjuryTypeBombWithModifier,
            injury_type_bomb_with_modifier_for_spp::InjuryTypeBombWithModifierForSpp,
            injury_type_breathe_fire::InjuryTypeBreatheFire,
            injury_type_breathe_fire_for_spp::InjuryTypeBreatheFireForSpp,
            injury_type_chainsaw::InjuryTypeChainsaw,
            injury_type_chainsaw_for_spp::InjuryTypeChainsawForSpp,
            injury_type_crowd_push::InjuryTypeCrowdPush,
            injury_type_crowd_push_for_spp::InjuryTypeCrowdPushForSpp,
            injury_type_drop_dodge::InjuryTypeDropDodge,
            injury_type_drop_dodge_for_spp::InjuryTypeDropDodgeForSpp,
            injury_type_drop_gfi::InjuryTypeDropGFI,
            injury_type_drop_jump::InjuryTypeDropJump,
            injury_type_eat_player::InjuryTypeEatPlayer,
            injury_type_fireball::InjuryTypeFireball,
            injury_type_foul::InjuryTypeFoul,
            injury_type_foul_for_spp::InjuryTypeFoulForSpp,
            injury_type_fumbled_ktm::InjuryTypeFumbledKtm,
            injury_type_fumbled_ktm_apo_ko::InjuryTypeFumbledKtmApoKo,
            injury_type_keg_hit::InjuryTypeKegHit,
            injury_type_ktm_crowd::InjuryTypeKTMCrowd,
            injury_type_ktm_injury::InjuryTypeKTMInjury,
            injury_type_lightning::InjuryTypeLightning,
            injury_type_piling_on_armour::InjuryTypePilingOnArmour,
            injury_type_piling_on_injury::InjuryTypePilingOnInjury,
            injury_type_piling_on_knocked_out::InjuryTypePilingOnKnockedOut,
            injury_type_projectile_vomit::InjuryTypeProjectileVomit,
            injury_type_quick_bite::InjuryTypeQuickBite,
            injury_type_sabotaged::InjuryTypeSabotaged,
            injury_type_saboteur::InjuryTypeSaboteur,
            injury_type_stab::InjuryTypeStab,
            injury_type_stab_for_spp::InjuryTypeStabForSpp,
            injury_type_then_i_started_blastin::InjuryTypeThenIStartedBlastin,
            injury_type_throw_a_rock::InjuryTypeThrowARock,
            injury_type_throw_a_rock_stalling::InjuryTypeThrowARockStalling,
            injury_type_trap_door_fall::InjuryTypeTrapDoorFall,
            injury_type_trap_door_fall_for_spp::InjuryTypeTrapDoorFallForSpp,
            injury_type_ttm_hit_player::InjuryTypeTTMHitPlayer,
            injury_type_ttm_hit_player_for_spp::InjuryTypeTTMHitPlayerForSpp,
            injury_type_ttm_landing::InjuryTypeTTMLanding,
        };
        // Names match Java InjuryType.getName() (stored lowercase in the Java factory).
        self.register("ballAndChain", || Box::new(InjuryTypeBallAndChain::default()));
        self.register("bitten", || Box::new(InjuryTypeBitten::default()));
        self.register("block", || Box::new(InjuryTypeBlock::new(BlockMode::Regular, true)));
        self.register("blockProne", || Box::new(InjuryTypeBlockProne::default()));
        self.register("blockProneForSpp", || Box::new(InjuryTypeBlockProneForSpp::default()));
        self.register("blockStunned", || Box::new(InjuryTypeBlockStunned::default()));
        self.register("blockStunnedForSpp", || Box::new(InjuryTypeBlockStunnedForSpp::default()));
        self.register("bomb", || Box::new(InjuryTypeBomb::default()));
        self.register("bombForSpp", || Box::new(InjuryTypeBombWithModifierForSpp::default()));
        self.register("breatheFire", || Box::new(InjuryTypeBreatheFire::default()));
        self.register("breatheFireForSpp", || Box::new(InjuryTypeBreatheFireForSpp::default()));
        self.register("chainsaw", || Box::new(InjuryTypeChainsaw::default()));
        self.register("chainsawForSpp", || Box::new(InjuryTypeChainsawForSpp::default()));
        self.register("crowdpush", || Box::new(InjuryTypeCrowdPush::default()));
        self.register("crowdpushForSpp", || Box::new(InjuryTypeCrowdPushForSpp::default()));
        self.register("dropDodge", || Box::new(InjuryTypeDropDodge::default()));
        self.register("dropDodgeForSpp", || Box::new(InjuryTypeDropDodgeForSpp::default()));
        self.register("dropGfi", || Box::new(InjuryTypeDropGFI::default()));
        self.register("dropLeap", || Box::new(InjuryTypeDropJump::default()));
        self.register("eatPlayer", || Box::new(InjuryTypeEatPlayer::default()));
        self.register("fireball", || Box::new(InjuryTypeFireball::default()));
        self.register("foul", || Box::new(InjuryTypeFoul::default()));
        self.register("foulForSpp", || Box::new(InjuryTypeFoulForSpp::default()));
        self.register("foulWithChainsaw", || Box::new(InjuryTypeFoul::new_with_chainsaw(true)));
        self.register("foulForSppWithChainsaw", || Box::new(InjuryTypeFoulForSpp::new_with_chainsaw(true)));
        self.register("kegHit", || Box::new(InjuryTypeKegHit::default()));
        self.register("ktmCrowd", || Box::new(InjuryTypeKTMCrowd::default()));
        self.register("ktmFumbleInjury", || Box::new(InjuryTypeFumbledKtm::default()));
        self.register("ktmFumbleApoKoInjury", || Box::new(InjuryTypeFumbledKtmApoKo::default()));
        self.register("ktmInjury", || Box::new(InjuryTypeKTMInjury::default()));
        self.register("lightning", || Box::new(InjuryTypeLightning::default()));
        self.register("pilingOnArmor", || Box::new(InjuryTypePilingOnArmour::default()));
        self.register("pilingOnInjury", || Box::new(InjuryTypePilingOnInjury::default()));
        self.register("pilingOnKnockedOut", || Box::new(InjuryTypePilingOnKnockedOut::default()));
        self.register("projectileVomit", || Box::new(InjuryTypeProjectileVomit::default()));
        self.register("quickBite", || Box::new(InjuryTypeQuickBite::default()));
        self.register("sabotaged", || Box::new(InjuryTypeSabotaged::default()));
        self.register("saboteur", || Box::new(InjuryTypeSaboteur::default()));
        self.register("stab", || Box::new(InjuryTypeStab::default()));
        self.register("stabForSpp", || Box::new(InjuryTypeStabForSpp::default()));
        self.register("startedBlastin", || Box::new(InjuryTypeThenIStartedBlastin::default()));
        self.register("throwARock", || Box::new(InjuryTypeThrowARock::default()));
        // throwARockStalling is a Rust-only type for stalling scenarios (not in Java constants).
        self.register("throwARockStalling", || Box::new(InjuryTypeThrowARockStalling::default()));
        self.register("trapdoorFall", || Box::new(InjuryTypeTrapDoorFall::default()));
        self.register("trapdoorFallForSpp", || Box::new(InjuryTypeTrapDoorFallForSpp::default()));
        self.register("ttmHitPlayer", || Box::new(InjuryTypeTTMHitPlayer::default()));
        self.register("ttmHitPlayerForSpp", || Box::new(InjuryTypeTTMHitPlayerForSpp::default()));
        self.register("ttmLanding", || Box::new(InjuryTypeTTMLanding::default()));
        // bombForSppWithModifier is an internal Rust type for bomb-with-modifier SPP tracking.
        self.register("bombWithModifier", || Box::new(InjuryTypeBombWithModifier::default()));
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }

    pub fn registered_count(&self) -> usize { self.registry.len() }
}

impl Default for InjuryTypeServerFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use crate::injury::{InjuryContext, InjuryTypeServer};
    use crate::step::framework::test_team;

    struct DummyType { ctx: InjuryContext }
    impl DummyType { fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
    impl InjuryTypeServer for DummyType {
        fn handle_injury(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str, _c: FieldCoordinate, _fc: Option<FieldCoordinate>, _oc: Option<&InjuryContext>, _m: ApothecaryMode) {}
        fn injury_context(&self) -> &InjuryContext { &self.ctx }
        fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
        fn java_class_name(&self) -> &'static str { "Dummy" }
    }

    #[test]
    fn for_name_returns_registered_type() {
        let mut f = InjuryTypeServerFactory::new();
        f.register("Dummy", || Box::new(DummyType::new()));
        assert!(f.for_name("Dummy").is_some());
        assert!(f.for_name("dummy").is_some());
    }

    #[test]
    fn for_name_miss_returns_none() {
        let f = InjuryTypeServerFactory::new();
        assert!(f.for_name("Unknown").is_none());
    }

    #[test]
    fn registered_count_reflects_registrations() {
        let mut f = InjuryTypeServerFactory::new();
        assert_eq!(f.registered_count(), 0);
        f.register("Dummy", || Box::new(DummyType::new()));
        assert_eq!(f.registered_count(), 1);
    }

    #[test]
    fn initialize_registers_expected_types() {
        let mut f = InjuryTypeServerFactory::new();
        f.initialize();
        // 47 Java InjuryTypeConstants + 2 Rust-only extras (throwARockStalling, bombWithModifier)
        assert_eq!(f.registered_count(), 49);
    }

    #[test]
    fn for_name_block_after_initialize() {
        let mut f = InjuryTypeServerFactory::new();
        f.initialize();
        assert!(f.for_name("block").is_some());
        assert!(f.for_name("BLOCK").is_some());
    }

    #[test]
    fn for_name_foul_after_initialize() {
        let mut f = InjuryTypeServerFactory::new();
        f.initialize();
        assert!(f.for_name("foul").is_some());
    }

    #[test]
    fn for_name_piling_on_armor_after_initialize() {
        let mut f = InjuryTypeServerFactory::new();
        f.initialize();
        assert!(f.for_name("pilingOnArmor").is_some());
    }

    #[test]
    fn for_name_unknown_after_initialize_returns_none() {
        let mut f = InjuryTypeServerFactory::new();
        f.initialize();
        assert!(f.for_name("nonExistentInjuryType").is_none());
    }
}
