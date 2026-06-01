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

    #[test]
    fn all_table_ids_are_findable() {
        for def in SKILL_TABLE {
            assert_eq!(
                SkillId::from_class_name(def.class_name),
                Some(def.id),
                "from_class_name failed for {}",
                def.class_name
            );
        }
    }

    #[test]
    fn class_name_round_trip() {
        let ids = [SkillId::Block, SkillId::MightyBlow, SkillId::RightStuff, SkillId::TeamCaptain];
        for id in ids {
            let name = id.class_name();
            assert_eq!(SkillId::from_class_name(name), Some(id));
        }
    }

    #[test]
    fn unknown_class_name_returns_none() {
        assert!(SkillId::from_class_name("NonExistentSkill").is_none());
    }

    #[test]
    fn skill_def_lookup() {
        let def = skill_def(SkillId::Chainsaw).unwrap();
        assert_eq!(def.class_name, "Chainsaw");
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }

    // Block
    #[test]
    fn block_class_name_is_block() {
        let def = skill_def(SkillId::Block).unwrap();
        assert_eq!(def.class_name, "Block");
    }

    #[test]
    fn block_category_is_general() {
        let def = skill_def(SkillId::Block).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn block_is_available_in_all_editions() {
        let def = skill_def(SkillId::Block).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }

    #[test]
    fn block_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Block"), Some(SkillId::Block));
    }

    // Wrestle
    #[test]
    fn wrestle_class_name_is_wrestle() {
        let def = skill_def(SkillId::Wrestle).unwrap();
        assert_eq!(def.class_name, "Wrestle");
    }

    #[test]
    fn wrestle_category_is_general() {
        let def = skill_def(SkillId::Wrestle).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn wrestle_is_available_in_all_editions() {
        let def = skill_def(SkillId::Wrestle).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }

    #[test]
    fn wrestle_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Wrestle"), Some(SkillId::Wrestle));
    }

    // Guard
    #[test]
    fn guard_class_name_is_guard() {
        let def = skill_def(SkillId::Guard).unwrap();
        assert_eq!(def.class_name, "Guard");
    }

    #[test]
    fn guard_category_is_strength() {
        let def = skill_def(SkillId::Guard).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }

    #[test]
    fn guard_is_bb2016_edition() {
        let def = skill_def(SkillId::Guard).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }

    #[test]
    fn guard_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Guard"), Some(SkillId::Guard));
    }

    // Frenzy
    #[test]
    fn frenzy_class_name_is_frenzy() {
        let def = skill_def(SkillId::Frenzy).unwrap();
        assert_eq!(def.class_name, "Frenzy");
    }

    #[test]
    fn frenzy_category_is_general() {
        let def = skill_def(SkillId::Frenzy).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn frenzy_is_bb2016_edition() {
        let def = skill_def(SkillId::Frenzy).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }

    #[test]
    fn frenzy_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Frenzy"), Some(SkillId::Frenzy));
    }

    // Catch
    #[test]
    fn catch_class_name_is_catch() {
        let def = skill_def(SkillId::Catch).unwrap();
        assert_eq!(def.class_name, "Catch");
    }

    #[test]
    fn catch_category_is_agility() {
        let def = skill_def(SkillId::Catch).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }

    #[test]
    fn catch_is_available_in_all_editions() {
        let def = skill_def(SkillId::Catch).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }

    #[test]
    fn catch_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Catch"), Some(SkillId::Catch));
    }

    // SureHands
    #[test]
    fn sure_hands_class_name_is_sure_hands() {
        let def = skill_def(SkillId::SureHands).unwrap();
        assert_eq!(def.class_name, "SureHands");
    }

    #[test]
    fn sure_hands_category_is_general() {
        let def = skill_def(SkillId::SureHands).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn sure_hands_is_available_in_all_editions() {
        let def = skill_def(SkillId::SureHands).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }

    #[test]
    fn sure_hands_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SureHands"), Some(SkillId::SureHands));
    }

    // Tackle
    #[test]
    fn tackle_class_name_is_tackle() {
        let def = skill_def(SkillId::Tackle).unwrap();
        assert_eq!(def.class_name, "Tackle");
    }

    #[test]
    fn tackle_category_is_general() {
        let def = skill_def(SkillId::Tackle).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn tackle_is_available_in_all_editions() {
        let def = skill_def(SkillId::Tackle).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }

    #[test]
    fn tackle_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Tackle"), Some(SkillId::Tackle));
    }

    // DirtyPlayer
    #[test]
    fn dirty_player_class_name_is_dirty_player() {
        let def = skill_def(SkillId::DirtyPlayer).unwrap();
        assert_eq!(def.class_name, "DirtyPlayer");
    }

    #[test]
    fn dirty_player_category_is_general() {
        let def = skill_def(SkillId::DirtyPlayer).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }

    #[test]
    fn dirty_player_is_bb2020_edition() {
        let def = skill_def(SkillId::DirtyPlayer).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }

    #[test]
    fn dirty_player_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DirtyPlayer"), Some(SkillId::DirtyPlayer));
    }

    // Dodge
    #[test]
    fn dodge_class_name_is_dodge() {
        let def = skill_def(SkillId::Dodge).unwrap();
        assert_eq!(def.class_name, "Dodge");
    }

    #[test]
    fn dodge_category_is_agility() {
        let def = skill_def(SkillId::Dodge).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }

    #[test]
    fn dodge_is_bb2025_edition() {
        let def = skill_def(SkillId::Dodge).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }

    #[test]
    fn dodge_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Dodge"), Some(SkillId::Dodge));
    }

    // ── Common (all editions) ─────────────────────────────────────────────

    // Dauntless
    #[test]
    fn dauntless_class_name_is_dauntless() {
        let def = skill_def(SkillId::Dauntless).unwrap();
        assert_eq!(def.class_name, "Dauntless");
    }
    #[test]
    fn dauntless_category_is_general() {
        let def = skill_def(SkillId::Dauntless).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn dauntless_is_available_in_all_editions() {
        let def = skill_def(SkillId::Dauntless).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn dauntless_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Dauntless"), Some(SkillId::Dauntless));
    }

    // DisturbingPresence
    #[test]
    fn disturbing_presence_class_name_is_disturbing_presence() {
        let def = skill_def(SkillId::DisturbingPresence).unwrap();
        assert_eq!(def.class_name, "DisturbingPresence");
    }
    #[test]
    fn disturbing_presence_category_is_general() {
        let def = skill_def(SkillId::DisturbingPresence).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn disturbing_presence_is_available_in_all_editions() {
        let def = skill_def(SkillId::DisturbingPresence).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn disturbing_presence_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DisturbingPresence"), Some(SkillId::DisturbingPresence));
    }

    // DivingCatch
    #[test]
    fn diving_catch_class_name_is_diving_catch() {
        let def = skill_def(SkillId::DivingCatch).unwrap();
        assert_eq!(def.class_name, "DivingCatch");
    }
    #[test]
    fn diving_catch_category_is_agility() {
        let def = skill_def(SkillId::DivingCatch).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn diving_catch_is_available_in_all_editions() {
        let def = skill_def(SkillId::DivingCatch).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn diving_catch_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DivingCatch"), Some(SkillId::DivingCatch));
    }

    // DumpOff
    #[test]
    fn dump_off_class_name_is_dump_off() {
        let def = skill_def(SkillId::DumpOff).unwrap();
        assert_eq!(def.class_name, "DumpOff");
    }
    #[test]
    fn dump_off_category_is_passing() {
        let def = skill_def(SkillId::DumpOff).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn dump_off_is_available_in_all_editions() {
        let def = skill_def(SkillId::DumpOff).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn dump_off_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DumpOff"), Some(SkillId::DumpOff));
    }

    // ExtraArms
    #[test]
    fn extra_arms_class_name_is_extra_arms() {
        let def = skill_def(SkillId::ExtraArms).unwrap();
        assert_eq!(def.class_name, "ExtraArms");
    }
    #[test]
    fn extra_arms_category_is_mutation() {
        let def = skill_def(SkillId::ExtraArms).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn extra_arms_is_available_in_all_editions() {
        let def = skill_def(SkillId::ExtraArms).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn extra_arms_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ExtraArms"), Some(SkillId::ExtraArms));
    }

    // Fend
    #[test]
    fn fend_class_name_is_fend() {
        let def = skill_def(SkillId::Fend).unwrap();
        assert_eq!(def.class_name, "Fend");
    }
    #[test]
    fn fend_category_is_general() {
        let def = skill_def(SkillId::Fend).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn fend_is_available_in_all_editions() {
        let def = skill_def(SkillId::Fend).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn fend_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Fend"), Some(SkillId::Fend));
    }

    // FoulAppearance
    #[test]
    fn foul_appearance_class_name_is_foul_appearance() {
        let def = skill_def(SkillId::FoulAppearance).unwrap();
        assert_eq!(def.class_name, "FoulAppearance");
    }
    #[test]
    fn foul_appearance_category_is_mutation() {
        let def = skill_def(SkillId::FoulAppearance).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn foul_appearance_is_available_in_all_editions() {
        let def = skill_def(SkillId::FoulAppearance).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn foul_appearance_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("FoulAppearance"), Some(SkillId::FoulAppearance));
    }

    // HailMaryPass
    #[test]
    fn hail_mary_pass_class_name_is_hail_mary_pass() {
        let def = skill_def(SkillId::HailMaryPass).unwrap();
        assert_eq!(def.class_name, "HailMaryPass");
    }
    #[test]
    fn hail_mary_pass_category_is_passing() {
        let def = skill_def(SkillId::HailMaryPass).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn hail_mary_pass_is_available_in_all_editions() {
        let def = skill_def(SkillId::HailMaryPass).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn hail_mary_pass_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("HailMaryPass"), Some(SkillId::HailMaryPass));
    }

    // Horns
    #[test]
    fn horns_class_name_is_horns() {
        let def = skill_def(SkillId::Horns).unwrap();
        assert_eq!(def.class_name, "Horns");
    }
    #[test]
    fn horns_category_is_mutation() {
        let def = skill_def(SkillId::Horns).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn horns_is_available_in_all_editions() {
        let def = skill_def(SkillId::Horns).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn horns_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Horns"), Some(SkillId::Horns));
    }

    // JumpUp
    #[test]
    fn jump_up_class_name_is_jump_up() {
        let def = skill_def(SkillId::JumpUp).unwrap();
        assert_eq!(def.class_name, "JumpUp");
    }
    #[test]
    fn jump_up_category_is_agility() {
        let def = skill_def(SkillId::JumpUp).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn jump_up_is_available_in_all_editions() {
        let def = skill_def(SkillId::JumpUp).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn jump_up_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("JumpUp"), Some(SkillId::JumpUp));
    }

    // MovementIncrease
    #[test]
    fn movement_increase_class_name_is_movement_increase() {
        let def = skill_def(SkillId::MovementIncrease).unwrap();
        assert_eq!(def.class_name, "MovementIncrease");
    }
    #[test]
    fn movement_increase_category_is_stat_increase() {
        let def = skill_def(SkillId::MovementIncrease).unwrap();
        assert_eq!(def.category, SkillCategory::StatIncrease);
    }
    #[test]
    fn movement_increase_is_available_in_all_editions() {
        let def = skill_def(SkillId::MovementIncrease).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn movement_increase_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MovementIncrease"), Some(SkillId::MovementIncrease));
    }

    // Pass
    #[test]
    fn pass_class_name_is_pass() {
        let def = skill_def(SkillId::Pass).unwrap();
        assert_eq!(def.class_name, "Pass");
    }
    #[test]
    fn pass_category_is_passing() {
        let def = skill_def(SkillId::Pass).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn pass_is_available_in_all_editions() {
        let def = skill_def(SkillId::Pass).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn pass_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Pass"), Some(SkillId::Pass));
    }

    // Sprint
    #[test]
    fn sprint_class_name_is_sprint() {
        let def = skill_def(SkillId::Sprint).unwrap();
        assert_eq!(def.class_name, "Sprint");
    }
    #[test]
    fn sprint_category_is_agility() {
        let def = skill_def(SkillId::Sprint).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn sprint_is_available_in_all_editions() {
        let def = skill_def(SkillId::Sprint).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn sprint_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Sprint"), Some(SkillId::Sprint));
    }

    // StandFirm
    #[test]
    fn stand_firm_class_name_is_stand_firm() {
        let def = skill_def(SkillId::StandFirm).unwrap();
        assert_eq!(def.class_name, "StandFirm");
    }
    #[test]
    fn stand_firm_category_is_strength() {
        let def = skill_def(SkillId::StandFirm).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn stand_firm_is_available_in_all_editions() {
        let def = skill_def(SkillId::StandFirm).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn stand_firm_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("StandFirm"), Some(SkillId::StandFirm));
    }

    // StripBall
    #[test]
    fn strip_ball_class_name_is_strip_ball() {
        let def = skill_def(SkillId::StripBall).unwrap();
        assert_eq!(def.class_name, "StripBall");
    }
    #[test]
    fn strip_ball_category_is_general() {
        let def = skill_def(SkillId::StripBall).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn strip_ball_is_available_in_all_editions() {
        let def = skill_def(SkillId::StripBall).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn strip_ball_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("StripBall"), Some(SkillId::StripBall));
    }

    // Tentacles
    #[test]
    fn tentacles_class_name_is_tentacles() {
        let def = skill_def(SkillId::Tentacles).unwrap();
        assert_eq!(def.class_name, "Tentacles");
    }
    #[test]
    fn tentacles_category_is_mutation() {
        let def = skill_def(SkillId::Tentacles).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn tentacles_is_available_in_all_editions() {
        let def = skill_def(SkillId::Tentacles).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn tentacles_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Tentacles"), Some(SkillId::Tentacles));
    }

    // ThickSkull
    #[test]
    fn thick_skull_class_name_is_thick_skull() {
        let def = skill_def(SkillId::ThickSkull).unwrap();
        assert_eq!(def.class_name, "ThickSkull");
    }
    #[test]
    fn thick_skull_category_is_general() {
        let def = skill_def(SkillId::ThickSkull).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn thick_skull_is_available_in_all_editions() {
        let def = skill_def(SkillId::ThickSkull).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn thick_skull_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ThickSkull"), Some(SkillId::ThickSkull));
    }

    // TwoHeads
    #[test]
    fn two_heads_class_name_is_two_heads() {
        let def = skill_def(SkillId::TwoHeads).unwrap();
        assert_eq!(def.class_name, "TwoHeads");
    }
    #[test]
    fn two_heads_category_is_mutation() {
        let def = skill_def(SkillId::TwoHeads).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn two_heads_is_available_in_all_editions() {
        let def = skill_def(SkillId::TwoHeads).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn two_heads_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("TwoHeads"), Some(SkillId::TwoHeads));
    }

    // SafePairOfHands
    #[test]
    fn safe_pair_of_hands_class_name_is_safe_pair_of_hands() {
        let def = skill_def(SkillId::SafePairOfHands).unwrap();
        assert_eq!(def.class_name, "SafePairOfHands");
    }
    #[test]
    fn safe_pair_of_hands_category_is_general() {
        let def = skill_def(SkillId::SafePairOfHands).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn safe_pair_of_hands_is_available_in_all_editions() {
        let def = skill_def(SkillId::SafePairOfHands).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn safe_pair_of_hands_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SafePairOfHands"), Some(SkillId::SafePairOfHands));
    }

    // Slayer
    #[test]
    fn slayer_class_name_is_slayer() {
        let def = skill_def(SkillId::Slayer).unwrap();
        assert_eq!(def.class_name, "Slayer");
    }
    #[test]
    fn slayer_category_is_extraordinary() {
        let def = skill_def(SkillId::Slayer).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn slayer_is_available_in_all_editions() {
        let def = skill_def(SkillId::Slayer).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn slayer_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Slayer"), Some(SkillId::Slayer));
    }

    // ToxinConnoisseur
    #[test]
    fn toxin_connoisseur_class_name_is_toxin_connoisseur() {
        let def = skill_def(SkillId::ToxinConnoisseur).unwrap();
        assert_eq!(def.class_name, "ToxinConnoisseur");
    }
    #[test]
    fn toxin_connoisseur_category_is_extraordinary() {
        let def = skill_def(SkillId::ToxinConnoisseur).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn toxin_connoisseur_is_available_in_all_editions() {
        let def = skill_def(SkillId::ToxinConnoisseur).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn toxin_connoisseur_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ToxinConnoisseur"), Some(SkillId::ToxinConnoisseur));
    }

    // UnchannelledFury
    #[test]
    fn unchannelled_fury_class_name_is_unchannelled_fury() {
        let def = skill_def(SkillId::UnchannelledFury).unwrap();
        assert_eq!(def.class_name, "UnchannelledFury");
    }
    #[test]
    fn unchannelled_fury_category_is_extraordinary() {
        let def = skill_def(SkillId::UnchannelledFury).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn unchannelled_fury_is_available_in_all_editions() {
        let def = skill_def(SkillId::UnchannelledFury).unwrap();
        assert!(def.editions.is_empty(), "editions=[] means skill is common to all editions");
    }
    #[test]
    fn unchannelled_fury_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("UnchannelledFury"), Some(SkillId::UnchannelledFury));
    }

    // ── BB2020 / BB2020+BB2025 ────────────────────────────────────────────

    // AnimalSavagery
    #[test]
    fn animal_savagery_class_name_is_animal_savagery() {
        let def = skill_def(SkillId::AnimalSavagery).unwrap();
        assert_eq!(def.class_name, "AnimalSavagery");
    }
    #[test]
    fn animal_savagery_category_is_general() {
        let def = skill_def(SkillId::AnimalSavagery).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn animal_savagery_is_bb2020_edition() {
        let def = skill_def(SkillId::AnimalSavagery).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn animal_savagery_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("AnimalSavagery"), Some(SkillId::AnimalSavagery));
    }

    // Animosity
    #[test]
    fn animosity_class_name_is_animosity() {
        let def = skill_def(SkillId::Animosity).unwrap();
        assert_eq!(def.class_name, "Animosity");
    }
    #[test]
    fn animosity_category_is_general() {
        let def = skill_def(SkillId::Animosity).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn animosity_is_bb2020_edition() {
        let def = skill_def(SkillId::Animosity).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn animosity_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Animosity"), Some(SkillId::Animosity));
    }

    // BallAndChain
    #[test]
    fn ball_and_chain_class_name_is_ball_and_chain() {
        let def = skill_def(SkillId::BallAndChain).unwrap();
        assert_eq!(def.class_name, "BallAndChain");
    }
    #[test]
    fn ball_and_chain_category_is_extraordinary() {
        let def = skill_def(SkillId::BallAndChain).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn ball_and_chain_is_bb2020_edition() {
        let def = skill_def(SkillId::BallAndChain).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn ball_and_chain_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BallAndChain"), Some(SkillId::BallAndChain));
    }

    // Bombardier
    #[test]
    fn bombardier_class_name_is_bombardier() {
        let def = skill_def(SkillId::Bombardier).unwrap();
        assert_eq!(def.class_name, "Bombardier");
    }
    #[test]
    fn bombardier_category_is_extraordinary() {
        let def = skill_def(SkillId::Bombardier).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn bombardier_is_bb2020_edition() {
        let def = skill_def(SkillId::Bombardier).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn bombardier_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Bombardier"), Some(SkillId::Bombardier));
    }

    // BoneHead
    #[test]
    fn bone_head_class_name_is_bone_head() {
        let def = skill_def(SkillId::BoneHead).unwrap();
        assert_eq!(def.class_name, "BoneHead");
    }
    #[test]
    fn bone_head_category_is_general() {
        let def = skill_def(SkillId::BoneHead).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn bone_head_is_bb2020_edition() {
        let def = skill_def(SkillId::BoneHead).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn bone_head_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BoneHead"), Some(SkillId::BoneHead));
    }

    // Brawler
    #[test]
    fn brawler_class_name_is_brawler() {
        let def = skill_def(SkillId::Brawler).unwrap();
        assert_eq!(def.class_name, "Brawler");
    }
    #[test]
    fn brawler_category_is_strength() {
        let def = skill_def(SkillId::Brawler).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn brawler_is_bb2020_edition() {
        let def = skill_def(SkillId::Brawler).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn brawler_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Brawler"), Some(SkillId::Brawler));
    }

    // BreakTackle
    #[test]
    fn break_tackle_class_name_is_break_tackle() {
        let def = skill_def(SkillId::BreakTackle).unwrap();
        assert_eq!(def.class_name, "BreakTackle");
    }
    #[test]
    fn break_tackle_category_is_strength() {
        let def = skill_def(SkillId::BreakTackle).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn break_tackle_is_bb2020_edition() {
        let def = skill_def(SkillId::BreakTackle).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn break_tackle_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BreakTackle"), Some(SkillId::BreakTackle));
    }

    // BreatheFire
    #[test]
    fn breathe_fire_class_name_is_breathe_fire() {
        let def = skill_def(SkillId::BreatheFire).unwrap();
        assert_eq!(def.class_name, "BreatheFire");
    }
    #[test]
    fn breathe_fire_category_is_extraordinary() {
        let def = skill_def(SkillId::BreatheFire).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn breathe_fire_is_bb2020_edition() {
        let def = skill_def(SkillId::BreatheFire).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn breathe_fire_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BreatheFire"), Some(SkillId::BreatheFire));
    }

    // Chainsaw
    #[test]
    fn chainsaw_class_name_is_chainsaw() {
        let def = skill_def(SkillId::Chainsaw).unwrap();
        assert_eq!(def.class_name, "Chainsaw");
    }
    #[test]
    fn chainsaw_category_is_extraordinary() {
        let def = skill_def(SkillId::Chainsaw).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn chainsaw_is_bb2020_edition() {
        let def = skill_def(SkillId::Chainsaw).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn chainsaw_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Chainsaw"), Some(SkillId::Chainsaw));
    }

    // CloudBurster
    #[test]
    fn cloud_burster_class_name_is_cloud_burster() {
        let def = skill_def(SkillId::CloudBurster).unwrap();
        assert_eq!(def.class_name, "CloudBurster");
    }
    #[test]
    fn cloud_burster_category_is_passing() {
        let def = skill_def(SkillId::CloudBurster).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn cloud_burster_is_bb2020_edition() {
        let def = skill_def(SkillId::CloudBurster).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn cloud_burster_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("CloudBurster"), Some(SkillId::CloudBurster));
    }

    // Defensive
    #[test]
    fn defensive_class_name_is_defensive() {
        let def = skill_def(SkillId::Defensive).unwrap();
        assert_eq!(def.class_name, "Defensive");
    }
    #[test]
    fn defensive_category_is_general() {
        let def = skill_def(SkillId::Defensive).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn defensive_is_bb2020_edition() {
        let def = skill_def(SkillId::Defensive).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn defensive_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Defensive"), Some(SkillId::Defensive));
    }

    // Fumblerooskie
    #[test]
    fn fumblerooskie_class_name_is_fumblerooskie() {
        let def = skill_def(SkillId::Fumblerooskie).unwrap();
        assert_eq!(def.class_name, "Fumblerooskie");
    }
    #[test]
    fn fumblerooskie_category_is_passing() {
        let def = skill_def(SkillId::Fumblerooskie).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn fumblerooskie_is_bb2020_edition() {
        let def = skill_def(SkillId::Fumblerooskie).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn fumblerooskie_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Fumblerooskie"), Some(SkillId::Fumblerooskie));
    }

    // HitAndRun
    #[test]
    fn hit_and_run_class_name_is_hit_and_run() {
        let def = skill_def(SkillId::HitAndRun).unwrap();
        assert_eq!(def.class_name, "HitAndRun");
    }
    #[test]
    fn hit_and_run_category_is_agility() {
        let def = skill_def(SkillId::HitAndRun).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn hit_and_run_is_bb2020_edition() {
        let def = skill_def(SkillId::HitAndRun).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn hit_and_run_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("HitAndRun"), Some(SkillId::HitAndRun));
    }

    // HypnoticGaze
    #[test]
    fn hypnotic_gaze_class_name_is_hypnotic_gaze() {
        let def = skill_def(SkillId::HypnoticGaze).unwrap();
        assert_eq!(def.class_name, "HypnoticGaze");
    }
    #[test]
    fn hypnotic_gaze_category_is_extraordinary() {
        let def = skill_def(SkillId::HypnoticGaze).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn hypnotic_gaze_is_bb2020_edition() {
        let def = skill_def(SkillId::HypnoticGaze).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn hypnotic_gaze_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("HypnoticGaze"), Some(SkillId::HypnoticGaze));
    }

    // Leap
    #[test]
    fn leap_class_name_is_leap() {
        let def = skill_def(SkillId::Leap).unwrap();
        assert_eq!(def.class_name, "Leap");
    }
    #[test]
    fn leap_category_is_agility() {
        let def = skill_def(SkillId::Leap).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn leap_is_bb2020_edition() {
        let def = skill_def(SkillId::Leap).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn leap_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Leap"), Some(SkillId::Leap));
    }

    // MightyBlow
    #[test]
    fn mighty_blow_class_name_is_mighty_blow() {
        let def = skill_def(SkillId::MightyBlow).unwrap();
        assert_eq!(def.class_name, "MightyBlow");
    }
    #[test]
    fn mighty_blow_category_is_strength() {
        let def = skill_def(SkillId::MightyBlow).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn mighty_blow_is_bb2020_edition() {
        let def = skill_def(SkillId::MightyBlow).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn mighty_blow_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MightyBlow"), Some(SkillId::MightyBlow));
    }

    // MonstrousMouth
    #[test]
    fn monstrous_mouth_class_name_is_monstrous_mouth() {
        let def = skill_def(SkillId::MonstrousMouth).unwrap();
        assert_eq!(def.class_name, "MonstrousMouth");
    }
    #[test]
    fn monstrous_mouth_category_is_mutation() {
        let def = skill_def(SkillId::MonstrousMouth).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn monstrous_mouth_is_bb2020_edition() {
        let def = skill_def(SkillId::MonstrousMouth).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn monstrous_mouth_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MonstrousMouth"), Some(SkillId::MonstrousMouth));
    }

    // NoHands
    #[test]
    fn no_hands_class_name_is_no_hands() {
        let def = skill_def(SkillId::NoHands).unwrap();
        assert_eq!(def.class_name, "NoHands");
    }
    #[test]
    fn no_hands_category_is_extraordinary() {
        let def = skill_def(SkillId::NoHands).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn no_hands_is_bb2020_edition() {
        let def = skill_def(SkillId::NoHands).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn no_hands_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("NoHands"), Some(SkillId::NoHands));
    }

    // PassingIncrease
    #[test]
    fn passing_increase_class_name_is_passing_increase() {
        let def = skill_def(SkillId::PassingIncrease).unwrap();
        assert_eq!(def.class_name, "PassingIncrease");
    }
    #[test]
    fn passing_increase_category_is_stat_increase() {
        let def = skill_def(SkillId::PassingIncrease).unwrap();
        assert_eq!(def.category, SkillCategory::StatIncrease);
    }
    #[test]
    fn passing_increase_is_bb2020_edition() {
        let def = skill_def(SkillId::PassingIncrease).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn passing_increase_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PassingIncrease"), Some(SkillId::PassingIncrease));
    }

    // PileDriver
    #[test]
    fn pile_driver_class_name_is_pile_driver() {
        let def = skill_def(SkillId::PileDriver).unwrap();
        assert_eq!(def.class_name, "PileDriver");
    }
    #[test]
    fn pile_driver_category_is_strength() {
        let def = skill_def(SkillId::PileDriver).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn pile_driver_is_bb2020_edition() {
        let def = skill_def(SkillId::PileDriver).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn pile_driver_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PileDriver"), Some(SkillId::PileDriver));
    }

    // PilingOn
    #[test]
    fn piling_on_class_name_is_piling_on() {
        let def = skill_def(SkillId::PilingOn).unwrap();
        assert_eq!(def.class_name, "PilingOn");
    }
    #[test]
    fn piling_on_category_is_strength() {
        let def = skill_def(SkillId::PilingOn).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn piling_on_is_bb2020_edition() {
        let def = skill_def(SkillId::PilingOn).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn piling_on_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PilingOn"), Some(SkillId::PilingOn));
    }

    // PogoStick
    #[test]
    fn pogo_stick_class_name_is_pogo_stick() {
        let def = skill_def(SkillId::PogoStick).unwrap();
        assert_eq!(def.class_name, "PogoStick");
    }
    #[test]
    fn pogo_stick_category_is_agility() {
        let def = skill_def(SkillId::PogoStick).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn pogo_stick_is_bb2020_edition() {
        let def = skill_def(SkillId::PogoStick).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn pogo_stick_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PogoStick"), Some(SkillId::PogoStick));
    }

    // ProjectileVomit
    #[test]
    fn projectile_vomit_class_name_is_projectile_vomit() {
        let def = skill_def(SkillId::ProjectileVomit).unwrap();
        assert_eq!(def.class_name, "ProjectileVomit");
    }
    #[test]
    fn projectile_vomit_category_is_extraordinary() {
        let def = skill_def(SkillId::ProjectileVomit).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn projectile_vomit_is_bb2020_edition() {
        let def = skill_def(SkillId::ProjectileVomit).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn projectile_vomit_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ProjectileVomit"), Some(SkillId::ProjectileVomit));
    }

    // ReallyStupid
    #[test]
    fn really_stupid_class_name_is_really_stupid() {
        let def = skill_def(SkillId::ReallyStupid).unwrap();
        assert_eq!(def.class_name, "ReallyStupid");
    }
    #[test]
    fn really_stupid_category_is_general() {
        let def = skill_def(SkillId::ReallyStupid).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn really_stupid_is_bb2020_edition() {
        let def = skill_def(SkillId::ReallyStupid).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn really_stupid_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ReallyStupid"), Some(SkillId::ReallyStupid));
    }

    // Regeneration
    #[test]
    fn regeneration_class_name_is_regeneration() {
        let def = skill_def(SkillId::Regeneration).unwrap();
        assert_eq!(def.class_name, "Regeneration");
    }
    #[test]
    fn regeneration_category_is_extraordinary() {
        let def = skill_def(SkillId::Regeneration).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn regeneration_is_bb2020_edition() {
        let def = skill_def(SkillId::Regeneration).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn regeneration_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Regeneration"), Some(SkillId::Regeneration));
    }

    // RightStuff
    #[test]
    fn right_stuff_class_name_is_right_stuff() {
        let def = skill_def(SkillId::RightStuff).unwrap();
        assert_eq!(def.class_name, "RightStuff");
    }
    #[test]
    fn right_stuff_category_is_extraordinary() {
        let def = skill_def(SkillId::RightStuff).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn right_stuff_is_bb2020_edition() {
        let def = skill_def(SkillId::RightStuff).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn right_stuff_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("RightStuff"), Some(SkillId::RightStuff));
    }

    // RunningPass
    #[test]
    fn running_pass_class_name_is_running_pass() {
        let def = skill_def(SkillId::RunningPass).unwrap();
        assert_eq!(def.class_name, "RunningPass");
    }
    #[test]
    fn running_pass_category_is_passing() {
        let def = skill_def(SkillId::RunningPass).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn running_pass_is_bb2020_edition() {
        let def = skill_def(SkillId::RunningPass).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn running_pass_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("RunningPass"), Some(SkillId::RunningPass));
    }

    // Shadowing
    #[test]
    fn shadowing_class_name_is_shadowing() {
        let def = skill_def(SkillId::Shadowing).unwrap();
        assert_eq!(def.class_name, "Shadowing");
    }
    #[test]
    fn shadowing_category_is_general() {
        let def = skill_def(SkillId::Shadowing).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn shadowing_is_bb2020_edition() {
        let def = skill_def(SkillId::Shadowing).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn shadowing_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Shadowing"), Some(SkillId::Shadowing));
    }

    // SideStep
    #[test]
    fn side_step_class_name_is_side_step() {
        let def = skill_def(SkillId::SideStep).unwrap();
        assert_eq!(def.class_name, "SideStep");
    }
    #[test]
    fn side_step_category_is_agility() {
        let def = skill_def(SkillId::SideStep).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn side_step_is_bb2020_edition() {
        let def = skill_def(SkillId::SideStep).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn side_step_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SideStep"), Some(SkillId::SideStep));
    }

    // SneakyGit
    #[test]
    fn sneaky_git_class_name_is_sneaky_git() {
        let def = skill_def(SkillId::SneakyGit).unwrap();
        assert_eq!(def.class_name, "SneakyGit");
    }
    #[test]
    fn sneaky_git_category_is_agility() {
        let def = skill_def(SkillId::SneakyGit).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn sneaky_git_is_bb2020_edition() {
        let def = skill_def(SkillId::SneakyGit).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn sneaky_git_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SneakyGit"), Some(SkillId::SneakyGit));
    }

    // Stab
    #[test]
    fn stab_class_name_is_stab() {
        let def = skill_def(SkillId::Stab).unwrap();
        assert_eq!(def.class_name, "Stab");
    }
    #[test]
    fn stab_category_is_extraordinary() {
        let def = skill_def(SkillId::Stab).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn stab_is_bb2020_edition() {
        let def = skill_def(SkillId::Stab).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn stab_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Stab"), Some(SkillId::Stab));
    }

    // StrengthIncrease
    #[test]
    fn strength_increase_class_name_is_strength_increase() {
        let def = skill_def(SkillId::StrengthIncrease).unwrap();
        assert_eq!(def.class_name, "StrengthIncrease");
    }
    #[test]
    fn strength_increase_category_is_stat_increase() {
        let def = skill_def(SkillId::StrengthIncrease).unwrap();
        assert_eq!(def.category, SkillCategory::StatIncrease);
    }
    #[test]
    fn strength_increase_is_bb2020_edition() {
        let def = skill_def(SkillId::StrengthIncrease).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn strength_increase_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("StrengthIncrease"), Some(SkillId::StrengthIncrease));
    }

    // SureFeet
    #[test]
    fn sure_feet_class_name_is_sure_feet() {
        let def = skill_def(SkillId::SureFeet).unwrap();
        assert_eq!(def.class_name, "SureFeet");
    }
    #[test]
    fn sure_feet_category_is_agility() {
        let def = skill_def(SkillId::SureFeet).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn sure_feet_is_bb2020_edition() {
        let def = skill_def(SkillId::SureFeet).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn sure_feet_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SureFeet"), Some(SkillId::SureFeet));
    }

    // Swarming
    #[test]
    fn swarming_class_name_is_swarming() {
        let def = skill_def(SkillId::Swarming).unwrap();
        assert_eq!(def.class_name, "Swarming");
    }
    #[test]
    fn swarming_category_is_general() {
        let def = skill_def(SkillId::Swarming).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn swarming_is_bb2020_edition() {
        let def = skill_def(SkillId::Swarming).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn swarming_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Swarming"), Some(SkillId::Swarming));
    }

    // Swoop
    #[test]
    fn swoop_class_name_is_swoop() {
        let def = skill_def(SkillId::Swoop).unwrap();
        assert_eq!(def.class_name, "Swoop");
    }
    #[test]
    fn swoop_category_is_extraordinary() {
        let def = skill_def(SkillId::Swoop).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn swoop_is_bb2020_edition() {
        let def = skill_def(SkillId::Swoop).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn swoop_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Swoop"), Some(SkillId::Swoop));
    }

    // VeryLongLegs
    #[test]
    fn very_long_legs_class_name_is_very_long_legs() {
        let def = skill_def(SkillId::VeryLongLegs).unwrap();
        assert_eq!(def.class_name, "VeryLongLegs");
    }
    #[test]
    fn very_long_legs_category_is_extraordinary() {
        let def = skill_def(SkillId::VeryLongLegs).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn very_long_legs_is_bb2020_edition() {
        let def = skill_def(SkillId::VeryLongLegs).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn very_long_legs_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("VeryLongLegs"), Some(SkillId::VeryLongLegs));
    }

    // ArmBar
    #[test]
    fn arm_bar_class_name_is_arm_bar() {
        let def = skill_def(SkillId::ArmBar).unwrap();
        assert_eq!(def.class_name, "ArmBar");
    }
    #[test]
    fn arm_bar_category_is_strength() {
        let def = skill_def(SkillId::ArmBar).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn arm_bar_is_bb2020_edition() {
        let def = skill_def(SkillId::ArmBar).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn arm_bar_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ArmBar"), Some(SkillId::ArmBar));
    }

    // Cannoneer
    #[test]
    fn cannoneer_class_name_is_cannoneer() {
        let def = skill_def(SkillId::Cannoneer).unwrap();
        assert_eq!(def.class_name, "Cannoneer");
    }
    #[test]
    fn cannoneer_category_is_passing() {
        let def = skill_def(SkillId::Cannoneer).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn cannoneer_is_bb2020_edition() {
        let def = skill_def(SkillId::Cannoneer).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn cannoneer_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Cannoneer"), Some(SkillId::Cannoneer));
    }

    // IronHardSkin
    #[test]
    fn iron_hard_skin_class_name_is_iron_hard_skin() {
        let def = skill_def(SkillId::IronHardSkin).unwrap();
        assert_eq!(def.class_name, "IronHardSkin");
    }
    #[test]
    fn iron_hard_skin_category_is_mutation() {
        let def = skill_def(SkillId::IronHardSkin).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn iron_hard_skin_is_bb2020_edition() {
        let def = skill_def(SkillId::IronHardSkin).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn iron_hard_skin_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("IronHardSkin"), Some(SkillId::IronHardSkin));
    }

    // MyBall
    #[test]
    fn my_ball_class_name_is_my_ball() {
        let def = skill_def(SkillId::MyBall).unwrap();
        assert_eq!(def.class_name, "MyBall");
    }
    #[test]
    fn my_ball_category_is_trait() {
        let def = skill_def(SkillId::MyBall).unwrap();
        assert_eq!(def.category, SkillCategory::Trait);
    }
    #[test]
    fn my_ball_is_bb2020_edition() {
        let def = skill_def(SkillId::MyBall).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn my_ball_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MyBall"), Some(SkillId::MyBall));
    }

    // PickMeUp
    #[test]
    fn pick_me_up_class_name_is_pick_me_up() {
        let def = skill_def(SkillId::PickMeUp).unwrap();
        assert_eq!(def.class_name, "PickMeUp");
    }
    #[test]
    fn pick_me_up_category_is_trait() {
        let def = skill_def(SkillId::PickMeUp).unwrap();
        assert_eq!(def.category, SkillCategory::Trait);
    }
    #[test]
    fn pick_me_up_is_bb2020_edition() {
        let def = skill_def(SkillId::PickMeUp).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn pick_me_up_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PickMeUp"), Some(SkillId::PickMeUp));
    }

    // SafePass
    #[test]
    fn safe_pass_class_name_is_safe_pass() {
        let def = skill_def(SkillId::SafePass).unwrap();
        assert_eq!(def.class_name, "SafePass");
    }
    #[test]
    fn safe_pass_category_is_passing() {
        let def = skill_def(SkillId::SafePass).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn safe_pass_is_bb2020_edition() {
        let def = skill_def(SkillId::SafePass).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn safe_pass_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SafePass"), Some(SkillId::SafePass));
    }

    // OnTheBall
    #[test]
    fn on_the_ball_class_name_is_on_the_ball() {
        let def = skill_def(SkillId::OnTheBall).unwrap();
        assert_eq!(def.class_name, "OnTheBall");
    }
    #[test]
    fn on_the_ball_category_is_general() {
        let def = skill_def(SkillId::OnTheBall).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn on_the_ball_is_bb2020_edition() {
        let def = skill_def(SkillId::OnTheBall).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn on_the_ball_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("OnTheBall"), Some(SkillId::OnTheBall));
    }

    // ── BB2020 special ────────────────────────────────────────────────────

    // ASneakyPair
    #[test]
    fn a_sneaky_pair_class_name_is_a_sneaky_pair() {
        let def = skill_def(SkillId::ASneakyPair).unwrap();
        assert_eq!(def.class_name, "ASneakyPair");
    }
    #[test]
    fn a_sneaky_pair_category_is_extraordinary() {
        let def = skill_def(SkillId::ASneakyPair).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn a_sneaky_pair_is_bb2020_edition() {
        let def = skill_def(SkillId::ASneakyPair).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn a_sneaky_pair_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ASneakyPair"), Some(SkillId::ASneakyPair));
    }

    // BlastIt
    #[test]
    fn blast_it_class_name_is_blast_it() {
        let def = skill_def(SkillId::BlastIt).unwrap();
        assert_eq!(def.class_name, "BlastIt");
    }
    #[test]
    fn blast_it_category_is_extraordinary() {
        let def = skill_def(SkillId::BlastIt).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn blast_it_is_bb2020_edition() {
        let def = skill_def(SkillId::BlastIt).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn blast_it_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BlastIt"), Some(SkillId::BlastIt));
    }

    // BrutalBlock
    #[test]
    fn brutal_block_class_name_is_brutal_block() {
        let def = skill_def(SkillId::BrutalBlock).unwrap();
        assert_eq!(def.class_name, "BrutalBlock");
    }
    #[test]
    fn brutal_block_category_is_extraordinary() {
        let def = skill_def(SkillId::BrutalBlock).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn brutal_block_is_bb2020_edition() {
        let def = skill_def(SkillId::BrutalBlock).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn brutal_block_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BrutalBlock"), Some(SkillId::BrutalBlock));
    }

    // BurstOfSpeed
    #[test]
    fn burst_of_speed_class_name_is_burst_of_speed() {
        let def = skill_def(SkillId::BurstOfSpeed).unwrap();
        assert_eq!(def.class_name, "BurstOfSpeed");
    }
    #[test]
    fn burst_of_speed_category_is_extraordinary() {
        let def = skill_def(SkillId::BurstOfSpeed).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn burst_of_speed_is_bb2020_edition() {
        let def = skill_def(SkillId::BurstOfSpeed).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn burst_of_speed_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BurstOfSpeed"), Some(SkillId::BurstOfSpeed));
    }

    // ConsummateProfessional
    #[test]
    fn consummate_professional_class_name_is_consummate_professional() {
        let def = skill_def(SkillId::ConsummateProfessional).unwrap();
        assert_eq!(def.class_name, "ConsummateProfessional");
    }
    #[test]
    fn consummate_professional_category_is_extraordinary() {
        let def = skill_def(SkillId::ConsummateProfessional).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn consummate_professional_is_bb2020_edition() {
        let def = skill_def(SkillId::ConsummateProfessional).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn consummate_professional_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ConsummateProfessional"), Some(SkillId::ConsummateProfessional));
    }

    // DwarfenScourge
    #[test]
    fn dwarfen_scourge_class_name_is_dwarfen_scourge() {
        let def = skill_def(SkillId::DwarfenScourge).unwrap();
        assert_eq!(def.class_name, "DwarfenScourge");
    }
    #[test]
    fn dwarfen_scourge_category_is_extraordinary() {
        let def = skill_def(SkillId::DwarfenScourge).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn dwarfen_scourge_is_bb2020_edition() {
        let def = skill_def(SkillId::DwarfenScourge).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn dwarfen_scourge_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DwarfenScourge"), Some(SkillId::DwarfenScourge));
    }

    // BalefulHex
    #[test]
    fn baleful_hex_class_name_is_baleful_hex() {
        let def = skill_def(SkillId::BalefulHex).unwrap();
        assert_eq!(def.class_name, "BalefulHex");
    }
    #[test]
    fn baleful_hex_category_is_trait() {
        let def = skill_def(SkillId::BalefulHex).unwrap();
        assert_eq!(def.category, SkillCategory::Trait);
    }
    #[test]
    fn baleful_hex_is_bb2020_edition() {
        let def = skill_def(SkillId::BalefulHex).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn baleful_hex_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BalefulHex"), Some(SkillId::BalefulHex));
    }

    // ExcuseMeAreYouAZoat
    #[test]
    fn excuse_me_are_you_a_zoat_class_name_is_excuse_me_are_you_a_zoat() {
        let def = skill_def(SkillId::ExcuseMeAreYouAZoat).unwrap();
        assert_eq!(def.class_name, "ExcuseMeAreYouAZoat");
    }
    #[test]
    fn excuse_me_are_you_a_zoat_category_is_extraordinary() {
        let def = skill_def(SkillId::ExcuseMeAreYouAZoat).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn excuse_me_are_you_a_zoat_is_bb2020_edition() {
        let def = skill_def(SkillId::ExcuseMeAreYouAZoat).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn excuse_me_are_you_a_zoat_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ExcuseMeAreYouAZoat"), Some(SkillId::ExcuseMeAreYouAZoat));
    }

    // FrenziedRush
    #[test]
    fn frenzied_rush_class_name_is_frenzied_rush() {
        let def = skill_def(SkillId::FrenziedRush).unwrap();
        assert_eq!(def.class_name, "FrenziedRush");
    }
    #[test]
    fn frenzied_rush_category_is_extraordinary() {
        let def = skill_def(SkillId::FrenziedRush).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn frenzied_rush_is_bb2020_edition() {
        let def = skill_def(SkillId::FrenziedRush).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn frenzied_rush_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("FrenziedRush"), Some(SkillId::FrenziedRush));
    }

    // GhostlyFlames
    #[test]
    fn ghostly_flames_class_name_is_ghostly_flames() {
        let def = skill_def(SkillId::GhostlyFlames).unwrap();
        assert_eq!(def.class_name, "GhostlyFlames");
    }
    #[test]
    fn ghostly_flames_category_is_extraordinary() {
        let def = skill_def(SkillId::GhostlyFlames).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn ghostly_flames_is_bb2020_edition() {
        let def = skill_def(SkillId::GhostlyFlames).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn ghostly_flames_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("GhostlyFlames"), Some(SkillId::GhostlyFlames));
    }

    // Incorporeal
    #[test]
    fn incorporeal_class_name_is_incorporeal() {
        let def = skill_def(SkillId::Incorporeal).unwrap();
        assert_eq!(def.class_name, "Incorporeal");
    }
    #[test]
    fn incorporeal_category_is_extraordinary() {
        let def = skill_def(SkillId::Incorporeal).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn incorporeal_is_bb2020_edition() {
        let def = skill_def(SkillId::Incorporeal).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn incorporeal_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Incorporeal"), Some(SkillId::Incorporeal));
    }

    // LookIntoMyEyes
    #[test]
    fn look_into_my_eyes_class_name_is_look_into_my_eyes() {
        let def = skill_def(SkillId::LookIntoMyEyes).unwrap();
        assert_eq!(def.class_name, "LookIntoMyEyes");
    }
    #[test]
    fn look_into_my_eyes_category_is_trait() {
        let def = skill_def(SkillId::LookIntoMyEyes).unwrap();
        assert_eq!(def.category, SkillCategory::Trait);
    }
    #[test]
    fn look_into_my_eyes_is_bb2020_edition() {
        let def = skill_def(SkillId::LookIntoMyEyes).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn look_into_my_eyes_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("LookIntoMyEyes"), Some(SkillId::LookIntoMyEyes));
    }

    // LordOfChaos
    #[test]
    fn lord_of_chaos_class_name_is_lord_of_chaos() {
        let def = skill_def(SkillId::LordOfChaos).unwrap();
        assert_eq!(def.class_name, "LordOfChaos");
    }
    #[test]
    fn lord_of_chaos_category_is_extraordinary() {
        let def = skill_def(SkillId::LordOfChaos).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn lord_of_chaos_is_bb2020_edition() {
        let def = skill_def(SkillId::LordOfChaos).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn lord_of_chaos_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("LordOfChaos"), Some(SkillId::LordOfChaos));
    }

    // MasterAssassin
    #[test]
    fn master_assassin_class_name_is_master_assassin() {
        let def = skill_def(SkillId::MasterAssassin).unwrap();
        assert_eq!(def.class_name, "MasterAssassin");
    }
    #[test]
    fn master_assassin_category_is_extraordinary() {
        let def = skill_def(SkillId::MasterAssassin).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn master_assassin_is_bb2020_edition() {
        let def = skill_def(SkillId::MasterAssassin).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn master_assassin_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MasterAssassin"), Some(SkillId::MasterAssassin));
    }

    // MesmerizingDance
    #[test]
    fn mesmerizing_dance_class_name_is_mesmerizing_dance() {
        let def = skill_def(SkillId::MesmerizingDance).unwrap();
        assert_eq!(def.class_name, "MesmerizingDance");
    }
    #[test]
    fn mesmerizing_dance_category_is_extraordinary() {
        let def = skill_def(SkillId::MesmerizingDance).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn mesmerizing_dance_is_bb2020_edition() {
        let def = skill_def(SkillId::MesmerizingDance).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn mesmerizing_dance_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MesmerizingDance"), Some(SkillId::MesmerizingDance));
    }

    // PumpUpTheCrowd
    #[test]
    fn pump_up_the_crowd_class_name_is_pump_up_the_crowd() {
        let def = skill_def(SkillId::PumpUpTheCrowd).unwrap();
        assert_eq!(def.class_name, "PumpUpTheCrowd");
    }
    #[test]
    fn pump_up_the_crowd_category_is_extraordinary() {
        let def = skill_def(SkillId::PumpUpTheCrowd).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn pump_up_the_crowd_is_bb2020_edition() {
        let def = skill_def(SkillId::PumpUpTheCrowd).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn pump_up_the_crowd_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PumpUpTheCrowd"), Some(SkillId::PumpUpTheCrowd));
    }

    // PutridRegurgitation
    #[test]
    fn putrid_regurgitation_class_name_is_putrid_regurgitation() {
        let def = skill_def(SkillId::PutridRegurgitation).unwrap();
        assert_eq!(def.class_name, "PutridRegurgitation");
    }
    #[test]
    fn putrid_regurgitation_category_is_extraordinary() {
        let def = skill_def(SkillId::PutridRegurgitation).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn putrid_regurgitation_is_bb2020_edition() {
        let def = skill_def(SkillId::PutridRegurgitation).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn putrid_regurgitation_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PutridRegurgitation"), Some(SkillId::PutridRegurgitation));
    }

    // TheBallista
    #[test]
    fn the_ballista_class_name_is_the_ballista() {
        let def = skill_def(SkillId::TheBallista).unwrap();
        assert_eq!(def.class_name, "TheBallista");
    }
    #[test]
    fn the_ballista_category_is_extraordinary() {
        let def = skill_def(SkillId::TheBallista).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn the_ballista_is_bb2020_edition() {
        let def = skill_def(SkillId::TheBallista).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn the_ballista_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("TheBallista"), Some(SkillId::TheBallista));
    }

    // ThenIStartedBlastin
    #[test]
    fn then_i_started_blastin_class_name_is_then_i_started_blastin() {
        let def = skill_def(SkillId::ThenIStartedBlastin).unwrap();
        assert_eq!(def.class_name, "ThenIStartedBlastin");
    }
    #[test]
    fn then_i_started_blastin_category_is_extraordinary() {
        let def = skill_def(SkillId::ThenIStartedBlastin).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn then_i_started_blastin_is_bb2020_edition() {
        let def = skill_def(SkillId::ThenIStartedBlastin).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn then_i_started_blastin_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ThenIStartedBlastin"), Some(SkillId::ThenIStartedBlastin));
    }

    // TwoForOne
    #[test]
    fn two_for_one_class_name_is_two_for_one() {
        let def = skill_def(SkillId::TwoForOne).unwrap();
        assert_eq!(def.class_name, "TwoForOne");
    }
    #[test]
    fn two_for_one_category_is_extraordinary() {
        let def = skill_def(SkillId::TwoForOne).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn two_for_one_is_bb2020_edition() {
        let def = skill_def(SkillId::TwoForOne).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn two_for_one_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("TwoForOne"), Some(SkillId::TwoForOne));
    }

    // WhirlingDervish
    #[test]
    fn whirling_dervish_class_name_is_whirling_dervish() {
        let def = skill_def(SkillId::WhirlingDervish).unwrap();
        assert_eq!(def.class_name, "WhirlingDervish");
    }
    #[test]
    fn whirling_dervish_category_is_extraordinary() {
        let def = skill_def(SkillId::WhirlingDervish).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn whirling_dervish_is_bb2020_edition() {
        let def = skill_def(SkillId::WhirlingDervish).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn whirling_dervish_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WhirlingDervish"), Some(SkillId::WhirlingDervish));
    }

    // WisdomOfTheWhiteDwarf
    #[test]
    fn wisdom_of_the_white_dwarf_class_name_is_wisdom_of_the_white_dwarf() {
        let def = skill_def(SkillId::WisdomOfTheWhiteDwarf).unwrap();
        assert_eq!(def.class_name, "WisdomOfTheWhiteDwarf");
    }
    #[test]
    fn wisdom_of_the_white_dwarf_category_is_extraordinary() {
        let def = skill_def(SkillId::WisdomOfTheWhiteDwarf).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn wisdom_of_the_white_dwarf_is_bb2020_edition() {
        let def = skill_def(SkillId::WisdomOfTheWhiteDwarf).unwrap();
        assert!(def.editions.contains(&Rules::Bb2020));
    }
    #[test]
    fn wisdom_of_the_white_dwarf_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WisdomOfTheWhiteDwarf"), Some(SkillId::WisdomOfTheWhiteDwarf));
    }

    // ── BB2016 ────────────────────────────────────────────────────────────

    // Accurate
    #[test]
    fn accurate_class_name_is_accurate() {
        let def = skill_def(SkillId::Accurate).unwrap();
        assert_eq!(def.class_name, "Accurate");
    }
    #[test]
    fn accurate_category_is_passing() {
        let def = skill_def(SkillId::Accurate).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn accurate_is_bb2016_edition() {
        let def = skill_def(SkillId::Accurate).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn accurate_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Accurate"), Some(SkillId::Accurate));
    }

    // AlwaysHungry
    #[test]
    fn always_hungry_class_name_is_always_hungry() {
        let def = skill_def(SkillId::AlwaysHungry).unwrap();
        assert_eq!(def.class_name, "AlwaysHungry");
    }
    #[test]
    fn always_hungry_category_is_extraordinary() {
        let def = skill_def(SkillId::AlwaysHungry).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn always_hungry_is_bb2016_edition() {
        let def = skill_def(SkillId::AlwaysHungry).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn always_hungry_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("AlwaysHungry"), Some(SkillId::AlwaysHungry));
    }

    // ArmourIncrease
    #[test]
    fn armour_increase_class_name_is_armour_increase() {
        let def = skill_def(SkillId::ArmourIncrease).unwrap();
        assert_eq!(def.class_name, "ArmourIncrease");
    }
    #[test]
    fn armour_increase_category_is_stat_increase() {
        let def = skill_def(SkillId::ArmourIncrease).unwrap();
        assert_eq!(def.category, SkillCategory::StatIncrease);
    }
    #[test]
    fn armour_increase_is_bb2016_edition() {
        let def = skill_def(SkillId::ArmourIncrease).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn armour_increase_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ArmourIncrease"), Some(SkillId::ArmourIncrease));
    }

    // BloodLust
    #[test]
    fn blood_lust_class_name_is_blood_lust() {
        let def = skill_def(SkillId::BloodLust).unwrap();
        assert_eq!(def.class_name, "BloodLust");
    }
    #[test]
    fn blood_lust_category_is_extraordinary() {
        let def = skill_def(SkillId::BloodLust).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn blood_lust_is_bb2016_edition() {
        let def = skill_def(SkillId::BloodLust).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn blood_lust_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BloodLust"), Some(SkillId::BloodLust));
    }

    // Claw
    #[test]
    fn claw_class_name_is_claw() {
        let def = skill_def(SkillId::Claw).unwrap();
        assert_eq!(def.class_name, "Claw");
    }
    #[test]
    fn claw_category_is_mutation() {
        let def = skill_def(SkillId::Claw).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn claw_is_bb2016_edition() {
        let def = skill_def(SkillId::Claw).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn claw_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Claw"), Some(SkillId::Claw));
    }

    // Decay
    #[test]
    fn decay_class_name_is_decay() {
        let def = skill_def(SkillId::Decay).unwrap();
        assert_eq!(def.class_name, "Decay");
    }
    #[test]
    fn decay_category_is_extraordinary() {
        let def = skill_def(SkillId::Decay).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn decay_is_bb2016_edition() {
        let def = skill_def(SkillId::Decay).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn decay_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Decay"), Some(SkillId::Decay));
    }

    // Disposable
    #[test]
    fn disposable_class_name_is_disposable() {
        let def = skill_def(SkillId::Disposable).unwrap();
        assert_eq!(def.class_name, "Disposable");
    }
    #[test]
    fn disposable_category_is_extraordinary() {
        let def = skill_def(SkillId::Disposable).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn disposable_is_bb2016_edition() {
        let def = skill_def(SkillId::Disposable).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn disposable_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Disposable"), Some(SkillId::Disposable));
    }

    // DivingTackle
    #[test]
    fn diving_tackle_class_name_is_diving_tackle() {
        let def = skill_def(SkillId::DivingTackle).unwrap();
        assert_eq!(def.class_name, "DivingTackle");
    }
    #[test]
    fn diving_tackle_category_is_agility() {
        let def = skill_def(SkillId::DivingTackle).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn diving_tackle_is_bb2016_edition() {
        let def = skill_def(SkillId::DivingTackle).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn diving_tackle_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DivingTackle"), Some(SkillId::DivingTackle));
    }

    // FanFavourite
    #[test]
    fn fan_favourite_class_name_is_fan_favourite() {
        let def = skill_def(SkillId::FanFavourite).unwrap();
        assert_eq!(def.class_name, "FanFavourite");
    }
    #[test]
    fn fan_favourite_category_is_extraordinary() {
        let def = skill_def(SkillId::FanFavourite).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn fan_favourite_is_bb2016_edition() {
        let def = skill_def(SkillId::FanFavourite).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn fan_favourite_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("FanFavourite"), Some(SkillId::FanFavourite));
    }

    // Grab
    #[test]
    fn grab_class_name_is_grab() {
        let def = skill_def(SkillId::Grab).unwrap();
        assert_eq!(def.class_name, "Grab");
    }
    #[test]
    fn grab_category_is_strength() {
        let def = skill_def(SkillId::Grab).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn grab_is_bb2016_edition() {
        let def = skill_def(SkillId::Grab).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn grab_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Grab"), Some(SkillId::Grab));
    }

    // KickOffReturn
    #[test]
    fn kick_off_return_class_name_is_kick_off_return() {
        let def = skill_def(SkillId::KickOffReturn).unwrap();
        assert_eq!(def.class_name, "KickOffReturn");
    }
    #[test]
    fn kick_off_return_category_is_agility() {
        let def = skill_def(SkillId::KickOffReturn).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn kick_off_return_is_bb2016_edition() {
        let def = skill_def(SkillId::KickOffReturn).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn kick_off_return_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("KickOffReturn"), Some(SkillId::KickOffReturn));
    }

    // KickTeamMate
    #[test]
    fn kick_team_mate_class_name_is_kick_team_mate() {
        let def = skill_def(SkillId::KickTeamMate).unwrap();
        assert_eq!(def.class_name, "KickTeamMate");
    }
    #[test]
    fn kick_team_mate_category_is_extraordinary() {
        let def = skill_def(SkillId::KickTeamMate).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn kick_team_mate_is_bb2016_edition() {
        let def = skill_def(SkillId::KickTeamMate).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn kick_team_mate_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("KickTeamMate"), Some(SkillId::KickTeamMate));
    }

    // Loner
    #[test]
    fn loner_class_name_is_loner() {
        let def = skill_def(SkillId::Loner).unwrap();
        assert_eq!(def.class_name, "Loner");
    }
    #[test]
    fn loner_category_is_extraordinary() {
        let def = skill_def(SkillId::Loner).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn loner_is_bb2016_edition() {
        let def = skill_def(SkillId::Loner).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn loner_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Loner"), Some(SkillId::Loner));
    }

    // MultipleBlock
    #[test]
    fn multiple_block_class_name_is_multiple_block() {
        let def = skill_def(SkillId::MultipleBlock).unwrap();
        assert_eq!(def.class_name, "MultipleBlock");
    }
    #[test]
    fn multiple_block_category_is_strength() {
        let def = skill_def(SkillId::MultipleBlock).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn multiple_block_is_bb2016_edition() {
        let def = skill_def(SkillId::MultipleBlock).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn multiple_block_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MultipleBlock"), Some(SkillId::MultipleBlock));
    }

    // NervesOfSteel
    #[test]
    fn nerves_of_steel_class_name_is_nerves_of_steel() {
        let def = skill_def(SkillId::NervesOfSteel).unwrap();
        assert_eq!(def.class_name, "NervesOfSteel");
    }
    #[test]
    fn nerves_of_steel_category_is_passing() {
        let def = skill_def(SkillId::NervesOfSteel).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn nerves_of_steel_is_bb2016_edition() {
        let def = skill_def(SkillId::NervesOfSteel).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn nerves_of_steel_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("NervesOfSteel"), Some(SkillId::NervesOfSteel));
    }

    // NurglesRot
    #[test]
    fn nurgles_rot_class_name_is_nurgles_rot() {
        let def = skill_def(SkillId::NurglesRot).unwrap();
        assert_eq!(def.class_name, "NurglesRot");
    }
    #[test]
    fn nurgles_rot_category_is_extraordinary() {
        let def = skill_def(SkillId::NurglesRot).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn nurgles_rot_is_bb2016_edition() {
        let def = skill_def(SkillId::NurglesRot).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn nurgles_rot_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("NurglesRot"), Some(SkillId::NurglesRot));
    }

    // PassBlock
    #[test]
    fn pass_block_class_name_is_pass_block() {
        let def = skill_def(SkillId::PassBlock).unwrap();
        assert_eq!(def.class_name, "PassBlock");
    }
    #[test]
    fn pass_block_category_is_general() {
        let def = skill_def(SkillId::PassBlock).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn pass_block_is_bb2016_edition() {
        let def = skill_def(SkillId::PassBlock).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn pass_block_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PassBlock"), Some(SkillId::PassBlock));
    }

    // PrehensileTail
    #[test]
    fn prehensile_tail_class_name_is_prehensile_tail() {
        let def = skill_def(SkillId::PrehensileTail).unwrap();
        assert_eq!(def.class_name, "PrehensileTail");
    }
    #[test]
    fn prehensile_tail_category_is_mutation() {
        let def = skill_def(SkillId::PrehensileTail).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn prehensile_tail_is_bb2016_edition() {
        let def = skill_def(SkillId::PrehensileTail).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn prehensile_tail_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PrehensileTail"), Some(SkillId::PrehensileTail));
    }

    // SafeThrow
    #[test]
    fn safe_throw_class_name_is_safe_throw() {
        let def = skill_def(SkillId::SafeThrow).unwrap();
        assert_eq!(def.class_name, "SafeThrow");
    }
    #[test]
    fn safe_throw_category_is_passing() {
        let def = skill_def(SkillId::SafeThrow).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn safe_throw_is_bb2016_edition() {
        let def = skill_def(SkillId::SafeThrow).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn safe_throw_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SafeThrow"), Some(SkillId::SafeThrow));
    }

    // SecretWeapon
    #[test]
    fn secret_weapon_class_name_is_secret_weapon() {
        let def = skill_def(SkillId::SecretWeapon).unwrap();
        assert_eq!(def.class_name, "SecretWeapon");
    }
    #[test]
    fn secret_weapon_category_is_extraordinary() {
        let def = skill_def(SkillId::SecretWeapon).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn secret_weapon_is_bb2016_edition() {
        let def = skill_def(SkillId::SecretWeapon).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn secret_weapon_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SecretWeapon"), Some(SkillId::SecretWeapon));
    }

    // Stakes
    #[test]
    fn stakes_class_name_is_stakes() {
        let def = skill_def(SkillId::Stakes).unwrap();
        assert_eq!(def.class_name, "Stakes");
    }
    #[test]
    fn stakes_category_is_extraordinary() {
        let def = skill_def(SkillId::Stakes).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn stakes_is_bb2016_edition() {
        let def = skill_def(SkillId::Stakes).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn stakes_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Stakes"), Some(SkillId::Stakes));
    }

    // StrongArm
    #[test]
    fn strong_arm_class_name_is_strong_arm() {
        let def = skill_def(SkillId::StrongArm).unwrap();
        assert_eq!(def.class_name, "StrongArm");
    }
    #[test]
    fn strong_arm_category_is_passing() {
        let def = skill_def(SkillId::StrongArm).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn strong_arm_is_bb2016_edition() {
        let def = skill_def(SkillId::StrongArm).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn strong_arm_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("StrongArm"), Some(SkillId::StrongArm));
    }

    // Stunty
    #[test]
    fn stunty_class_name_is_stunty() {
        let def = skill_def(SkillId::Stunty).unwrap();
        assert_eq!(def.class_name, "Stunty");
    }
    #[test]
    fn stunty_category_is_extraordinary() {
        let def = skill_def(SkillId::Stunty).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn stunty_is_bb2016_edition() {
        let def = skill_def(SkillId::Stunty).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn stunty_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Stunty"), Some(SkillId::Stunty));
    }

    // TakeRoot
    #[test]
    fn take_root_class_name_is_take_root() {
        let def = skill_def(SkillId::TakeRoot).unwrap();
        assert_eq!(def.class_name, "TakeRoot");
    }
    #[test]
    fn take_root_category_is_extraordinary() {
        let def = skill_def(SkillId::TakeRoot).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn take_root_is_bb2016_edition() {
        let def = skill_def(SkillId::TakeRoot).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn take_root_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("TakeRoot"), Some(SkillId::TakeRoot));
    }

    // ThrowTeamMate
    #[test]
    fn throw_team_mate_class_name_is_throw_team_mate() {
        let def = skill_def(SkillId::ThrowTeamMate).unwrap();
        assert_eq!(def.class_name, "ThrowTeamMate");
    }
    #[test]
    fn throw_team_mate_category_is_extraordinary() {
        let def = skill_def(SkillId::ThrowTeamMate).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn throw_team_mate_is_bb2016_edition() {
        let def = skill_def(SkillId::ThrowTeamMate).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn throw_team_mate_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ThrowTeamMate"), Some(SkillId::ThrowTeamMate));
    }

    // Timmmber
    #[test]
    fn timmmber_class_name_is_timmmber() {
        let def = skill_def(SkillId::Timmmber).unwrap();
        assert_eq!(def.class_name, "Timmmber");
    }
    #[test]
    fn timmmber_category_is_extraordinary() {
        let def = skill_def(SkillId::Timmmber).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn timmmber_is_bb2016_edition() {
        let def = skill_def(SkillId::Timmmber).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn timmmber_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Timmmber"), Some(SkillId::Timmmber));
    }

    // Titchy
    #[test]
    fn titchy_class_name_is_titchy() {
        let def = skill_def(SkillId::Titchy).unwrap();
        assert_eq!(def.class_name, "Titchy");
    }
    #[test]
    fn titchy_category_is_extraordinary() {
        let def = skill_def(SkillId::Titchy).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn titchy_is_bb2016_edition() {
        let def = skill_def(SkillId::Titchy).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn titchy_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Titchy"), Some(SkillId::Titchy));
    }

    // WeepingDagger
    #[test]
    fn weeping_dagger_class_name_is_weeping_dagger() {
        let def = skill_def(SkillId::WeepingDagger).unwrap();
        assert_eq!(def.class_name, "WeepingDagger");
    }
    #[test]
    fn weeping_dagger_category_is_extraordinary() {
        let def = skill_def(SkillId::WeepingDagger).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn weeping_dagger_is_bb2016_edition() {
        let def = skill_def(SkillId::WeepingDagger).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn weeping_dagger_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WeepingDagger"), Some(SkillId::WeepingDagger));
    }

    // WildAnimal
    #[test]
    fn wild_animal_class_name_is_wild_animal() {
        let def = skill_def(SkillId::WildAnimal).unwrap();
        assert_eq!(def.class_name, "WildAnimal");
    }
    #[test]
    fn wild_animal_category_is_general() {
        let def = skill_def(SkillId::WildAnimal).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn wild_animal_is_bb2016_edition() {
        let def = skill_def(SkillId::WildAnimal).unwrap();
        assert!(def.editions.contains(&Rules::Bb2016));
    }
    #[test]
    fn wild_animal_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WildAnimal"), Some(SkillId::WildAnimal));
    }

    // ── BB2025 ────────────────────────────────────────────────────────────

    // AgilityIncrease
    #[test]
    fn agility_increase_class_name_is_agility_increase() {
        let def = skill_def(SkillId::AgilityIncrease).unwrap();
        assert_eq!(def.class_name, "AgilityIncrease");
    }
    #[test]
    fn agility_increase_category_is_stat_increase() {
        let def = skill_def(SkillId::AgilityIncrease).unwrap();
        assert_eq!(def.category, SkillCategory::StatIncrease);
    }
    #[test]
    fn agility_increase_is_bb2025_edition() {
        let def = skill_def(SkillId::AgilityIncrease).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn agility_increase_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("AgilityIncrease"), Some(SkillId::AgilityIncrease));
    }

    // BigHand
    #[test]
    fn big_hand_class_name_is_big_hand() {
        let def = skill_def(SkillId::BigHand).unwrap();
        assert_eq!(def.class_name, "BigHand");
    }
    #[test]
    fn big_hand_category_is_mutation() {
        let def = skill_def(SkillId::BigHand).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn big_hand_is_bb2025_edition() {
        let def = skill_def(SkillId::BigHand).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn big_hand_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BigHand"), Some(SkillId::BigHand));
    }

    // Bullseye
    #[test]
    fn bullseye_class_name_is_bullseye() {
        let def = skill_def(SkillId::Bullseye).unwrap();
        assert_eq!(def.class_name, "Bullseye");
    }
    #[test]
    fn bullseye_category_is_passing() {
        let def = skill_def(SkillId::Bullseye).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn bullseye_is_bb2025_edition() {
        let def = skill_def(SkillId::Bullseye).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn bullseye_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Bullseye"), Some(SkillId::Bullseye));
    }

    // EyeGouge
    #[test]
    fn eye_gouge_class_name_is_eye_gouge() {
        let def = skill_def(SkillId::EyeGouge).unwrap();
        assert_eq!(def.class_name, "EyeGouge");
    }
    #[test]
    fn eye_gouge_category_is_strength() {
        let def = skill_def(SkillId::EyeGouge).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn eye_gouge_is_bb2025_edition() {
        let def = skill_def(SkillId::EyeGouge).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn eye_gouge_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("EyeGouge"), Some(SkillId::EyeGouge));
    }

    // Fumblerooski
    #[test]
    fn fumblerooski_class_name_is_fumblerooski() {
        let def = skill_def(SkillId::Fumblerooski).unwrap();
        assert_eq!(def.class_name, "Fumblerooski");
    }
    #[test]
    fn fumblerooski_category_is_passing() {
        let def = skill_def(SkillId::Fumblerooski).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn fumblerooski_is_bb2025_edition() {
        let def = skill_def(SkillId::Fumblerooski).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn fumblerooski_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Fumblerooski"), Some(SkillId::Fumblerooski));
    }

    // GiveAndGo
    #[test]
    fn give_and_go_class_name_is_give_and_go() {
        let def = skill_def(SkillId::GiveAndGo).unwrap();
        assert_eq!(def.class_name, "GiveAndGo");
    }
    #[test]
    fn give_and_go_category_is_agility() {
        let def = skill_def(SkillId::GiveAndGo).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn give_and_go_is_bb2025_edition() {
        let def = skill_def(SkillId::GiveAndGo).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn give_and_go_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("GiveAndGo"), Some(SkillId::GiveAndGo));
    }

    // Hatred
    #[test]
    fn hatred_class_name_is_hatred() {
        let def = skill_def(SkillId::Hatred).unwrap();
        assert_eq!(def.class_name, "Hatred");
    }
    #[test]
    fn hatred_category_is_general() {
        let def = skill_def(SkillId::Hatred).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn hatred_is_bb2025_edition() {
        let def = skill_def(SkillId::Hatred).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn hatred_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Hatred"), Some(SkillId::Hatred));
    }

    // Insignificant
    #[test]
    fn insignificant_class_name_is_insignificant() {
        let def = skill_def(SkillId::Insignificant).unwrap();
        assert_eq!(def.class_name, "Insignificant");
    }
    #[test]
    fn insignificant_category_is_extraordinary() {
        let def = skill_def(SkillId::Insignificant).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn insignificant_is_bb2025_edition() {
        let def = skill_def(SkillId::Insignificant).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn insignificant_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Insignificant"), Some(SkillId::Insignificant));
    }

    // Juggernaut
    #[test]
    fn juggernaut_class_name_is_juggernaut() {
        let def = skill_def(SkillId::Juggernaut).unwrap();
        assert_eq!(def.class_name, "Juggernaut");
    }
    #[test]
    fn juggernaut_category_is_strength() {
        let def = skill_def(SkillId::Juggernaut).unwrap();
        assert_eq!(def.category, SkillCategory::Strength);
    }
    #[test]
    fn juggernaut_is_bb2025_edition() {
        let def = skill_def(SkillId::Juggernaut).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn juggernaut_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Juggernaut"), Some(SkillId::Juggernaut));
    }

    // Kick
    #[test]
    fn kick_class_name_is_kick() {
        let def = skill_def(SkillId::Kick).unwrap();
        assert_eq!(def.class_name, "Kick");
    }
    #[test]
    fn kick_category_is_general() {
        let def = skill_def(SkillId::Kick).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn kick_is_bb2025_edition() {
        let def = skill_def(SkillId::Kick).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn kick_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Kick"), Some(SkillId::Kick));
    }

    // Leader
    #[test]
    fn leader_class_name_is_leader() {
        let def = skill_def(SkillId::Leader).unwrap();
        assert_eq!(def.class_name, "Leader");
    }
    #[test]
    fn leader_category_is_general() {
        let def = skill_def(SkillId::Leader).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn leader_is_bb2025_edition() {
        let def = skill_def(SkillId::Leader).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn leader_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Leader"), Some(SkillId::Leader));
    }

    // LethalFlight
    #[test]
    fn lethal_flight_class_name_is_lethal_flight() {
        let def = skill_def(SkillId::LethalFlight).unwrap();
        assert_eq!(def.class_name, "LethalFlight");
    }
    #[test]
    fn lethal_flight_category_is_extraordinary() {
        let def = skill_def(SkillId::LethalFlight).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn lethal_flight_is_bb2025_edition() {
        let def = skill_def(SkillId::LethalFlight).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn lethal_flight_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("LethalFlight"), Some(SkillId::LethalFlight));
    }

    // LoneFouler
    #[test]
    fn lone_fouler_class_name_is_lone_fouler() {
        let def = skill_def(SkillId::LoneFouler).unwrap();
        assert_eq!(def.class_name, "LoneFouler");
    }
    #[test]
    fn lone_fouler_category_is_agility() {
        let def = skill_def(SkillId::LoneFouler).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn lone_fouler_is_bb2025_edition() {
        let def = skill_def(SkillId::LoneFouler).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn lone_fouler_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("LoneFouler"), Some(SkillId::LoneFouler));
    }

    // NoBall
    #[test]
    fn no_ball_class_name_is_no_ball() {
        let def = skill_def(SkillId::NoBall).unwrap();
        assert_eq!(def.class_name, "NoBall");
    }
    #[test]
    fn no_ball_category_is_extraordinary() {
        let def = skill_def(SkillId::NoBall).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn no_ball_is_bb2025_edition() {
        let def = skill_def(SkillId::NoBall).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn no_ball_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("NoBall"), Some(SkillId::NoBall));
    }

    // Pogo
    #[test]
    fn pogo_class_name_is_pogo() {
        let def = skill_def(SkillId::Pogo).unwrap();
        assert_eq!(def.class_name, "Pogo");
    }
    #[test]
    fn pogo_category_is_agility() {
        let def = skill_def(SkillId::Pogo).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn pogo_is_bb2025_edition() {
        let def = skill_def(SkillId::Pogo).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn pogo_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Pogo"), Some(SkillId::Pogo));
    }

    // Pro
    #[test]
    fn pro_class_name_is_pro() {
        let def = skill_def(SkillId::Pro).unwrap();
        assert_eq!(def.class_name, "Pro");
    }
    #[test]
    fn pro_category_is_general() {
        let def = skill_def(SkillId::Pro).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn pro_is_bb2025_edition() {
        let def = skill_def(SkillId::Pro).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn pro_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Pro"), Some(SkillId::Pro));
    }

    // Punt
    #[test]
    fn punt_class_name_is_punt() {
        let def = skill_def(SkillId::Punt).unwrap();
        assert_eq!(def.class_name, "Punt");
    }
    #[test]
    fn punt_category_is_passing() {
        let def = skill_def(SkillId::Punt).unwrap();
        assert_eq!(def.category, SkillCategory::Passing);
    }
    #[test]
    fn punt_is_bb2025_edition() {
        let def = skill_def(SkillId::Punt).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn punt_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Punt"), Some(SkillId::Punt));
    }

    // PutTheBootIn
    #[test]
    fn put_the_boot_in_class_name_is_put_the_boot_in() {
        let def = skill_def(SkillId::PutTheBootIn).unwrap();
        assert_eq!(def.class_name, "PutTheBootIn");
    }
    #[test]
    fn put_the_boot_in_category_is_general() {
        let def = skill_def(SkillId::PutTheBootIn).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn put_the_boot_in_is_bb2025_edition() {
        let def = skill_def(SkillId::PutTheBootIn).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn put_the_boot_in_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("PutTheBootIn"), Some(SkillId::PutTheBootIn));
    }

    // QuickFoul
    #[test]
    fn quick_foul_class_name_is_quick_foul() {
        let def = skill_def(SkillId::QuickFoul).unwrap();
        assert_eq!(def.class_name, "QuickFoul");
    }
    #[test]
    fn quick_foul_category_is_agility() {
        let def = skill_def(SkillId::QuickFoul).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn quick_foul_is_bb2025_edition() {
        let def = skill_def(SkillId::QuickFoul).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn quick_foul_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("QuickFoul"), Some(SkillId::QuickFoul));
    }

    // Saboteur
    #[test]
    fn saboteur_class_name_is_saboteur() {
        let def = skill_def(SkillId::Saboteur).unwrap();
        assert_eq!(def.class_name, "Saboteur");
    }
    #[test]
    fn saboteur_category_is_extraordinary() {
        let def = skill_def(SkillId::Saboteur).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn saboteur_is_bb2025_edition() {
        let def = skill_def(SkillId::Saboteur).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn saboteur_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Saboteur"), Some(SkillId::Saboteur));
    }

    // Sidestep (BB2025 variant)
    #[test]
    fn sidestep_class_name_is_sidestep() {
        let def = skill_def(SkillId::Sidestep).unwrap();
        assert_eq!(def.class_name, "Sidestep");
    }
    #[test]
    fn sidestep_category_is_agility() {
        let def = skill_def(SkillId::Sidestep).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn sidestep_is_bb2025_edition() {
        let def = skill_def(SkillId::Sidestep).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn sidestep_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Sidestep"), Some(SkillId::Sidestep));
    }

    // BlastinSolvesEverything
    #[test]
    fn blastin_solves_everything_class_name_is_blastin_solves_everything() {
        let def = skill_def(SkillId::BlastinSolvesEverything).unwrap();
        assert_eq!(def.class_name, "BlastinSolvesEverything");
    }
    #[test]
    fn blastin_solves_everything_category_is_extraordinary() {
        let def = skill_def(SkillId::BlastinSolvesEverything).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn blastin_solves_everything_is_bb2025_edition() {
        let def = skill_def(SkillId::BlastinSolvesEverything).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn blastin_solves_everything_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("BlastinSolvesEverything"), Some(SkillId::BlastinSolvesEverything));
    }

    // DwarvenScourge
    #[test]
    fn dwarven_scourge_class_name_is_dwarven_scourge() {
        let def = skill_def(SkillId::DwarvenScourge).unwrap();
        assert_eq!(def.class_name, "DwarvenScourge");
    }
    #[test]
    fn dwarven_scourge_category_is_extraordinary() {
        let def = skill_def(SkillId::DwarvenScourge).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn dwarven_scourge_is_bb2025_edition() {
        let def = skill_def(SkillId::DwarvenScourge).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn dwarven_scourge_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("DwarvenScourge"), Some(SkillId::DwarvenScourge));
    }

    // KrumpAndSmash
    #[test]
    fn krump_and_smash_class_name_is_krump_and_smash() {
        let def = skill_def(SkillId::KrumpAndSmash).unwrap();
        assert_eq!(def.class_name, "KrumpAndSmash");
    }
    #[test]
    fn krump_and_smash_category_is_extraordinary() {
        let def = skill_def(SkillId::KrumpAndSmash).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn krump_and_smash_is_bb2025_edition() {
        let def = skill_def(SkillId::KrumpAndSmash).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn krump_and_smash_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("KrumpAndSmash"), Some(SkillId::KrumpAndSmash));
    }

    // MesmerisingDance (BB2025 British spelling)
    #[test]
    fn mesmerising_dance_class_name_is_mesmerising_dance() {
        let def = skill_def(SkillId::MesmerisingDance).unwrap();
        assert_eq!(def.class_name, "MesmerisingDance");
    }
    #[test]
    fn mesmerising_dance_category_is_extraordinary() {
        let def = skill_def(SkillId::MesmerisingDance).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn mesmerising_dance_is_bb2025_edition() {
        let def = skill_def(SkillId::MesmerisingDance).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn mesmerising_dance_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("MesmerisingDance"), Some(SkillId::MesmerisingDance));
    }

    // SlashingNails
    #[test]
    fn slashing_nails_class_name_is_slashing_nails() {
        let def = skill_def(SkillId::SlashingNails).unwrap();
        assert_eq!(def.class_name, "SlashingNails");
    }
    #[test]
    fn slashing_nails_category_is_mutation() {
        let def = skill_def(SkillId::SlashingNails).unwrap();
        assert_eq!(def.category, SkillCategory::Mutation);
    }
    #[test]
    fn slashing_nails_is_bb2025_edition() {
        let def = skill_def(SkillId::SlashingNails).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn slashing_nails_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SlashingNails"), Some(SkillId::SlashingNails));
    }

    // SteadyFooting
    #[test]
    fn steady_footing_class_name_is_steady_footing() {
        let def = skill_def(SkillId::SteadyFooting).unwrap();
        assert_eq!(def.class_name, "SteadyFooting");
    }
    #[test]
    fn steady_footing_category_is_agility() {
        let def = skill_def(SkillId::SteadyFooting).unwrap();
        assert_eq!(def.category, SkillCategory::Agility);
    }
    #[test]
    fn steady_footing_is_bb2025_edition() {
        let def = skill_def(SkillId::SteadyFooting).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn steady_footing_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("SteadyFooting"), Some(SkillId::SteadyFooting));
    }

    // TeamCaptain
    #[test]
    fn team_captain_class_name_is_team_captain() {
        let def = skill_def(SkillId::TeamCaptain).unwrap();
        assert_eq!(def.class_name, "TeamCaptain");
    }
    #[test]
    fn team_captain_category_is_general() {
        let def = skill_def(SkillId::TeamCaptain).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn team_captain_is_bb2025_edition() {
        let def = skill_def(SkillId::TeamCaptain).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn team_captain_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("TeamCaptain"), Some(SkillId::TeamCaptain));
    }

    // Taunt
    #[test]
    fn taunt_class_name_is_taunt() {
        let def = skill_def(SkillId::Taunt).unwrap();
        assert_eq!(def.class_name, "Taunt");
    }
    #[test]
    fn taunt_category_is_general() {
        let def = skill_def(SkillId::Taunt).unwrap();
        assert_eq!(def.category, SkillCategory::General);
    }
    #[test]
    fn taunt_is_bb2025_edition() {
        let def = skill_def(SkillId::Taunt).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn taunt_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Taunt"), Some(SkillId::Taunt));
    }

    // Unsteady
    #[test]
    fn unsteady_class_name_is_unsteady() {
        let def = skill_def(SkillId::Unsteady).unwrap();
        assert_eq!(def.class_name, "Unsteady");
    }
    #[test]
    fn unsteady_category_is_extraordinary() {
        let def = skill_def(SkillId::Unsteady).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn unsteady_is_bb2025_edition() {
        let def = skill_def(SkillId::Unsteady).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn unsteady_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("Unsteady"), Some(SkillId::Unsteady));
    }

    // ViolentInnovator
    #[test]
    fn violent_innovator_class_name_is_violent_innovator() {
        let def = skill_def(SkillId::ViolentInnovator).unwrap();
        assert_eq!(def.class_name, "ViolentInnovator");
    }
    #[test]
    fn violent_innovator_category_is_extraordinary() {
        let def = skill_def(SkillId::ViolentInnovator).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn violent_innovator_is_bb2025_edition() {
        let def = skill_def(SkillId::ViolentInnovator).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn violent_innovator_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("ViolentInnovator"), Some(SkillId::ViolentInnovator));
    }

    // WoodlandFury
    #[test]
    fn woodland_fury_class_name_is_woodland_fury() {
        let def = skill_def(SkillId::WoodlandFury).unwrap();
        assert_eq!(def.class_name, "WoodlandFury");
    }
    #[test]
    fn woodland_fury_category_is_extraordinary() {
        let def = skill_def(SkillId::WoodlandFury).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn woodland_fury_is_bb2025_edition() {
        let def = skill_def(SkillId::WoodlandFury).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn woodland_fury_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WoodlandFury"), Some(SkillId::WoodlandFury));
    }

    // WorkingInTandem
    #[test]
    fn working_in_tandem_class_name_is_working_in_tandem() {
        let def = skill_def(SkillId::WorkingInTandem).unwrap();
        assert_eq!(def.class_name, "WorkingInTandem");
    }
    #[test]
    fn working_in_tandem_category_is_extraordinary() {
        let def = skill_def(SkillId::WorkingInTandem).unwrap();
        assert_eq!(def.category, SkillCategory::Extraordinary);
    }
    #[test]
    fn working_in_tandem_is_bb2025_edition() {
        let def = skill_def(SkillId::WorkingInTandem).unwrap();
        assert!(def.editions.contains(&Rules::Bb2025));
    }
    #[test]
    fn working_in_tandem_lookup_by_class_name() {
        assert_eq!(SkillId::from_class_name("WorkingInTandem"), Some(SkillId::WorkingInTandem));
    }

    // AllYouCanEat
    #[test] fn all_you_can_eat_class_name() { assert_eq!(skill_def(SkillId::AllYouCanEat).unwrap().class_name, "AllYouCanEat"); }
    #[test] fn all_you_can_eat_category_is_trait() { assert_eq!(skill_def(SkillId::AllYouCanEat).unwrap().category, SkillCategory::Trait); }
    #[test] fn all_you_can_eat_is_bb2020_and_bb2025() { let d = skill_def(SkillId::AllYouCanEat).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn all_you_can_eat_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("AllYouCanEat"), Some(SkillId::AllYouCanEat)); }

    // BeerBarrelBash
    #[test] fn beer_barrel_bash_class_name() { assert_eq!(skill_def(SkillId::BeerBarrelBash).unwrap().class_name, "BeerBarrelBash"); }
    #[test] fn beer_barrel_bash_category_is_trait() { assert_eq!(skill_def(SkillId::BeerBarrelBash).unwrap().category, SkillCategory::Trait); }
    #[test] fn beer_barrel_bash_is_bb2020_and_bb2025() { let d = skill_def(SkillId::BeerBarrelBash).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn beer_barrel_bash_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("BeerBarrelBash"), Some(SkillId::BeerBarrelBash)); }

    // BlackInk
    #[test] fn black_ink_class_name() { assert_eq!(skill_def(SkillId::BlackInk).unwrap().class_name, "BlackInk"); }
    #[test] fn black_ink_category_is_trait() { assert_eq!(skill_def(SkillId::BlackInk).unwrap().category, SkillCategory::Trait); }
    #[test] fn black_ink_is_bb2020_and_bb2025() { let d = skill_def(SkillId::BlackInk).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn black_ink_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("BlackInk"), Some(SkillId::BlackInk)); }

    // BlindRage
    #[test] fn blind_rage_class_name() { assert_eq!(skill_def(SkillId::BlindRage).unwrap().class_name, "BlindRage"); }
    #[test] fn blind_rage_category_is_trait() { assert_eq!(skill_def(SkillId::BlindRage).unwrap().category, SkillCategory::Trait); }
    #[test] fn blind_rage_is_bb2020_and_bb2025() { let d = skill_def(SkillId::BlindRage).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn blind_rage_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("BlindRage"), Some(SkillId::BlindRage)); }

    // BoundingLeap
    #[test] fn bounding_leap_class_name() { assert_eq!(skill_def(SkillId::BoundingLeap).unwrap().class_name, "BoundingLeap"); }
    #[test] fn bounding_leap_category_is_trait() { assert_eq!(skill_def(SkillId::BoundingLeap).unwrap().category, SkillCategory::Trait); }
    #[test] fn bounding_leap_is_bb2020_and_bb2025() { let d = skill_def(SkillId::BoundingLeap).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn bounding_leap_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("BoundingLeap"), Some(SkillId::BoundingLeap)); }

    // BugmansXXXXXX
    #[test] fn bugmans_xxxxxx_class_name() { assert_eq!(skill_def(SkillId::BugmansXXXXXX).unwrap().class_name, "BugmansXXXXXX"); }
    #[test] fn bugmans_xxxxxx_category_is_trait() { assert_eq!(skill_def(SkillId::BugmansXXXXXX).unwrap().category, SkillCategory::Trait); }
    #[test] fn bugmans_xxxxxx_is_bb2020_and_bb2025() { let d = skill_def(SkillId::BugmansXXXXXX).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn bugmans_xxxxxx_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("BugmansXXXXXX"), Some(SkillId::BugmansXXXXXX)); }

    // CatchOfTheDay
    #[test] fn catch_of_the_day_class_name() { assert_eq!(skill_def(SkillId::CatchOfTheDay).unwrap().class_name, "CatchOfTheDay"); }
    #[test] fn catch_of_the_day_category_is_trait() { assert_eq!(skill_def(SkillId::CatchOfTheDay).unwrap().category, SkillCategory::Trait); }
    #[test] fn catch_of_the_day_is_bb2020_and_bb2025() { let d = skill_def(SkillId::CatchOfTheDay).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn catch_of_the_day_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("CatchOfTheDay"), Some(SkillId::CatchOfTheDay)); }

    // CrushingBlow
    #[test] fn crushing_blow_class_name() { assert_eq!(skill_def(SkillId::CrushingBlow).unwrap().class_name, "CrushingBlow"); }
    #[test] fn crushing_blow_category_is_trait() { assert_eq!(skill_def(SkillId::CrushingBlow).unwrap().category, SkillCategory::Trait); }
    #[test] fn crushing_blow_is_bb2020_and_bb2025() { let d = skill_def(SkillId::CrushingBlow).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn crushing_blow_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("CrushingBlow"), Some(SkillId::CrushingBlow)); }

    // Drunkard
    #[test] fn drunkard_class_name() { assert_eq!(skill_def(SkillId::Drunkard).unwrap().class_name, "Drunkard"); }
    #[test] fn drunkard_category_is_trait() { assert_eq!(skill_def(SkillId::Drunkard).unwrap().category, SkillCategory::Trait); }
    #[test] fn drunkard_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Drunkard).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn drunkard_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Drunkard"), Some(SkillId::Drunkard)); }

    // FuriousOutburst
    #[test] fn furious_outburst_class_name() { assert_eq!(skill_def(SkillId::FuriousOutburst).unwrap().class_name, "FuriousOutburst"); }
    #[test] fn furious_outburst_category_is_trait() { assert_eq!(skill_def(SkillId::FuriousOutburst).unwrap().category, SkillCategory::Trait); }
    #[test] fn furious_outburst_is_bb2020_and_bb2025() { let d = skill_def(SkillId::FuriousOutburst).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn furious_outburst_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("FuriousOutburst"), Some(SkillId::FuriousOutburst)); }

    // FuryOfTheBloodGod
    #[test] fn fury_of_the_blood_god_class_name() { assert_eq!(skill_def(SkillId::FuryOfTheBloodGod).unwrap().class_name, "FuryOfTheBloodGod"); }
    #[test] fn fury_of_the_blood_god_category_is_trait() { assert_eq!(skill_def(SkillId::FuryOfTheBloodGod).unwrap().category, SkillCategory::Trait); }
    #[test] fn fury_of_the_blood_god_is_bb2020_and_bb2025() { let d = skill_def(SkillId::FuryOfTheBloodGod).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn fury_of_the_blood_god_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("FuryOfTheBloodGod"), Some(SkillId::FuryOfTheBloodGod)); }

    // GoredByTheBull
    #[test] fn gored_by_the_bull_class_name() { assert_eq!(skill_def(SkillId::GoredByTheBull).unwrap().class_name, "GoredByTheBull"); }
    #[test] fn gored_by_the_bull_category_is_trait() { assert_eq!(skill_def(SkillId::GoredByTheBull).unwrap().category, SkillCategory::Trait); }
    #[test] fn gored_by_the_bull_is_bb2020_and_bb2025() { let d = skill_def(SkillId::GoredByTheBull).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn gored_by_the_bull_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("GoredByTheBull"), Some(SkillId::GoredByTheBull)); }

    // HalflingLuck
    #[test] fn halfling_luck_class_name() { assert_eq!(skill_def(SkillId::HalflingLuck).unwrap().class_name, "HalflingLuck"); }
    #[test] fn halfling_luck_category_is_trait() { assert_eq!(skill_def(SkillId::HalflingLuck).unwrap().category, SkillCategory::Trait); }
    #[test] fn halfling_luck_is_bb2020_and_bb2025() { let d = skill_def(SkillId::HalflingLuck).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn halfling_luck_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("HalflingLuck"), Some(SkillId::HalflingLuck)); }

    // IllBeBack
    #[test] fn ill_be_back_class_name() { assert_eq!(skill_def(SkillId::IllBeBack).unwrap().class_name, "IllBeBack"); }
    #[test] fn ill_be_back_category_is_trait() { assert_eq!(skill_def(SkillId::IllBeBack).unwrap().category, SkillCategory::Trait); }
    #[test] fn ill_be_back_is_bb2020_and_bb2025() { let d = skill_def(SkillId::IllBeBack).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn ill_be_back_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("IllBeBack"), Some(SkillId::IllBeBack)); }

    // Indomitable
    #[test] fn indomitable_class_name() { assert_eq!(skill_def(SkillId::Indomitable).unwrap().class_name, "Indomitable"); }
    #[test] fn indomitable_category_is_trait() { assert_eq!(skill_def(SkillId::Indomitable).unwrap().category, SkillCategory::Trait); }
    #[test] fn indomitable_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Indomitable).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn indomitable_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Indomitable"), Some(SkillId::Indomitable)); }

    // Kaboom
    #[test] fn kaboom_class_name() { assert_eq!(skill_def(SkillId::Kaboom).unwrap().class_name, "Kaboom"); }
    #[test] fn kaboom_category_is_trait() { assert_eq!(skill_def(SkillId::Kaboom).unwrap().category, SkillCategory::Trait); }
    #[test] fn kaboom_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Kaboom).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn kaboom_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Kaboom"), Some(SkillId::Kaboom)); }

    // KeenPlayer
    #[test] fn keen_player_class_name() { assert_eq!(skill_def(SkillId::KeenPlayer).unwrap().class_name, "KeenPlayer"); }
    #[test] fn keen_player_category_is_trait() { assert_eq!(skill_def(SkillId::KeenPlayer).unwrap().category, SkillCategory::Trait); }
    #[test] fn keen_player_is_bb2020_and_bb2025() { let d = skill_def(SkillId::KeenPlayer).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn keen_player_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("KeenPlayer"), Some(SkillId::KeenPlayer)); }

    // KickEmWhileTheyReDown
    #[test] fn kick_em_while_theyre_down_class_name() { assert_eq!(skill_def(SkillId::KickEmWhileTheyReDown).unwrap().class_name, "KickEmWhileTheyReDown"); }
    #[test] fn kick_em_while_theyre_down_category_is_trait() { assert_eq!(skill_def(SkillId::KickEmWhileTheyReDown).unwrap().category, SkillCategory::Trait); }
    #[test] fn kick_em_while_theyre_down_is_bb2020_and_bb2025() { let d = skill_def(SkillId::KickEmWhileTheyReDown).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn kick_em_while_theyre_down_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("KickEmWhileTheyReDown"), Some(SkillId::KickEmWhileTheyReDown)); }

    // MaximumCarnage
    #[test] fn maximum_carnage_class_name() { assert_eq!(skill_def(SkillId::MaximumCarnage).unwrap().class_name, "MaximumCarnage"); }
    #[test] fn maximum_carnage_category_is_trait() { assert_eq!(skill_def(SkillId::MaximumCarnage).unwrap().category, SkillCategory::Trait); }
    #[test] fn maximum_carnage_is_bb2020_and_bb2025() { let d = skill_def(SkillId::MaximumCarnage).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn maximum_carnage_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("MaximumCarnage"), Some(SkillId::MaximumCarnage)); }

    // OldPro
    #[test] fn old_pro_class_name() { assert_eq!(skill_def(SkillId::OldPro).unwrap().class_name, "OldPro"); }
    #[test] fn old_pro_category_is_trait() { assert_eq!(skill_def(SkillId::OldPro).unwrap().category, SkillCategory::Trait); }
    #[test] fn old_pro_is_bb2020_and_bb2025() { let d = skill_def(SkillId::OldPro).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn old_pro_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("OldPro"), Some(SkillId::OldPro)); }

    // PlagueRidden
    #[test] fn plague_ridden_class_name() { assert_eq!(skill_def(SkillId::PlagueRidden).unwrap().class_name, "PlagueRidden"); }
    #[test] fn plague_ridden_category_is_trait() { assert_eq!(skill_def(SkillId::PlagueRidden).unwrap().category, SkillCategory::Trait); }
    #[test] fn plague_ridden_is_bb2020_and_bb2025() { let d = skill_def(SkillId::PlagueRidden).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn plague_ridden_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("PlagueRidden"), Some(SkillId::PlagueRidden)); }

    // PrimalSavagery
    #[test] fn primal_savagery_class_name() { assert_eq!(skill_def(SkillId::PrimalSavagery).unwrap().class_name, "PrimalSavagery"); }
    #[test] fn primal_savagery_category_is_trait() { assert_eq!(skill_def(SkillId::PrimalSavagery).unwrap().category, SkillCategory::Trait); }
    #[test] fn primal_savagery_is_bb2020_and_bb2025() { let d = skill_def(SkillId::PrimalSavagery).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn primal_savagery_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("PrimalSavagery"), Some(SkillId::PrimalSavagery)); }

    // QuickBite
    #[test] fn quick_bite_class_name() { assert_eq!(skill_def(SkillId::QuickBite).unwrap().class_name, "QuickBite"); }
    #[test] fn quick_bite_category_is_trait() { assert_eq!(skill_def(SkillId::QuickBite).unwrap().category, SkillCategory::Trait); }
    #[test] fn quick_bite_is_bb2020_and_bb2025() { let d = skill_def(SkillId::QuickBite).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn quick_bite_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("QuickBite"), Some(SkillId::QuickBite)); }

    // RaidingParty
    #[test] fn raiding_party_class_name() { assert_eq!(skill_def(SkillId::RaidingParty).unwrap().class_name, "RaidingParty"); }
    #[test] fn raiding_party_category_is_trait() { assert_eq!(skill_def(SkillId::RaidingParty).unwrap().category, SkillCategory::Trait); }
    #[test] fn raiding_party_is_bb2020_and_bb2025() { let d = skill_def(SkillId::RaidingParty).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn raiding_party_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("RaidingParty"), Some(SkillId::RaidingParty)); }

    // Ram
    #[test] fn ram_class_name() { assert_eq!(skill_def(SkillId::Ram).unwrap().class_name, "Ram"); }
    #[test] fn ram_category_is_trait() { assert_eq!(skill_def(SkillId::Ram).unwrap().category, SkillCategory::Trait); }
    #[test] fn ram_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Ram).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn ram_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Ram"), Some(SkillId::Ram)); }

    // Reliable
    #[test] fn reliable_class_name() { assert_eq!(skill_def(SkillId::Reliable).unwrap().class_name, "Reliable"); }
    #[test] fn reliable_category_is_trait() { assert_eq!(skill_def(SkillId::Reliable).unwrap().category, SkillCategory::Trait); }
    #[test] fn reliable_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Reliable).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn reliable_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Reliable"), Some(SkillId::Reliable)); }

    // SavageBlow
    #[test] fn savage_blow_class_name() { assert_eq!(skill_def(SkillId::SavageBlow).unwrap().class_name, "SavageBlow"); }
    #[test] fn savage_blow_category_is_trait() { assert_eq!(skill_def(SkillId::SavageBlow).unwrap().category, SkillCategory::Trait); }
    #[test] fn savage_blow_is_bb2020_and_bb2025() { let d = skill_def(SkillId::SavageBlow).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn savage_blow_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("SavageBlow"), Some(SkillId::SavageBlow)); }

    // SavageMauling
    #[test] fn savage_mauling_class_name() { assert_eq!(skill_def(SkillId::SavageMauling).unwrap().class_name, "SavageMauling"); }
    #[test] fn savage_mauling_category_is_trait() { assert_eq!(skill_def(SkillId::SavageMauling).unwrap().category, SkillCategory::Trait); }
    #[test] fn savage_mauling_is_bb2020_and_bb2025() { let d = skill_def(SkillId::SavageMauling).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn savage_mauling_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("SavageMauling"), Some(SkillId::SavageMauling)); }

    // ShotToNothing
    #[test] fn shot_to_nothing_class_name() { assert_eq!(skill_def(SkillId::ShotToNothing).unwrap().class_name, "ShotToNothing"); }
    #[test] fn shot_to_nothing_category_is_trait() { assert_eq!(skill_def(SkillId::ShotToNothing).unwrap().category, SkillCategory::Trait); }
    #[test] fn shot_to_nothing_is_bb2020_and_bb2025() { let d = skill_def(SkillId::ShotToNothing).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn shot_to_nothing_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("ShotToNothing"), Some(SkillId::ShotToNothing)); }

    // SneakiestOfTheLot
    #[test] fn sneakiest_of_the_lot_class_name() { assert_eq!(skill_def(SkillId::SneakiestOfTheLot).unwrap().class_name, "SneakiestOfTheLot"); }
    #[test] fn sneakiest_of_the_lot_category_is_trait() { assert_eq!(skill_def(SkillId::SneakiestOfTheLot).unwrap().category, SkillCategory::Trait); }
    #[test] fn sneakiest_of_the_lot_is_bb2020_and_bb2025() { let d = skill_def(SkillId::SneakiestOfTheLot).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn sneakiest_of_the_lot_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("SneakiestOfTheLot"), Some(SkillId::SneakiestOfTheLot)); }

    // StarOfTheShow
    #[test] fn star_of_the_show_class_name() { assert_eq!(skill_def(SkillId::StarOfTheShow).unwrap().class_name, "StarOfTheShow"); }
    #[test] fn star_of_the_show_category_is_trait() { assert_eq!(skill_def(SkillId::StarOfTheShow).unwrap().category, SkillCategory::Trait); }
    #[test] fn star_of_the_show_is_bb2020_and_bb2025() { let d = skill_def(SkillId::StarOfTheShow).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn star_of_the_show_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("StarOfTheShow"), Some(SkillId::StarOfTheShow)); }

    // StrongPassingGame
    #[test] fn strong_passing_game_class_name() { assert_eq!(skill_def(SkillId::StrongPassingGame).unwrap().class_name, "StrongPassingGame"); }
    #[test] fn strong_passing_game_category_is_trait() { assert_eq!(skill_def(SkillId::StrongPassingGame).unwrap().category, SkillCategory::Trait); }
    #[test] fn strong_passing_game_is_bb2020_and_bb2025() { let d = skill_def(SkillId::StrongPassingGame).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn strong_passing_game_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("StrongPassingGame"), Some(SkillId::StrongPassingGame)); }

    // SwiftAsTheBreeze
    #[test] fn swift_as_the_breeze_class_name() { assert_eq!(skill_def(SkillId::SwiftAsTheBreeze).unwrap().class_name, "SwiftAsTheBreeze"); }
    #[test] fn swift_as_the_breeze_category_is_trait() { assert_eq!(skill_def(SkillId::SwiftAsTheBreeze).unwrap().category, SkillCategory::Trait); }
    #[test] fn swift_as_the_breeze_is_bb2020_and_bb2025() { let d = skill_def(SkillId::SwiftAsTheBreeze).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn swift_as_the_breeze_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("SwiftAsTheBreeze"), Some(SkillId::SwiftAsTheBreeze)); }

    // TastyMorsel
    #[test] fn tasty_morsel_class_name() { assert_eq!(skill_def(SkillId::TastyMorsel).unwrap().class_name, "TastyMorsel"); }
    #[test] fn tasty_morsel_category_is_trait() { assert_eq!(skill_def(SkillId::TastyMorsel).unwrap().category, SkillCategory::Trait); }
    #[test] fn tasty_morsel_is_bb2020_and_bb2025() { let d = skill_def(SkillId::TastyMorsel).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn tasty_morsel_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("TastyMorsel"), Some(SkillId::TastyMorsel)); }

    // TheFlashingBlade
    #[test] fn the_flashing_blade_class_name() { assert_eq!(skill_def(SkillId::TheFlashingBlade).unwrap().class_name, "TheFlashingBlade"); }
    #[test] fn the_flashing_blade_category_is_trait() { assert_eq!(skill_def(SkillId::TheFlashingBlade).unwrap().category, SkillCategory::Trait); }
    #[test] fn the_flashing_blade_is_bb2020_and_bb2025() { let d = skill_def(SkillId::TheFlashingBlade).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn the_flashing_blade_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("TheFlashingBlade"), Some(SkillId::TheFlashingBlade)); }

    // ThinkingMansTroll
    #[test] fn thinking_mans_troll_class_name() { assert_eq!(skill_def(SkillId::ThinkingMansTroll).unwrap().class_name, "ThinkingMansTroll"); }
    #[test] fn thinking_mans_troll_category_is_trait() { assert_eq!(skill_def(SkillId::ThinkingMansTroll).unwrap().category, SkillCategory::Trait); }
    #[test] fn thinking_mans_troll_is_bb2020_and_bb2025() { let d = skill_def(SkillId::ThinkingMansTroll).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn thinking_mans_troll_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("ThinkingMansTroll"), Some(SkillId::ThinkingMansTroll)); }

    // Treacherous
    #[test] fn treacherous_class_name() { assert_eq!(skill_def(SkillId::Treacherous).unwrap().class_name, "Treacherous"); }
    #[test] fn treacherous_category_is_trait() { assert_eq!(skill_def(SkillId::Treacherous).unwrap().category, SkillCategory::Trait); }
    #[test] fn treacherous_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Treacherous).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn treacherous_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Treacherous"), Some(SkillId::Treacherous)); }

    // Trickster
    #[test] fn trickster_class_name() { assert_eq!(skill_def(SkillId::Trickster).unwrap().class_name, "Trickster"); }
    #[test] fn trickster_category_is_trait() { assert_eq!(skill_def(SkillId::Trickster).unwrap().category, SkillCategory::Trait); }
    #[test] fn trickster_is_bb2020_and_bb2025() { let d = skill_def(SkillId::Trickster).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn trickster_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("Trickster"), Some(SkillId::Trickster)); }

    // UnstoppableMomentum
    #[test] fn unstoppable_momentum_class_name() { assert_eq!(skill_def(SkillId::UnstoppableMomentum).unwrap().class_name, "UnstoppableMomentum"); }
    #[test] fn unstoppable_momentum_category_is_trait() { assert_eq!(skill_def(SkillId::UnstoppableMomentum).unwrap().category, SkillCategory::Trait); }
    #[test] fn unstoppable_momentum_is_bb2020_and_bb2025() { let d = skill_def(SkillId::UnstoppableMomentum).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn unstoppable_momentum_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("UnstoppableMomentum"), Some(SkillId::UnstoppableMomentum)); }

    // ViciousVines
    #[test] fn vicious_vines_class_name() { assert_eq!(skill_def(SkillId::ViciousVines).unwrap().class_name, "ViciousVines"); }
    #[test] fn vicious_vines_category_is_trait() { assert_eq!(skill_def(SkillId::ViciousVines).unwrap().category, SkillCategory::Trait); }
    #[test] fn vicious_vines_is_bb2020_and_bb2025() { let d = skill_def(SkillId::ViciousVines).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn vicious_vines_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("ViciousVines"), Some(SkillId::ViciousVines)); }

    // WatchOut
    #[test] fn watch_out_class_name() { assert_eq!(skill_def(SkillId::WatchOut).unwrap().class_name, "WatchOut"); }
    #[test] fn watch_out_category_is_trait() { assert_eq!(skill_def(SkillId::WatchOut).unwrap().category, SkillCategory::Trait); }
    #[test] fn watch_out_is_bb2020_and_bb2025() { let d = skill_def(SkillId::WatchOut).unwrap(); assert!(d.editions.contains(&Rules::Bb2020) && d.editions.contains(&Rules::Bb2025)); }
    #[test] fn watch_out_lookup_by_class_name() { assert_eq!(SkillId::from_class_name("WatchOut"), Some(SkillId::WatchOut)); }
}
