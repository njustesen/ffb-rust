use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// ── Skill IDs ─────────────────────────────────────────────────────────────────
// Ordered so the first 64 fit in a u64 bitmask.
// Common skills (shared across rulesets) come first (indices 0..22).
// BB2025-specific skills follow (indices 23..77).
// Team captain / special skills last (indices 78..97).
// Any skill beyond index 63 falls into the BTreeSet overflow.

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum SkillId {
    // ── Common skills (indices 0–22) ──────────────────────────────────────
    Block = 0,
    Catch = 1,
    Dauntless = 2,
    DisturbingPresence = 3,
    DivingCatch = 4,
    DumpOff = 5,
    ExtraArms = 6,
    Fend = 7,
    FoulAppearance = 8,
    HailMaryPass = 9,
    Horns = 10,
    JumpUp = 11,
    MovementIncrease = 12,
    Pass = 13,
    Sprint = 14,
    StandFirm = 15,
    StripBall = 16,
    SureHands = 17,
    Tackle = 18,
    Tentacles = 19,
    ThickSkull = 20,
    TwoHeads = 21,
    Wrestle = 22,

    // ── BB2025 skills (indices 23–77) ────────────────────────────────────
    AgilityIncrease = 23,
    Animosity = 24,
    BallAndChain = 25,
    BigHand = 26,
    Bombardier = 27,
    BoneHead = 28,
    Brawler = 29,
    BreakTackle = 30,
    BreatheFire = 31,
    Bullseye = 32,
    Chainsaw = 33,
    CloudBurster = 34,
    Defensive = 35,
    DirtyPlayer = 36,
    Dodge = 37,
    EyeGouge = 38,
    Fumblerooski = 39,
    GiveAndGo = 40,
    Hatred = 41,
    HitAndRun = 42,
    HypnoticGaze = 43,
    Insignificant = 44,
    Juggernaut = 45,
    Kick = 46,
    Leader = 47,
    Leap = 48,
    LethalFlight = 49,
    LoneFouler = 50,
    MightyBlow = 51,
    MonstrousMouth = 52,
    NoBall = 53,
    PassingIncrease = 54,
    PileDriver = 55,
    Pogo = 56,
    Pro = 57,
    ProjectileVomit = 58,
    Punt = 59,
    PutTheBootIn = 60,
    QuickFoul = 61,
    ReallyStupid = 62,
    Regeneration = 63,

    // ── Overflow (index ≥ 64 → BTreeSet) ─────────────────────────────────
    RightStuff = 64,
    Saboteur = 65,
    Shadowing = 66,
    SideStep = 67,
    SneakyGit = 68,
    Stab = 69,
    SteadyFooting = 70,
    StrengthIncrease = 71,
    SureFeet = 72,
    Swoop = 73,
    Taunt = 74,
    Unsteady = 75,
    VeryLongLegs = 76,
    ViolentInnovator = 77,

    // ── Team captain special skills (indices 78–97) ───────────────────────
    ASneakyPair = 78,
    BlastIt = 79,
    BlastinSolvesEverything = 80,
    DwarvenScourge = 81,
    ExcuseMeAreYouAZoat = 82,
    FrenziedRush = 83,
    Incorporeal = 84,
    KrumpAndSmash = 85,
    LordOfChaos = 86,
    MasterAssassin = 87,
    MesmerisingDance = 88,
    PumpUpTheCrowd = 89,
    PutridRegurgitation = 90,
    SlashingNails = 91,
    TeamCaptain = 92,
    TheBallista = 93,
    WhirlingDervish = 94,
    WisdomOfTheWhiteDwarf = 95,
    WoodlandFury = 96,
    WorkingInTandem = 97,

    // ── Stat-modifier traits used in legacy/special contexts ──────────────
    ArmourIncrease = 98,
    Claws = 99,
    PrehensileTail = 100,
    WildAnimal = 101,
    AlwaysHungry = 102,
    Bloodlust = 103,
    KnockBack = 104,
    Guard = 105,
    Frenzy = 106,
    Loner = 107,
    NervesOfSteel = 108,
    OnTheBall = 109,
    SafePairOfHands = 110,
    Stunty = 111,
    TakeRoot = 112,
    DeathRoller = 113,
    ThrowTeamMate = 114,
    DivingTackle = 115,
    StrongArm = 116,
    Decay = 117,
    NurglesRot = 118,
    Titchy = 119,
    SecretWeapon = 120,
}

// Skills whose index fits in the u64 bitmask (< 64).
const BITMASK_LIMIT: u8 = 64;

// ── SkillSet ──────────────────────────────────────────────────────────────────

/// Compact set of skills.
/// The first 64 SkillId variants (by repr value) are stored in a u64 bitmask.
/// Rarer skills beyond index 63 fall into a heap-allocated BTreeSet.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct SkillSet {
    mask: u64,
    overflow: BTreeSet<SkillId>,
}

impl SkillSet {
    pub const fn empty() -> Self {
        Self { mask: 0, overflow: BTreeSet::new() }
    }

    pub fn add(&mut self, skill: SkillId) {
        let idx = skill as u8;
        if idx < BITMASK_LIMIT {
            self.mask |= 1u64 << idx;
        } else {
            self.overflow.insert(skill);
        }
    }

    pub fn remove(&mut self, skill: SkillId) {
        let idx = skill as u8;
        if idx < BITMASK_LIMIT {
            self.mask &= !(1u64 << idx);
        } else {
            self.overflow.remove(&skill);
        }
    }

    #[inline]
    pub fn has(&self, skill: SkillId) -> bool {
        let idx = skill as u8;
        if idx < BITMASK_LIMIT {
            self.mask & (1u64 << idx) != 0
        } else {
            self.overflow.contains(&skill)
        }
    }

    pub fn count(&self) -> usize {
        self.mask.count_ones() as usize + self.overflow.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mask == 0 && self.overflow.is_empty()
    }

    pub fn union(&mut self, other: &SkillSet) {
        self.mask |= other.mask;
        for &s in &other.overflow {
            self.overflow.insert(s);
        }
    }

    pub fn iter_overflow(&self) -> impl Iterator<Item = &SkillId> {
        self.overflow.iter()
    }
}

impl FromIterator<SkillId> for SkillSet {
    fn from_iter<I: IntoIterator<Item = SkillId>>(iter: I) -> Self {
        let mut set = Self::empty();
        for s in iter {
            set.add(s);
        }
        set
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_has_common_skill() {
        let mut s = SkillSet::empty();
        assert!(!s.has(SkillId::Block));
        s.add(SkillId::Block);
        assert!(s.has(SkillId::Block));
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn remove_common_skill() {
        let mut s = SkillSet::empty();
        s.add(SkillId::Block);
        s.remove(SkillId::Block);
        assert!(!s.has(SkillId::Block));
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn add_and_has_overflow_skill() {
        let mut s = SkillSet::empty();
        s.add(SkillId::RightStuff); // index 64 → overflow
        assert!(s.has(SkillId::RightStuff));
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn round_trip_add_remove_overflow() {
        let mut s = SkillSet::empty();
        s.add(SkillId::Stunty);
        s.add(SkillId::Loner);
        assert!(s.has(SkillId::Stunty));
        assert!(s.has(SkillId::Loner));
        s.remove(SkillId::Stunty);
        assert!(!s.has(SkillId::Stunty));
        assert!(s.has(SkillId::Loner));
    }

    #[test]
    fn from_iter() {
        let s: SkillSet = [SkillId::Block, SkillId::Dodge, SkillId::Stunty]
            .into_iter()
            .collect();
        assert!(s.has(SkillId::Block));
        assert!(s.has(SkillId::Dodge));
        assert!(s.has(SkillId::Stunty));
        assert_eq!(s.count(), 3);
    }

    #[test]
    fn union() {
        let mut a: SkillSet = [SkillId::Block].into_iter().collect();
        let b: SkillSet = [SkillId::Dodge, SkillId::Stunty].into_iter().collect();
        a.union(&b);
        assert!(a.has(SkillId::Block));
        assert!(a.has(SkillId::Dodge));
        assert!(a.has(SkillId::Stunty));
    }

    #[test]
    fn empty_is_empty() {
        assert!(SkillSet::empty().is_empty());
        let mut s = SkillSet::empty();
        s.add(SkillId::Block);
        assert!(!s.is_empty());
    }
}
