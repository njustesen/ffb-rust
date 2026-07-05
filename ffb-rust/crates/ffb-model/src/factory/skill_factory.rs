/// 1:1 translation of com.fumbbl.ffb.factory.SkillFactory.
///
/// Java used a classpath-scanning `Scanner<Skill>` to discover all subclasses of `Skill`
/// and register them by their `getName()` (lowercased) and their `Class<?>`.  Rust has
/// no runtime reflection, so we build a static `HashMap<String, SkillId>` at construction
/// time by iterating every known `SkillId` variant and inserting its Java class simple name.
///
/// The two lookup methods mirror the Java API:
/// * `for_name(name)`       – Java `forName(String)`: case-insensitive display-name lookup,
///                            with the same "Ball & Chain" / "Ball &amp; Chain" alias.
/// * `for_class_name(name)` – exact Java class simple-name lookup (used during JSON
///                            deserialization where the wire format carries the class name).
use std::collections::HashMap;

use crate::enums::SkillId;

/// All `SkillId` variants in declaration order.  Every new variant added to the enum
/// **must** also appear here so the factory map stays complete.
const ALL_SKILL_IDS: &[SkillId] = &[
    // ── Common (all editions) ───────────────────────────────────────────────
    SkillId::Block,
    SkillId::Catch,
    SkillId::Dauntless,
    SkillId::DisturbingPresence,
    SkillId::DivingCatch,
    SkillId::DumpOff,
    SkillId::ExtraArms,
    SkillId::Fend,
    SkillId::FoulAppearance,
    SkillId::HailMaryPass,
    SkillId::Horns,
    SkillId::JumpUp,
    SkillId::MovementIncrease,
    SkillId::Pass,
    SkillId::Sprint,
    SkillId::StandFirm,
    SkillId::StripBall,
    SkillId::SureHands,
    SkillId::Tackle,
    SkillId::Tentacles,
    SkillId::ThickSkull,
    SkillId::TwoHeads,
    SkillId::Wrestle,
    // ── BB2020 & BB2025 skills ──────────────────────────────────────────────
    SkillId::AnimalSavagery,
    SkillId::Animosity,
    SkillId::BallAndChain,
    SkillId::Bombardier,
    SkillId::BoneHead,
    SkillId::Brawler,
    SkillId::BreakTackle,
    SkillId::BreatheFire,
    SkillId::Chainsaw,
    SkillId::CloudBurster,
    SkillId::Defensive,
    SkillId::DirtyPlayer,
    SkillId::Fumblerooskie,
    SkillId::HitAndRun,
    SkillId::HypnoticGaze,
    SkillId::Leap,
    SkillId::MightyBlow,
    SkillId::MonstrousMouth,
    SkillId::NoHands,
    SkillId::PassingIncrease,
    SkillId::PileDriver,
    SkillId::PilingOn,
    SkillId::PogoStick,
    SkillId::ProjectileVomit,
    SkillId::ReallyStupid,
    SkillId::Regeneration,
    SkillId::RightStuff,
    SkillId::RunningPass,
    SkillId::Shadowing,
    SkillId::SideStep,
    SkillId::SneakyGit,
    SkillId::Stab,
    SkillId::StrengthIncrease,
    SkillId::SureFeet,
    SkillId::Swarming,
    SkillId::Swoop,
    SkillId::VeryLongLegs,
    // ── BB2020 special skills ───────────────────────────────────────────────
    SkillId::ASneakyPair,
    SkillId::BlastIt,
    SkillId::BrutalBlock,
    SkillId::BurstOfSpeed,
    SkillId::ConsummateProfessional,
    SkillId::DwarfenScourge,
    SkillId::ExcuseMeAreYouAZoat,
    SkillId::FrenziedRush,
    SkillId::GhostlyFlames,
    SkillId::Incorporeal,
    SkillId::LordOfChaos,
    SkillId::MasterAssassin,
    SkillId::MesmerizingDance,
    SkillId::PumpUpTheCrowd,
    SkillId::PutridRegurgitation,
    SkillId::TheBallista,
    SkillId::ThenIStartedBlastin,
    SkillId::TwoForOne,
    SkillId::WhirlingDervish,
    SkillId::WisdomOfTheWhiteDwarf,
    SkillId::Yoink,
    // ── Mixed (multiple editions) ──────────────────────────────────────────
    SkillId::ArmBar,
    SkillId::BalefulHex,
    SkillId::Cannoneer,
    SkillId::IronHardSkin,
    SkillId::LookIntoMyEyes,
    SkillId::MyBall,
    SkillId::PickMeUp,
    SkillId::SafePass,
    SkillId::SafePairOfHands,
    SkillId::Slayer,
    SkillId::ToxinConnoisseur,
    SkillId::UnchannelledFury,
    // ── BB2016-only skills ──────────────────────────────────────────────────
    SkillId::Accurate,
    SkillId::AlwaysHungry,
    SkillId::ArmourIncrease,
    SkillId::BloodLust,
    SkillId::Claw,
    SkillId::Decay,
    SkillId::Disposable,
    SkillId::DivingTackle,
    SkillId::FanFavourite,
    SkillId::Frenzy,
    SkillId::Grab,
    SkillId::Guard,
    SkillId::KickOffReturn,
    SkillId::KickTeamMate,
    SkillId::Loner,
    SkillId::MultipleBlock,
    SkillId::NervesOfSteel,
    SkillId::NurglesRot,
    SkillId::PassBlock,
    SkillId::PrehensileTail,
    SkillId::SafeThrow,
    SkillId::SecretWeapon,
    SkillId::Stakes,
    SkillId::StrongArm,
    SkillId::Stunty,
    SkillId::TakeRoot,
    SkillId::ThrowTeamMate,
    SkillId::Timmmber,
    SkillId::Titchy,
    SkillId::WeepingDagger,
    SkillId::WildAnimal,
    // ── BB2025-only skills ──────────────────────────────────────────────────
    SkillId::AgilityIncrease,
    SkillId::BigHand,
    SkillId::Bullseye,
    SkillId::Dodge,
    SkillId::EyeGouge,
    SkillId::Fumblerooski,
    SkillId::GiveAndGo,
    SkillId::Hatred,
    SkillId::Insignificant,
    SkillId::Juggernaut,
    SkillId::Kick,
    SkillId::Leader,
    SkillId::LethalFlight,
    SkillId::LoneFouler,
    SkillId::NoBall,
    SkillId::OnTheBall,
    SkillId::Pogo,
    SkillId::Pro,
    SkillId::Punt,
    SkillId::PutTheBootIn,
    SkillId::QuickFoul,
    SkillId::Saboteur,
    SkillId::Sidestep,
    SkillId::BlastinSolvesEverything,
    SkillId::DwarvenScourge,
    SkillId::KrumpAndSmash,
    SkillId::MesmerisingDance,
    SkillId::SlashingNails,
    SkillId::SteadyFooting,
    SkillId::TeamCaptain,
    SkillId::Taunt,
    SkillId::Unsteady,
    SkillId::ViolentInnovator,
    SkillId::WoodlandFury,
    SkillId::WorkingInTandem,
    // ── Mixed/special star player traits (BB2020 + BB2025) ────────────────────
    SkillId::AllYouCanEat,
    SkillId::BeerBarrelBash,
    SkillId::BlackInk,
    SkillId::BlindRage,
    SkillId::BoundingLeap,
    SkillId::BugmansXXXXXX,
    SkillId::CatchOfTheDay,
    SkillId::CrushingBlow,
    SkillId::Drunkard,
    SkillId::FuriousOutburst,
    SkillId::FuryOfTheBloodGod,
    SkillId::GoredByTheBull,
    SkillId::HalflingLuck,
    SkillId::IllBeBack,
    SkillId::Indomitable,
    SkillId::Kaboom,
    SkillId::KeenPlayer,
    SkillId::KickEmWhileTheyReDown,
    SkillId::MaximumCarnage,
    SkillId::OldPro,
    SkillId::PlagueRidden,
    SkillId::PrimalSavagery,
    SkillId::QuickBite,
    SkillId::RaidingParty,
    SkillId::Ram,
    SkillId::Reliable,
    SkillId::SavageBlow,
    SkillId::SavageMauling,
    SkillId::ShotToNothing,
    SkillId::SneakiestOfTheLot,
    SkillId::StarOfTheShow,
    SkillId::StrongPassingGame,
    SkillId::SwiftAsTheBreeze,
    SkillId::TastyMorsel,
    SkillId::TheFlashingBlade,
    SkillId::ThinkingMansTroll,
    SkillId::Treacherous,
    SkillId::Trickster,
    SkillId::UnstoppableMomentum,
    SkillId::ViciousVines,
    SkillId::WatchOut,
];

pub struct SkillFactory {
    /// Keyed by the exact Java class simple name, e.g. "Block", "SideStep".
    /// Built from `SkillId::class_name()` at construction time.
    skills: HashMap<String, SkillId>,
}

impl SkillFactory {
    pub fn new() -> Self {
        let mut skills = HashMap::with_capacity(ALL_SKILL_IDS.len());
        for &id in ALL_SKILL_IDS {
            skills.insert(id.class_name().to_string(), id);
        }
        // Java alias: "Ball & Chain" / "Ball &amp; Chain" → BallAndChain.
        // `for_name` handles this alias via `from_class_name`; we also add it
        // to the class-name map for completeness so callers using either key
        // get the right result.
        skills.insert("Ball & Chain".to_string(), SkillId::BallAndChain);
        skills.insert("Ball &amp; Chain".to_string(), SkillId::BallAndChain);
        Self { skills }
    }

    /// Java: `SkillFactory.forName(String)`.
    ///
    /// Case-insensitive lookup by display name (the value returned by `Skill.getName()`
    /// in Java, which is usually the human-readable skill name).  Delegates to
    /// `SkillId::from_class_name` which normalises the input identically to the Java
    /// factory (strips non-alphanumeric characters, lowercases).
    ///
    /// Special aliases (Java source):
    /// ```text
    /// if ("Ball & Chain".equalsIgnoreCase(name) || "Ball &amp; Chain".equalsIgnoreCase(name))
    ///     return skills.get("ball and chain");
    /// ```
    pub fn for_name(&self, name: &str) -> Option<SkillId> {
        // Handle the explicit Java alias before the generic normalisation path.
        if name.eq_ignore_ascii_case("Ball & Chain") || name.eq_ignore_ascii_case("Ball &amp; Chain") {
            return Some(SkillId::BallAndChain);
        }
        SkillId::from_class_name(name)
    }

    /// Exact Java class simple-name lookup, e.g. `"Block"` → `Some(SkillId::Block)`.
    ///
    /// Used during JSON/network deserialization where the wire format carries the
    /// unqualified Java class name.  Unlike `for_name`, this is case-sensitive and
    /// does **not** normalise the input.
    pub fn for_class_name(&self, name: &str) -> Option<SkillId> {
        self.skills.get(name).copied()
    }

    /// Returns an iterator over every registered `SkillId`.
    ///
    /// Mirrors `SkillFactory.getSkills()` which returns `skills.values()`.
    pub fn get_skills(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.skills.values().copied()
    }

    /// Total number of registered skills.
    pub fn len(&self) -> usize {
        // Subtract the two alias entries that map to BallAndChain.
        self.skills.len() - 2
    }

    /// Returns `true` when the factory holds no skills (should never be true in practice).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SkillFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory() -> SkillFactory {
        SkillFactory::new()
    }

    // ── for_class_name: exact Java class-name lookups ──────────────────────

    #[test]
    fn for_class_name_block() {
        assert_eq!(factory().for_class_name("Block"), Some(SkillId::Block));
    }

    #[test]
    fn for_class_name_dodge() {
        assert_eq!(factory().for_class_name("Dodge"), Some(SkillId::Dodge));
    }

    #[test]
    fn for_class_name_mighty_blow() {
        assert_eq!(factory().for_class_name("MightyBlow"), Some(SkillId::MightyBlow));
    }

    #[test]
    fn for_class_name_ball_and_chain() {
        assert_eq!(factory().for_class_name("BallAndChain"), Some(SkillId::BallAndChain));
    }

    #[test]
    fn for_class_name_side_step_bb2020() {
        // BB2020 variant has uppercase S in both words.
        assert_eq!(factory().for_class_name("SideStep"), Some(SkillId::SideStep));
    }

    #[test]
    fn for_class_name_sidestep_bb2025() {
        // BB2025 variant is all lowercase second word.
        assert_eq!(factory().for_class_name("Sidestep"), Some(SkillId::Sidestep));
    }

    #[test]
    fn for_class_name_regeneration() {
        assert_eq!(factory().for_class_name("Regeneration"), Some(SkillId::Regeneration));
    }

    #[test]
    fn for_class_name_secret_weapon() {
        assert_eq!(factory().for_class_name("SecretWeapon"), Some(SkillId::SecretWeapon));
    }

    #[test]
    fn for_class_name_pogo_stick() {
        assert_eq!(factory().for_class_name("PogoStick"), Some(SkillId::PogoStick));
    }

    #[test]
    fn for_class_name_pogo_bb2025() {
        assert_eq!(factory().for_class_name("Pogo"), Some(SkillId::Pogo));
    }

    #[test]
    fn for_class_name_hypnotic_gaze() {
        assert_eq!(factory().for_class_name("HypnoticGaze"), Some(SkillId::HypnoticGaze));
    }

    #[test]
    fn for_class_name_unknown_returns_none() {
        assert_eq!(factory().for_class_name("NonExistentSkill"), None);
    }

    #[test]
    fn for_class_name_empty_string_returns_none() {
        assert_eq!(factory().for_class_name(""), None);
    }

    #[test]
    fn for_class_name_wrong_case_returns_none() {
        // for_class_name is case-sensitive (unlike for_name).
        assert_eq!(factory().for_class_name("block"), None);
        assert_eq!(factory().for_class_name("BLOCK"), None);
    }

    // ── for_class_name: alias entries for Ball & Chain ────────────────────

    #[test]
    fn for_class_name_ball_ampersand_chain_alias() {
        assert_eq!(factory().for_class_name("Ball & Chain"), Some(SkillId::BallAndChain));
    }

    #[test]
    fn for_class_name_ball_amp_entity_alias() {
        assert_eq!(factory().for_class_name("Ball &amp; Chain"), Some(SkillId::BallAndChain));
    }

    // ── for_name: case-insensitive / normalised lookups ───────────────────

    #[test]
    fn for_name_ball_and_chain_alias() {
        assert_eq!(factory().for_name("Ball & Chain"), Some(SkillId::BallAndChain));
        assert_eq!(factory().for_name("ball & chain"), Some(SkillId::BallAndChain));
        assert_eq!(factory().for_name("Ball &amp; Chain"), Some(SkillId::BallAndChain));
    }

    #[test]
    fn for_name_block_lowercase() {
        // from_class_name normalises to lowercase.
        assert_eq!(factory().for_name("block"), Some(SkillId::Block));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(factory().for_name("no such skill"), None);
    }

    // ── round-trip: every variant reachable via for_class_name ───────────

    #[test]
    fn all_variants_round_trip_via_for_class_name() {
        let factory = factory();
        for &id in ALL_SKILL_IDS {
            let name = id.class_name();
            assert_eq!(
                factory.for_class_name(name),
                Some(id),
                "for_class_name(\"{name}\") did not return {id:?}"
            );
        }
    }

    // ── coverage / count ──────────────────────────────────────────────────

    #[test]
    fn skill_count_matches_all_skill_ids_slice() {
        let factory = factory();
        // Every entry in ALL_SKILL_IDS must appear in the map exactly once
        // (the alias entries are excluded by `len()`).
        assert_eq!(factory.len(), ALL_SKILL_IDS.len());
    }

    #[test]
    fn get_skills_returns_all_variants() {
        let factory = factory();
        let mut found: Vec<SkillId> = factory.get_skills().collect();
        found.sort_by_key(|id| id.class_name());
        // Just check the count; order is HashMap-dependent.
        // +2 because the HashMap contains the two Ball & Chain alias entries.
        assert_eq!(found.len(), ALL_SKILL_IDS.len() + 2);
    }
}
