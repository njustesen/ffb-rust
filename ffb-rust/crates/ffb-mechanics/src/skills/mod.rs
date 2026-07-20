use ffb_model::enums::{SkillCategory, SkillUsageType, DeclareCondition, Rules};

pub use ffb_model::enums::SkillId;

/// Static skill definition record.
pub struct SkillDef {
    pub id: SkillId,
    pub class_name: &'static str,
    pub category: SkillCategory,
    pub usage_type: SkillUsageType,
    pub declare_condition: DeclareCondition,
    /// Editions this skill is available in (empty = common to all).
    pub editions: &'static [Rules],
}

/// Complete static skill table (all editions).
///
/// This is the single source of truth for skill metadata at runtime;
/// the Java skill subclasses are folded into rows here.
pub static SKILL_TABLE: &[SkillDef] = &[
    // ── Common ────────────────────────────────────────────────────────────
    SkillDef { id: SkillId::Block, class_name: "Block", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Catch, class_name: "Catch", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Dauntless, class_name: "Dauntless", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::DisturbingPresence, class_name: "DisturbingPresence", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::DivingCatch, class_name: "DivingCatch", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::DumpOff, class_name: "DumpOff", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::ExtraArms, class_name: "ExtraArms", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Fend, class_name: "Fend", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::FoulAppearance, class_name: "FoulAppearance", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::HailMaryPass, class_name: "HailMaryPass", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Horns, class_name: "Horns", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::JumpUp, class_name: "JumpUp", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::MovementIncrease, class_name: "MovementIncrease", category: SkillCategory::StatIncrease, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Pass, class_name: "Pass", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Sprint, class_name: "Sprint", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::StandFirm, class_name: "StandFirm", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::StripBall, class_name: "StripBall", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::SureHands, class_name: "SureHands", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Tackle, class_name: "Tackle", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Tentacles, class_name: "Tentacles", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::ThickSkull, class_name: "ThickSkull", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::TwoHeads, class_name: "TwoHeads", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Wrestle, class_name: "Wrestle", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },

    // ── BB2020 + BB2025 ────────────────────────────────────────────────────
    SkillDef { id: SkillId::AnimalSavagery, class_name: "AnimalSavagery", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Animosity, class_name: "Animosity", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BallAndChain, class_name: "BallAndChain", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Bombardier, class_name: "Bombardier", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerTurn, declare_condition: DeclareCondition::Standing, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BoneHead, class_name: "BoneHead", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Brawler, class_name: "Brawler", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BreakTackle, class_name: "BreakTackle", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BreatheFire, class_name: "BreatheFire", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerTurn, declare_condition: DeclareCondition::Standing, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Chainsaw, class_name: "Chainsaw", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::CloudBurster, class_name: "CloudBurster", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Defensive, class_name: "Defensive", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::DirtyPlayer, class_name: "DirtyPlayer", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Fumblerooskie, class_name: "Fumblerooskie", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::HitAndRun, class_name: "HitAndRun", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::HypnoticGaze, class_name: "HypnoticGaze", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerTurn, declare_condition: DeclareCondition::Standing, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Leap, class_name: "Leap", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MightyBlow, class_name: "MightyBlow", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MonstrousMouth, class_name: "MonstrousMouth", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::NoHands, class_name: "NoHands", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PassingIncrease, class_name: "PassingIncrease", category: SkillCategory::StatIncrease, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PileDriver, class_name: "PileDriver", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PilingOn, class_name: "PilingOn", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PogoStick, class_name: "PogoStick", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::ProjectileVomit, class_name: "ProjectileVomit", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerTurn, declare_condition: DeclareCondition::Standing, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ReallyStupid, class_name: "ReallyStupid", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Regeneration, class_name: "Regeneration", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::RightStuff, class_name: "RightStuff", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::RunningPass, class_name: "RunningPass", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::Shadowing, class_name: "Shadowing", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SideStep, class_name: "SideStep", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::SneakyGit, class_name: "SneakyGit", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Stab, class_name: "Stab", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::StrengthIncrease, class_name: "StrengthIncrease", category: SkillCategory::StatIncrease, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SureFeet, class_name: "SureFeet", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Swarming, class_name: "Swarming", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Swoop, class_name: "Swoop", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::VeryLongLegs, class_name: "VeryLongLegs", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },

    // ── BB2020-only special ────────────────────────────────────────────────
    SkillDef { id: SkillId::ASneakyPair, class_name: "ASneakyPair", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BlastIt, class_name: "BlastIt", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BrutalBlock, class_name: "BrutalBlock", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::BurstOfSpeed, class_name: "BurstOfSpeed", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::ConsummateProfessional, class_name: "ConsummateProfessional", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::DwarfenScourge, class_name: "DwarfenScourge", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::BalefulHex, class_name: "BalefulHex", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ExcuseMeAreYouAZoat, class_name: "ExcuseMeAreYouAZoat", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::FrenziedRush, class_name: "FrenziedRush", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::GhostlyFlames, class_name: "GhostlyFlames", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::Incorporeal, class_name: "Incorporeal", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::LookIntoMyEyes, class_name: "LookIntoMyEyes", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::LordOfChaos, class_name: "LordOfChaos", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MasterAssassin, class_name: "MasterAssassin", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MesmerizingDance, class_name: "MesmerizingDance", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::PumpUpTheCrowd, class_name: "PumpUpTheCrowd", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PutridRegurgitation, class_name: "PutridRegurgitation", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::TheBallista, class_name: "TheBallista", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ThenIStartedBlastin, class_name: "ThenIStartedBlastin", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::TwoForOne, class_name: "TwoForOne", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020] },
    SkillDef { id: SkillId::WhirlingDervish, class_name: "WhirlingDervish", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::WisdomOfTheWhiteDwarf, class_name: "WisdomOfTheWhiteDwarf", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    // Java: mixed/special/Yoink — super("Yoink!", TRAIT, ONCE_PER_GAME), @RulesCollection(BB2020) + (BB2025)
    SkillDef { id: SkillId::Yoink, class_name: "Yoink", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },

    // ── BB2016-only ────────────────────────────────────────────────────────
    SkillDef { id: SkillId::Accurate, class_name: "Accurate", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::AlwaysHungry, class_name: "AlwaysHungry", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::ArmourIncrease, class_name: "ArmourIncrease", category: SkillCategory::StatIncrease, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::BloodLust, class_name: "BloodLust", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Claw, class_name: "Claw", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Decay, class_name: "Decay", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Disposable, class_name: "Disposable", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::DivingTackle, class_name: "DivingTackle", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::FanFavourite, class_name: "FanFavourite", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Frenzy, class_name: "Frenzy", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Grab, class_name: "Grab", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Guard, class_name: "Guard", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::KickOffReturn, class_name: "KickOffReturn", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::KickTeamMate, class_name: "KickTeamMate", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Loner, class_name: "Loner", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::MultipleBlock, class_name: "MultipleBlock", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::NervesOfSteel, class_name: "NervesOfSteel", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::NurglesRot, class_name: "NurglesRot", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::PassBlock, class_name: "PassBlock", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::PrehensileTail, class_name: "PrehensileTail", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::ArmBar, class_name: "ArmBar", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Cannoneer, class_name: "Cannoneer", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::IronHardSkin, class_name: "IronHardSkin", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MyBall, class_name: "MyBall", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PickMeUp, class_name: "PickMeUp", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SafePass, class_name: "SafePass", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SafePairOfHands, class_name: "SafePairOfHands", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::Slayer, class_name: "Slayer", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::ToxinConnoisseur, class_name: "ToxinConnoisseur", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::UnchannelledFury, class_name: "UnchannelledFury", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[] },
    SkillDef { id: SkillId::SafeThrow, class_name: "SafeThrow", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::SecretWeapon, class_name: "SecretWeapon", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Stakes, class_name: "Stakes", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::StrongArm, class_name: "StrongArm", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Stunty, class_name: "Stunty", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::TakeRoot, class_name: "TakeRoot", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::ThrowTeamMate, class_name: "ThrowTeamMate", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Timmmber, class_name: "Timmmber", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::Titchy, class_name: "Titchy", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::WeepingDagger, class_name: "WeepingDagger", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },
    SkillDef { id: SkillId::WildAnimal, class_name: "WildAnimal", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2016] },

    // ── BB2025-only ────────────────────────────────────────────────────────
    SkillDef { id: SkillId::AgilityIncrease, class_name: "AgilityIncrease", category: SkillCategory::StatIncrease, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::BigHand, class_name: "BigHand", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Bullseye, class_name: "Bullseye", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Dodge, class_name: "Dodge", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::EyeGouge, class_name: "EyeGouge", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Fumblerooski, class_name: "Fumblerooski", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::GiveAndGo, class_name: "GiveAndGo", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Hatred, class_name: "Hatred", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Insignificant, class_name: "Insignificant", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Juggernaut, class_name: "Juggernaut", category: SkillCategory::Strength, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Kick, class_name: "Kick", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Leader, class_name: "Leader", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::LethalFlight, class_name: "LethalFlight", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::LoneFouler, class_name: "LoneFouler", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::NoBall, class_name: "NoBall", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::OnTheBall, class_name: "OnTheBall", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Pogo, class_name: "Pogo", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Pro, class_name: "Pro", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Punt, class_name: "Punt", category: SkillCategory::Passing, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::PutTheBootIn, class_name: "PutTheBootIn", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::QuickFoul, class_name: "QuickFoul", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Saboteur, class_name: "Saboteur", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Sidestep, class_name: "Sidestep", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::BlastinSolvesEverything, class_name: "BlastinSolvesEverything", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::DwarvenScourge, class_name: "DwarvenScourge", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::KrumpAndSmash, class_name: "KrumpAndSmash", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::MesmerisingDance, class_name: "MesmerisingDance", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::SlashingNails, class_name: "SlashingNails", category: SkillCategory::Mutation, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::SteadyFooting, class_name: "SteadyFooting", category: SkillCategory::Agility, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::TeamCaptain, class_name: "TeamCaptain", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Taunt, class_name: "Taunt", category: SkillCategory::General, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::Unsteady, class_name: "Unsteady", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::ViolentInnovator, class_name: "ViolentInnovator", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::WoodlandFury, class_name: "WoodlandFury", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },
    SkillDef { id: SkillId::WorkingInTandem, class_name: "WorkingInTandem", category: SkillCategory::Extraordinary, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2025] },

    // ── Mixed/special star player traits (BB2020 + BB2025) ────────────────────
    SkillDef { id: SkillId::AllYouCanEat, class_name: "AllYouCanEat", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BeerBarrelBash, class_name: "BeerBarrelBash", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerDrive, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BlackInk, class_name: "BlackInk", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BlindRage, class_name: "BlindRage", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BoundingLeap, class_name: "BoundingLeap", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::BugmansXXXXXX, class_name: "BugmansXXXXXX", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::CatchOfTheDay, class_name: "CatchOfTheDay", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerHalf, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::CrushingBlow, class_name: "CrushingBlow", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Drunkard, class_name: "Drunkard", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::FuriousOutburst, class_name: "FuriousOutburst", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerHalf, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::FuryOfTheBloodGod, class_name: "FuryOfTheBloodGod", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::GoredByTheBull, class_name: "GoredByTheBull", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::HalflingLuck, class_name: "HalflingLuck", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::IllBeBack, class_name: "IllBeBack", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Indomitable, class_name: "Indomitable", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Kaboom, class_name: "Kaboom", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::KeenPlayer, class_name: "KeenPlayer", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::KickEmWhileTheyReDown, class_name: "KickEmWhileTheyReDown", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::MaximumCarnage, class_name: "MaximumCarnage", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::OldPro, class_name: "OldPro", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PlagueRidden, class_name: "PlagueRidden", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::PrimalSavagery, class_name: "PrimalSavagery", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::QuickBite, class_name: "QuickBite", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::RaidingParty, class_name: "RaidingParty", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerDrive, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Ram, class_name: "Ram", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Reliable, class_name: "Reliable", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SavageBlow, class_name: "SavageBlow", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SavageMauling, class_name: "SavageMauling", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ShotToNothing, class_name: "ShotToNothing", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SneakiestOfTheLot, class_name: "SneakiestOfTheLot", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::StarOfTheShow, class_name: "StarOfTheShow", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::StrongPassingGame, class_name: "StrongPassingGame", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::SwiftAsTheBreeze, class_name: "SwiftAsTheBreeze", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::TastyMorsel, class_name: "TastyMorsel", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::TheFlashingBlade, class_name: "TheFlashingBlade", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ThinkingMansTroll, class_name: "ThinkingMansTroll", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerHalf, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Treacherous, class_name: "Treacherous", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerGame, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::Trickster, class_name: "Trickster", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::UnstoppableMomentum, class_name: "UnstoppableMomentum", category: SkillCategory::Trait, usage_type: SkillUsageType::Regular, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::ViciousVines, class_name: "ViciousVines", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerHalf, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
    SkillDef { id: SkillId::WatchOut, class_name: "WatchOut", category: SkillCategory::Trait, usage_type: SkillUsageType::OncePerHalf, declare_condition: DeclareCondition::None, editions: &[Rules::Bb2020, Rules::Bb2025] },
];

/// Look up a skill definition by id.
pub fn skill_def(id: SkillId) -> Option<&'static SkillDef> {
    SKILL_TABLE.iter().find(|d| d.id == id)
}

/// Look up a skill definition by Java class name.
pub fn skill_def_by_class_name(class_name: &str) -> Option<&'static SkillDef> {
    let id = SkillId::from_class_name(class_name)?;
    skill_def(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Editions that can appear explicitly in a `SkillDef::editions` slice.
    /// An empty slice means the skill is common to all editions.
    const EXPLICIT_EDITIONS: [Rules; 3] = [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025];

    fn available_in(def: &SkillDef, edition: Rules) -> bool {
        def.editions.is_empty() || def.editions.contains(&edition)
    }

    /// Pins the table size and checks structural invariants for every row:
    /// unique ids, unique non-empty class names, and agreement with
    /// `SkillId::class_name()` (the ffb-model side of the same mapping).
    #[test]
    fn table_rows_are_unique_and_consistent() {
        assert_eq!(
            SKILL_TABLE.len(),
            200,
            "SKILL_TABLE row count changed — update the pinned counts in these tests"
        );

        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for def in SKILL_TABLE {
            assert!(!def.class_name.is_empty(), "empty class_name for {:?}", def.id);
            assert!(ids.insert(def.id), "duplicate SkillId in table: {:?}", def.id);
            assert!(names.insert(def.class_name), "duplicate class_name in table: {}", def.class_name);
            // Cross-check against ffb-model's own id -> class-name table.
            assert_eq!(
                def.class_name,
                def.id.class_name(),
                "class_name disagrees with SkillId::class_name() for {:?}",
                def.id
            );
        }
    }

    /// Every SkillId variant has a SKILL_TABLE row (Yoink was the last gap).
    #[test]
    fn every_skill_id_has_a_table_entry() {
        assert!(skill_def(SkillId::Yoink).is_some());
        assert_eq!(skill_def(SkillId::Yoink).unwrap().class_name, "Yoink");
    }

    /// Lookup by id and lookup by class name round-trip for every row.
    #[test]
    fn lookups_round_trip_for_all_skills() {
        for def in SKILL_TABLE {
            assert_eq!(
                SkillId::from_class_name(def.class_name),
                Some(def.id),
                "from_class_name failed for {}",
                def.class_name
            );
            let by_id = skill_def(def.id)
                .unwrap_or_else(|| panic!("no skill_def for {:?}", def.id));
            assert_eq!(by_id.class_name, def.class_name);
            let by_name = skill_def_by_class_name(def.class_name)
                .unwrap_or_else(|| panic!("no skill_def_by_class_name for {}", def.class_name));
            assert_eq!(by_name.id, def.id);
        }
    }

    #[test]
    fn unknown_class_name_returns_none() {
        assert!(SkillId::from_class_name("NonExistentSkill").is_none());
        assert!(skill_def_by_class_name("NonExistentSkill").is_none());
    }

    /// Every skill belongs to at least one edition; edition slices only name
    /// concrete editions (never `Rules::Common` — the empty slice encodes
    /// "common to all") and contain no duplicates. Per-edition membership
    /// counts are pinned so an accidental edition-flag change fails loudly.
    #[test]
    fn edition_membership_is_pinned() {
        let mut common = 0usize;
        for def in SKILL_TABLE {
            assert!(
                !def.editions.contains(&Rules::Common),
                "{}: use an empty editions slice for common skills",
                def.class_name
            );
            let distinct: HashSet<_> = def.editions.iter().collect();
            assert_eq!(distinct.len(), def.editions.len(), "duplicate edition entries for {}", def.class_name);
            assert!(
                EXPLICIT_EDITIONS.iter().any(|&e| available_in(def, e)),
                "{} belongs to no edition",
                def.class_name
            );
            if def.editions.is_empty() {
                common += 1;
            }
        }

        let count_for = |edition| SKILL_TABLE.iter().filter(|d| available_in(d, edition)).count();
        assert_eq!(common, 27, "skills common to all editions");
        assert_eq!(count_for(Rules::Bb2016), 58, "skills available in BB2016");
        assert_eq!(count_for(Rules::Bb2020), 135, "skills available in BB2020");
        assert_eq!(count_for(Rules::Bb2025), 157, "skills available in BB2025");
    }

    /// Pins the per-category distribution of the table. Category validity is
    /// already type-checked (`SkillCategory` enum); this catches accidental
    /// recategorization of individual skills.
    #[test]
    fn category_distribution_is_pinned() {
        let count = |cat| SKILL_TABLE.iter().filter(|d| d.category == cat).count();
        let expected = [
            (SkillCategory::General, 29),
            (SkillCategory::Agility, 19),
            (SkillCategory::Strength, 12),
            (SkillCategory::Passing, 15),
            (SkillCategory::Mutation, 11),
            (SkillCategory::Extraordinary, 63),
            (SkillCategory::StatIncrease, 5),
            (SkillCategory::Trait, 46),
        ];
        let mut total = 0;
        for (cat, n) in expected {
            assert_eq!(count(cat), n, "category count changed for {:?}", cat);
            total += n;
        }
        assert_eq!(total, SKILL_TABLE.len(), "pinned distribution does not cover every table row");
    }
}
