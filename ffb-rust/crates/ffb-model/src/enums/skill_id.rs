use serde::{Deserialize, Serialize};

/// Unique identifier for a Blood Bowl skill (all editions combined).
///
/// Variants correspond 1-to-1 with Java skill class names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillId {
    // ── Common (all editions) ───────────────────────────────────────────────
    Block,
    Catch,
    Dauntless,
    DisturbingPresence,
    DivingCatch,
    DumpOff,
    ExtraArms,
    Fend,
    FoulAppearance,
    HailMaryPass,
    Horns,
    JumpUp,
    MovementIncrease,
    Pass,
    Sprint,
    StandFirm,
    StripBall,
    SureHands,
    Tackle,
    Tentacles,
    ThickSkull,
    TwoHeads,
    Wrestle,

    // ── BB2020 & BB2025 skills ──────────────────────────────────────────────
    AnimalSavagery,
    Animosity,
    BallAndChain,
    Bombardier,
    BoneHead,
    Brawler,
    BreakTackle,
    BreatheFire,
    Chainsaw,
    CloudBurster,
    Defensive,
    DirtyPlayer,
    Fumblerooskie,
    HitAndRun,
    HypnoticGaze,
    Leap,
    MightyBlow,
    MonstrousMouth,
    NoHands,
    PassingIncrease,
    PileDriver,
    PilingOn,
    PogoStick,
    ProjectileVomit,
    ReallyStupid,
    Regeneration,
    RightStuff,
    RunningPass,
    Shadowing,
    SideStep,
    SneakyGit,
    Stab,
    StrengthIncrease,
    SureFeet,
    Swarming,
    Swoop,
    VeryLongLegs,

    // ── BB2020 special skills ───────────────────────────────────────────────
    ASneakyPair,
    BlastIt,
    BrutalBlock,
    BurstOfSpeed,
    ConsummateProfessional,
    DwarfenScourge,
    ExcuseMeAreYouAZoat,
    FrenziedRush,
    GhostlyFlames,
    Incorporeal,
    LordOfChaos,
    MasterAssassin,
    MesmerizingDance,
    PumpUpTheCrowd,
    PutridRegurgitation,
    TheBallista,
    ThenIStartedBlastin,
    TwoForOne,
    WhirlingDervish,
    WisdomOfTheWhiteDwarf,
    Yoink,

    // ── Mixed (multiple editions) ──────────────────────────────────────────
    ArmBar,
    BalefulHex,
    Cannoneer,
    IronHardSkin,
    LookIntoMyEyes,
    MyBall,
    PickMeUp,
    SafePass,
    SafePairOfHands,
    Slayer,
    ToxinConnoisseur,
    UnchannelledFury,

    // ── BB2016-only skills ──────────────────────────────────────────────────
    Accurate,
    AlwaysHungry,
    ArmourIncrease,
    BloodLust,
    Claw,
    Decay,
    Disposable,
    DivingTackle,
    FanFavourite,
    Frenzy,
    Grab,
    Guard,
    KickOffReturn,
    KickTeamMate,
    Loner,
    MultipleBlock,
    NervesOfSteel,
    NurglesRot,
    PassBlock,
    PrehensileTail,
    SafeThrow,
    SecretWeapon,
    Stakes,
    StrongArm,
    Stunty,
    TakeRoot,
    ThrowTeamMate,
    Timmmber,
    Titchy,
    WeepingDagger,
    WildAnimal,

    // ── BB2025-only skills ──────────────────────────────────────────────────
    AgilityIncrease,
    BigHand,
    Bullseye,
    Dodge,
    EyeGouge,
    Fumblerooski,
    GiveAndGo,
    Hatred,
    Insignificant,
    Juggernaut,
    Kick,
    Leader,
    LethalFlight,
    LoneFouler,
    NoBall,
    OnTheBall,
    Pogo,
    Pro,
    Punt,
    PutTheBootIn,
    QuickFoul,
    Saboteur,
    Sidestep,
    BlastinSolvesEverything,
    DwarvenScourge,
    KrumpAndSmash,
    MesmerisingDance,
    SlashingNails,
    SteadyFooting,
    TeamCaptain,
    Taunt,
    Unsteady,
    ViolentInnovator,
    WoodlandFury,
    WorkingInTandem,

    // ── Mixed/special star player traits (BB2020 + BB2025) ────────────────────
    AllYouCanEat,
    BeerBarrelBash,
    BlackInk,
    BlindRage,
    BoundingLeap,
    BugmansXXXXXX,
    CatchOfTheDay,
    CrushingBlow,
    Drunkard,
    FuriousOutburst,
    FuryOfTheBloodGod,
    GoredByTheBull,
    HalflingLuck,
    IllBeBack,
    Indomitable,
    Kaboom,
    KeenPlayer,
    KickEmWhileTheyReDown,
    MaximumCarnage,
    OldPro,
    PlagueRidden,
    PrimalSavagery,
    QuickBite,
    RaidingParty,
    Ram,
    Reliable,
    SavageBlow,
    SavageMauling,
    ShotToNothing,
    SneakiestOfTheLot,
    StarOfTheShow,
    StrongPassingGame,
    SwiftAsTheBreeze,
    TastyMorsel,
    TheFlashingBlade,
    ThinkingMansTroll,
    Treacherous,
    Trickster,
    UnstoppableMomentum,
    ViciousVines,
    WatchOut,
}

impl SkillId {
    /// The Java class name used for this skill (matches JSON `class_name` field).
    pub fn class_name(self) -> &'static str {
        match self {
            SkillId::Block => "Block",
            SkillId::Catch => "Catch",
            SkillId::Dauntless => "Dauntless",
            SkillId::DisturbingPresence => "DisturbingPresence",
            SkillId::DivingCatch => "DivingCatch",
            SkillId::DumpOff => "DumpOff",
            SkillId::ExtraArms => "ExtraArms",
            SkillId::Fend => "Fend",
            SkillId::FoulAppearance => "FoulAppearance",
            SkillId::HailMaryPass => "HailMaryPass",
            SkillId::Horns => "Horns",
            SkillId::JumpUp => "JumpUp",
            SkillId::MovementIncrease => "MovementIncrease",
            SkillId::Pass => "Pass",
            SkillId::Sprint => "Sprint",
            SkillId::StandFirm => "StandFirm",
            SkillId::StripBall => "StripBall",
            SkillId::SureHands => "SureHands",
            SkillId::Tackle => "Tackle",
            SkillId::Tentacles => "Tentacles",
            SkillId::ThickSkull => "ThickSkull",
            SkillId::TwoHeads => "TwoHeads",
            SkillId::Wrestle => "Wrestle",
            SkillId::AnimalSavagery => "AnimalSavagery",
            SkillId::Animosity => "Animosity",
            SkillId::BallAndChain => "BallAndChain",
            SkillId::Bombardier => "Bombardier",
            SkillId::BoneHead => "BoneHead",
            SkillId::Brawler => "Brawler",
            SkillId::BreakTackle => "BreakTackle",
            SkillId::BreatheFire => "BreatheFire",
            SkillId::Chainsaw => "Chainsaw",
            SkillId::CloudBurster => "CloudBurster",
            SkillId::Defensive => "Defensive",
            SkillId::DirtyPlayer => "DirtyPlayer",
            SkillId::Fumblerooskie => "Fumblerooskie",
            SkillId::HitAndRun => "HitAndRun",
            SkillId::HypnoticGaze => "HypnoticGaze",
            SkillId::Leap => "Leap",
            SkillId::MightyBlow => "MightyBlow",
            SkillId::MonstrousMouth => "MonstrousMouth",
            SkillId::NoHands => "NoHands",
            SkillId::PassingIncrease => "PassingIncrease",
            SkillId::PileDriver => "PileDriver",
            SkillId::PilingOn => "PilingOn",
            SkillId::PogoStick => "PogoStick",
            SkillId::ProjectileVomit => "ProjectileVomit",
            SkillId::ReallyStupid => "ReallyStupid",
            SkillId::Regeneration => "Regeneration",
            SkillId::RightStuff => "RightStuff",
            SkillId::RunningPass => "RunningPass",
            SkillId::Shadowing => "Shadowing",
            SkillId::SideStep => "SideStep",
            SkillId::SneakyGit => "SneakyGit",
            SkillId::Stab => "Stab",
            SkillId::StrengthIncrease => "StrengthIncrease",
            SkillId::SureFeet => "SureFeet",
            SkillId::Swarming => "Swarming",
            SkillId::Swoop => "Swoop",
            SkillId::VeryLongLegs => "VeryLongLegs",
            SkillId::ASneakyPair => "ASneakyPair",
            SkillId::BlastIt => "BlastIt",
            SkillId::BrutalBlock => "BrutalBlock",
            SkillId::BurstOfSpeed => "BurstOfSpeed",
            SkillId::ConsummateProfessional => "ConsummateProfessional",
            SkillId::DwarfenScourge => "DwarfenScourge",
            SkillId::ExcuseMeAreYouAZoat => "ExcuseMeAreYouAZoat",
            SkillId::FrenziedRush => "FrenziedRush",
            SkillId::GhostlyFlames => "GhostlyFlames",
            SkillId::Incorporeal => "Incorporeal",
            SkillId::LordOfChaos => "LordOfChaos",
            SkillId::MasterAssassin => "MasterAssassin",
            SkillId::MesmerizingDance => "MesmerizingDance",
            SkillId::PumpUpTheCrowd => "PumpUpTheCrowd",
            SkillId::PutridRegurgitation => "PutridRegurgitation",
            SkillId::TheBallista => "TheBallista",
            SkillId::ThenIStartedBlastin => "ThenIStartedBlastin",
            SkillId::TwoForOne => "TwoForOne",
            SkillId::WhirlingDervish => "WhirlingDervish",
            SkillId::WisdomOfTheWhiteDwarf => "WisdomOfTheWhiteDwarf",
            SkillId::Yoink => "Yoink",
            SkillId::Accurate => "Accurate",
            SkillId::AlwaysHungry => "AlwaysHungry",
            SkillId::ArmourIncrease => "ArmourIncrease",
            SkillId::BloodLust => "BloodLust",
            SkillId::Claw => "Claw",
            SkillId::Decay => "Decay",
            SkillId::Disposable => "Disposable",
            SkillId::DivingTackle => "DivingTackle",
            SkillId::FanFavourite => "FanFavourite",
            SkillId::Frenzy => "Frenzy",
            SkillId::Grab => "Grab",
            SkillId::Guard => "Guard",
            SkillId::KickOffReturn => "KickOffReturn",
            SkillId::KickTeamMate => "KickTeamMate",
            SkillId::Loner => "Loner",
            SkillId::MultipleBlock => "MultipleBlock",
            SkillId::NervesOfSteel => "NervesOfSteel",
            SkillId::NurglesRot => "NurglesRot",
            SkillId::PassBlock => "PassBlock",
            SkillId::PrehensileTail => "PrehensileTail",
            SkillId::ArmBar => "ArmBar",
            SkillId::BalefulHex => "BalefulHex",
            SkillId::Cannoneer => "Cannoneer",
            SkillId::LookIntoMyEyes => "LookIntoMyEyes",
            SkillId::IronHardSkin => "IronHardSkin",
            SkillId::MyBall => "MyBall",
            SkillId::PickMeUp => "PickMeUp",
            SkillId::SafePass => "SafePass",
            SkillId::SafePairOfHands => "SafePairOfHands",
            SkillId::Slayer => "Slayer",
            SkillId::ToxinConnoisseur => "ToxinConnoisseur",
            SkillId::UnchannelledFury => "UnchannelledFury",
            SkillId::SafeThrow => "SafeThrow",
            SkillId::SecretWeapon => "SecretWeapon",
            SkillId::Stakes => "Stakes",
            SkillId::StrongArm => "StrongArm",
            SkillId::Stunty => "Stunty",
            SkillId::TakeRoot => "TakeRoot",
            SkillId::ThrowTeamMate => "ThrowTeamMate",
            SkillId::Timmmber => "Timmmber",
            SkillId::Titchy => "Titchy",
            SkillId::WeepingDagger => "WeepingDagger",
            SkillId::WildAnimal => "WildAnimal",
            SkillId::AgilityIncrease => "AgilityIncrease",
            SkillId::BigHand => "BigHand",
            SkillId::Bullseye => "Bullseye",
            SkillId::Dodge => "Dodge",
            SkillId::EyeGouge => "EyeGouge",
            SkillId::Fumblerooski => "Fumblerooski",
            SkillId::GiveAndGo => "GiveAndGo",
            SkillId::Hatred => "Hatred",
            SkillId::Insignificant => "Insignificant",
            SkillId::Juggernaut => "Juggernaut",
            SkillId::Kick => "Kick",
            SkillId::Leader => "Leader",
            SkillId::LethalFlight => "LethalFlight",
            SkillId::LoneFouler => "LoneFouler",
            SkillId::NoBall => "NoBall",
            SkillId::OnTheBall => "OnTheBall",
            SkillId::Pogo => "Pogo",
            SkillId::Pro => "Pro",
            SkillId::Punt => "Punt",
            SkillId::PutTheBootIn => "PutTheBootIn",
            SkillId::QuickFoul => "QuickFoul",
            SkillId::Saboteur => "Saboteur",
            SkillId::Sidestep => "Sidestep",
            SkillId::BlastinSolvesEverything => "BlastinSolvesEverything",
            SkillId::DwarvenScourge => "DwarvenScourge",
            SkillId::KrumpAndSmash => "KrumpAndSmash",
            SkillId::MesmerisingDance => "MesmerisingDance",
            SkillId::SlashingNails => "SlashingNails",
            SkillId::SteadyFooting => "SteadyFooting",
            SkillId::TeamCaptain => "TeamCaptain",
            SkillId::Taunt => "Taunt",
            SkillId::Unsteady => "Unsteady",
            SkillId::ViolentInnovator => "ViolentInnovator",
            SkillId::WoodlandFury => "WoodlandFury",
            SkillId::WorkingInTandem => "WorkingInTandem",
            SkillId::Wrestle => "Wrestle",
            SkillId::AllYouCanEat => "AllYouCanEat",
            SkillId::BeerBarrelBash => "BeerBarrelBash",
            SkillId::BlackInk => "BlackInk",
            SkillId::BlindRage => "BlindRage",
            SkillId::BoundingLeap => "BoundingLeap",
            SkillId::BugmansXXXXXX => "BugmansXXXXXX",
            SkillId::CatchOfTheDay => "CatchOfTheDay",
            SkillId::CrushingBlow => "CrushingBlow",
            SkillId::Drunkard => "Drunkard",
            SkillId::FuriousOutburst => "FuriousOutburst",
            SkillId::FuryOfTheBloodGod => "FuryOfTheBloodGod",
            SkillId::GoredByTheBull => "GoredByTheBull",
            SkillId::HalflingLuck => "HalflingLuck",
            SkillId::IllBeBack => "IllBeBack",
            SkillId::Indomitable => "Indomitable",
            SkillId::Kaboom => "Kaboom",
            SkillId::KeenPlayer => "KeenPlayer",
            SkillId::KickEmWhileTheyReDown => "KickEmWhileTheyReDown",
            SkillId::MaximumCarnage => "MaximumCarnage",
            SkillId::OldPro => "OldPro",
            SkillId::PlagueRidden => "PlagueRidden",
            SkillId::PrimalSavagery => "PrimalSavagery",
            SkillId::QuickBite => "QuickBite",
            SkillId::RaidingParty => "RaidingParty",
            SkillId::Ram => "Ram",
            SkillId::Reliable => "Reliable",
            SkillId::SavageBlow => "SavageBlow",
            SkillId::SavageMauling => "SavageMauling",
            SkillId::ShotToNothing => "ShotToNothing",
            SkillId::SneakiestOfTheLot => "SneakiestOfTheLot",
            SkillId::StarOfTheShow => "StarOfTheShow",
            SkillId::StrongPassingGame => "StrongPassingGame",
            SkillId::SwiftAsTheBreeze => "SwiftAsTheBreeze",
            SkillId::TastyMorsel => "TastyMorsel",
            SkillId::TheFlashingBlade => "TheFlashingBlade",
            SkillId::ThinkingMansTroll => "ThinkingMansTroll",
            SkillId::Treacherous => "Treacherous",
            SkillId::Trickster => "Trickster",
            SkillId::UnstoppableMomentum => "UnstoppableMomentum",
            SkillId::ViciousVines => "ViciousVines",
            SkillId::WatchOut => "WatchOut",
        }
    }

    /// Java: Skill.getSkillUsageType() — returns the usage tracking type for this skill.
    pub fn usage_type(self) -> crate::enums::SkillUsageType {
        use crate::enums::SkillUsageType::*;
        match self {
            // OncePerDrive (mixed): BeerBarrelBash, RaidingParty
            SkillId::BeerBarrelBash | SkillId::RaidingParty => OncePerDrive,

            // OncePerHalf (bb2020): GhostlyFlames, ThenIStartedBlastin
            SkillId::GhostlyFlames | SkillId::ThenIStartedBlastin => OncePerHalf,
            // OncePerHalf (bb2025): Leader, BlastinSolvesEverything, MesmerisingDance, SlashingNails
            SkillId::Leader | SkillId::BlastinSolvesEverything | SkillId::MesmerisingDance | SkillId::SlashingNails => OncePerHalf,
            // OncePerHalf (mixed): CatchOfTheDay, FuriousOutburst
            SkillId::CatchOfTheDay | SkillId::FuriousOutburst => OncePerHalf,
            // FrenziedRush/PutridRegurgitation: bb2020=OncePerGame, bb2025=OncePerHalf — use bb2025
            SkillId::FrenziedRush | SkillId::PutridRegurgitation => OncePerHalf,

            // OncePerTurn (bb2025): Dodge, Pro, LoneFouler, SureFeet
            SkillId::Dodge | SkillId::Pro | SkillId::LoneFouler | SkillId::SureFeet => OncePerTurn,
            // OncePerTurnByTeamMate: WisdomOfTheWhiteDwarf, Swoop
            SkillId::WisdomOfTheWhiteDwarf | SkillId::Swoop => OncePerTurnByTeamMate,

            // OncePerGame (bb2020)
            SkillId::BlastIt | SkillId::BrutalBlock | SkillId::BurstOfSpeed |
            SkillId::ConsummateProfessional | SkillId::DwarfenScourge |
            SkillId::ExcuseMeAreYouAZoat | SkillId::Incorporeal | SkillId::LordOfChaos |
            SkillId::MasterAssassin | SkillId::MesmerizingDance | SkillId::PumpUpTheCrowd |
            SkillId::TheBallista => OncePerGame,
            // OncePerGame (bb2025): DwarvenScourge, KrumpAndSmash + bb2025 versions of shared ones
            SkillId::DwarvenScourge | SkillId::KrumpAndSmash => OncePerGame,
            // OncePerGame (mixed)
            SkillId::AllYouCanEat | SkillId::BalefulHex | SkillId::BlackInk |
            SkillId::BoundingLeap | SkillId::CrushingBlow | SkillId::FuryOfTheBloodGod |
            SkillId::GoredByTheBull | SkillId::HalflingLuck | SkillId::Indomitable |
            SkillId::Kaboom | SkillId::KickEmWhileTheyReDown | SkillId::LookIntoMyEyes |
            SkillId::MaximumCarnage | SkillId::OldPro | SkillId::PrimalSavagery |
            SkillId::QuickBite => OncePerGame,

            // All other skills are Regular
            _ => Regular,
        }
    }

    /// Parse from a Java class name string OR a human-readable skill name.
    ///
    /// Normalizes the input by stripping all non-alphanumeric characters and
    /// lowercasing before matching, so "Secret Weapon", "secret_weapon",
    /// "SecretWeapon", and "secret weapon" all resolve identically.
    pub fn from_class_name(s: &str) -> Option<SkillId> {
        // Exact case-sensitive match first: distinguishes pairs like "SideStep" (BB2020)
        // vs "Sidestep" (BB2025) that normalize to the same lowercase key.
        let exact = match s {
            "SideStep" => return Some(SkillId::SideStep),
            "Sidestep" => return Some(SkillId::Sidestep),
            _ => None::<SkillId>,
        };
        let _ = exact; // suppress warning
        let n: String = s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
        let skill = match n.as_str() {
            "accurate" => SkillId::Accurate,
            "agilityincrease" => SkillId::AgilityIncrease,
            "alwayshungry" => SkillId::AlwaysHungry,
            "animalsavagery" => SkillId::AnimalSavagery,
            "animosity" => SkillId::Animosity,
            "armourincrease" => SkillId::ArmourIncrease,
            "asneakypair" => SkillId::ASneakyPair,
            "ballandchain" => SkillId::BallAndChain,
            "bighand" => SkillId::BigHand,
            "blastinsolveseverything" => SkillId::BlastinSolvesEverything,
            "blastit" => SkillId::BlastIt,
            "bloodlust" => SkillId::BloodLust,
            "block" => SkillId::Block,
            "bombardier" => SkillId::Bombardier,
            "bonehead" => SkillId::BoneHead,
            "brawler" => SkillId::Brawler,
            "breaktackle" => SkillId::BreakTackle,
            "breathefire" => SkillId::BreatheFire,
            "brutalblock" => SkillId::BrutalBlock,
            "bullseye" => SkillId::Bullseye,
            "burstofspeed" => SkillId::BurstOfSpeed,
            "catch" => SkillId::Catch,
            "chainsaw" => SkillId::Chainsaw,
            "claw" | "claws" => SkillId::Claw,
            "cloudburster" => SkillId::CloudBurster,
            "consummateprofessional" => SkillId::ConsummateProfessional,
            "dauntless" => SkillId::Dauntless,
            "decay" => SkillId::Decay,
            "defensive" => SkillId::Defensive,
            "dirtyplayer" => SkillId::DirtyPlayer,
            "disposable" => SkillId::Disposable,
            "disturbingpresence" => SkillId::DisturbingPresence,
            "divingcatch" => SkillId::DivingCatch,
            "divingtackle" => SkillId::DivingTackle,
            "dodge" => SkillId::Dodge,
            "dumpoff" => SkillId::DumpOff,
            "dwarfenscourge" => SkillId::DwarfenScourge,
            "dwarvenscourge" => SkillId::DwarvenScourge,
            "excusemeareyouazoat" => SkillId::ExcuseMeAreYouAZoat,
            "extraarms" => SkillId::ExtraArms,
            "eyegouge" => SkillId::EyeGouge,
            "fanfavourite" => SkillId::FanFavourite,
            "fend" => SkillId::Fend,
            "foulappearance" => SkillId::FoulAppearance,
            "frenzy" => SkillId::Frenzy,
            "frenziedrush" => SkillId::FrenziedRush,
            "fumblerooski" => SkillId::Fumblerooski,
            "fumblerooskie" => SkillId::Fumblerooskie,
            "ghostlyflames" => SkillId::GhostlyFlames,
            "giveandgo" => SkillId::GiveAndGo,
            "grab" => SkillId::Grab,
            "guard" => SkillId::Guard,
            "hailmarypass" => SkillId::HailMaryPass,
            "hatred" => SkillId::Hatred,
            "hitandrun" => SkillId::HitAndRun,
            "horns" => SkillId::Horns,
            "hypnoticgaze" => SkillId::HypnoticGaze,
            "incorporeal" => SkillId::Incorporeal,
            "insignificant" => SkillId::Insignificant,
            "juggernaut" => SkillId::Juggernaut,
            "jumpup" => SkillId::JumpUp,
            "kick" => SkillId::Kick,
            "kickoffreturn" => SkillId::KickOffReturn,
            "kickteammate" => SkillId::KickTeamMate,
            "krumpandsmash" => SkillId::KrumpAndSmash,
            "leader" => SkillId::Leader,
            "leap" => SkillId::Leap,
            "lethalflight" => SkillId::LethalFlight,
            "lonefouler" => SkillId::LoneFouler,
            "loner" => SkillId::Loner,
            "lordofchaos" => SkillId::LordOfChaos,
            "masterassassin" => SkillId::MasterAssassin,
            "mesmerizingdance" => SkillId::MesmerizingDance,
            "mesmerisingdance" => SkillId::MesmerisingDance,
            "mightyblow" => SkillId::MightyBlow,
            "monstrousmouth" => SkillId::MonstrousMouth,
            "movementincrease" => SkillId::MovementIncrease,
            "multipleblock" => SkillId::MultipleBlock,
            "nervesofsteel" => SkillId::NervesOfSteel,
            "noball" => SkillId::NoBall,
            "ontheball" => SkillId::OnTheBall,
            "nohands" => SkillId::NoHands,
            "nurglesrot" => SkillId::NurglesRot,
            "pass" => SkillId::Pass,
            "passblock" => SkillId::PassBlock,
            "passingincrease" => SkillId::PassingIncrease,
            "piledriver" => SkillId::PileDriver,
            "pilingon" => SkillId::PilingOn,
            "pogo" => SkillId::Pogo,
            "pogostick" => SkillId::PogoStick,
            "prehensiletail" => SkillId::PrehensileTail,
            "pro" => SkillId::Pro,
            "projectilevomit" => SkillId::ProjectileVomit,
            "pumpupthecrowd" => SkillId::PumpUpTheCrowd,
            "punt" => SkillId::Punt,
            "putridregurgitation" => SkillId::PutridRegurgitation,
            "putthebootin" => SkillId::PutTheBootIn,
            "quickfoul" => SkillId::QuickFoul,
            "reallystupid" => SkillId::ReallyStupid,
            "regeneration" => SkillId::Regeneration,
            "rightstuff" => SkillId::RightStuff,
            "runningpass" => SkillId::RunningPass,
            "saboteur" => SkillId::Saboteur,
            "armbar" => SkillId::ArmBar,
            "balefulhex" => SkillId::BalefulHex,
            "cannoneer" => SkillId::Cannoneer,
            "lookintomyeyes" => SkillId::LookIntoMyEyes,
            "ironhardskin" => SkillId::IronHardSkin,
            "myball" => SkillId::MyBall,
            "pickmeup" => SkillId::PickMeUp,
            "safepass" => SkillId::SafePass,
            "safepairofhands" => SkillId::SafePairOfHands,
            "slayer" => SkillId::Slayer,
            "toxinconnoisseur" => SkillId::ToxinConnoisseur,
            "unchannelledfury" => SkillId::UnchannelledFury,
            "safethrow" => SkillId::SafeThrow,
            "secretweapon" => SkillId::SecretWeapon,
            "shadowing" => SkillId::Shadowing,
            "sidestep" => SkillId::SideStep,
            "slashingnails" => SkillId::SlashingNails,
            "sneakygit" => SkillId::SneakyGit,
            "sprint" => SkillId::Sprint,
            "stab" => SkillId::Stab,
            "stakes" => SkillId::Stakes,
            "standfirm" => SkillId::StandFirm,
            "steadyfooting" => SkillId::SteadyFooting,
            "strengthincrease" => SkillId::StrengthIncrease,
            "stripball" => SkillId::StripBall,
            "strongarm" => SkillId::StrongArm,
            "stunty" => SkillId::Stunty,
            "surefeet" => SkillId::SureFeet,
            "surehands" => SkillId::SureHands,
            "swarming" => SkillId::Swarming,
            "swoop" => SkillId::Swoop,
            "tackle" => SkillId::Tackle,
            "takeroot" => SkillId::TakeRoot,
            "taunt" => SkillId::Taunt,
            "teamcaptain" => SkillId::TeamCaptain,
            "tentacles" => SkillId::Tentacles,
            "theballista" => SkillId::TheBallista,
            "thenistartedblastin" => SkillId::ThenIStartedBlastin,
            "thickskull" => SkillId::ThickSkull,
            "throwteammate" => SkillId::ThrowTeamMate,
            "timmmber" => SkillId::Timmmber,
            "titchy" => SkillId::Titchy,
            "twoforone" => SkillId::TwoForOne,
            "twoheads" => SkillId::TwoHeads,
            "unsteady" => SkillId::Unsteady,
            "verylonglegs" => SkillId::VeryLongLegs,
            "violentinnovator" => SkillId::ViolentInnovator,
            "weepingdagger" => SkillId::WeepingDagger,
            "whirlingdervish" => SkillId::WhirlingDervish,
            "wildanimal" => SkillId::WildAnimal,
            "wisdomofthewhitedwarf" => SkillId::WisdomOfTheWhiteDwarf,
            "woodlandfury" => SkillId::WoodlandFury,
            "yoink" => SkillId::Yoink,
            "workingintandem" => SkillId::WorkingInTandem,
            "wrestle" => SkillId::Wrestle,
            "allyoucaneat" => SkillId::AllYouCanEat,
            "beerbarrelbash" => SkillId::BeerBarrelBash,
            "blackink" => SkillId::BlackInk,
            "blindrage" => SkillId::BlindRage,
            "boundingleap" => SkillId::BoundingLeap,
            "bugmansxxxxxx" => SkillId::BugmansXXXXXX,
            "catchoftheday" => SkillId::CatchOfTheDay,
            "crushingblow" => SkillId::CrushingBlow,
            "drunkard" => SkillId::Drunkard,
            "furiousoutburst" => SkillId::FuriousOutburst,
            "furyofthebloodgod" => SkillId::FuryOfTheBloodGod,
            "goredbythebull" => SkillId::GoredByTheBull,
            "halflingluck" => SkillId::HalflingLuck,
            "illbeback" => SkillId::IllBeBack,
            "indomitable" => SkillId::Indomitable,
            "kaboom" => SkillId::Kaboom,
            "keenplayer" => SkillId::KeenPlayer,
            "kickemwhiletheyredown" => SkillId::KickEmWhileTheyReDown,
            "maximumcarnage" => SkillId::MaximumCarnage,
            "oldpro" => SkillId::OldPro,
            "plagueridden" => SkillId::PlagueRidden,
            "primalsavagery" => SkillId::PrimalSavagery,
            "quickbite" => SkillId::QuickBite,
            "raidingparty" => SkillId::RaidingParty,
            "ram" => SkillId::Ram,
            "reliable" => SkillId::Reliable,
            "savageblow" => SkillId::SavageBlow,
            "savagemauling" => SkillId::SavageMauling,
            "shottonothing" => SkillId::ShotToNothing,
            "sneakiestofthelot" => SkillId::SneakiestOfTheLot,
            "staroftheshow" => SkillId::StarOfTheShow,
            "strongpassinggame" => SkillId::StrongPassingGame,
            "swiftasthebreeze" => SkillId::SwiftAsTheBreeze,
            "tastymorsel" => SkillId::TastyMorsel,
            "theflashingblade" => SkillId::TheFlashingBlade,
            "thinkingmanstroll" => SkillId::ThinkingMansTroll,
            "treacherous" => SkillId::Treacherous,
            "trickster" => SkillId::Trickster,
            "unstoppablemomentum" => SkillId::UnstoppableMomentum,
            "viciousvines" => SkillId::ViciousVines,
            "watchout" => SkillId::WatchOut,

            // ── Human-readable aliases (space/hyphen separated) used in FUMBBL JSON data ──────
            _ => return None,
        };
        Some(skill)
    }

    /// Returns the NamedProperty string keys this skill grants.
    /// 1:1 translation of Skill.getSkillProperties() → SkillId lookup table.
    pub fn properties(self) -> &'static [&'static str] {
        match self {
            SkillId::SteadyFooting => &["canAvoidFallingDown"],
            SkillId::TakeRoot => &["becomesImmovable"],
            SkillId::Sprint => &["canMakeAnExtraGfi"],
            SkillId::SureFeet => &["canMakeAnExtraGfiOnce"],
            SkillId::Block => &["preventFallOnBothDown"],
            SkillId::Dodge => &["ignoreDefenderStumblesResult", "canRerollDodge"],
            SkillId::Fend => &["preventOpponentFollowingUp"],
            SkillId::StandFirm => &["canRefuseToBePushed"],
            SkillId::SideStep => &["canChooseOwnPushedBackSquare"],
            SkillId::Sidestep => &["canChooseOwnPushedBackSquare"],
            SkillId::Shadowing => &["canFollowPlayerLeavingTacklezones"],
            // Java bb2025/EyeGouge.postConstruct: registerProperty(canRemoveOpponentAssists)
            SkillId::EyeGouge => &["canRemoveOpponentAssists"],
            // Java Animosity.postConstruct (all editions): registerProperty(hasToRollToPassBallOn)
            SkillId::Animosity => &["hasToRollToPassBallOn"],
            SkillId::HypnoticGaze => &["inflictsConfusion", "canGazeDuringMove"],
            SkillId::Leap => &["canLeap"],
            SkillId::PogoStick => &[
                "canLeap",
                "ignoreTacklezonesWhenJumping",
                "failedRushForJumpAlwaysLandsInTargetSquare",
            ],
            SkillId::Pogo => &[
                "canLeap",
                "ignoreTacklezonesWhenJumping",
                "failedRushForJumpAlwaysLandsInTargetSquare",
            ],
            // Java mixed/Juggernaut: canConvertBothDownToPush + 3 CancelSkillProperty registrations.
            SkillId::Juggernaut => &[
                "canConvertBothDownToPush",
                "cancelsCanTakeDownPlayersWithHimOnBothDown",
                "cancelsCanRefuseToBePushed",
                "cancelsPreventOpponentFollowingUp",
            ],
            SkillId::Frenzy => &["forceFollowup", "forceSecondBlock"],
            // Java: Guard.postConstruct registers only assistsBlocksInTacklezones
            SkillId::Guard => &["assistsBlocksInTacklezones"],
            SkillId::DivingTackle => &["canAttemptToTackleDodgingPlayer"],
            SkillId::Tentacles => &["canHoldPlayersLeavingTacklezones"],
            SkillId::AlwaysHungry => &["mightEatPlayerToThrow"],
            SkillId::BoneHead => &["appliesConfusion"],
            SkillId::ReallyStupid => &["appliesConfusion", "needsToRollHighToAvoidConfusion"],
            SkillId::MightyBlow => &["affectsEitherArmourOrInjuryOnBlock"],
            SkillId::DirtyPlayer => &["affectsEitherArmourOrInjuryOnFoul"],
            SkillId::Stab => &[
                "canPerformArmourRollInsteadOfBlock",
                "providesBlockAlternative",
                "providesStabBlockAlternative",
            ],
            SkillId::Chainsaw => &[
                "makesStrengthTestObsolete",
                "blocksLikeChainsaw",
                "needsNoDiceDecorations",
                "providesChainsawBlockAlternative",
                "providesChainsawFoulingAlternative",
            ],
            SkillId::Claw => &["reducesArmourToFixedValue"],
            SkillId::ThickSkull => &["convertKOToStunOn8"],
            SkillId::AnimalSavagery => &[
                "enableStandUpAndEndBlitzAction",
                "needsToRollForActionBlockingIsEasier",
            ],
            // Java: HitAndRun.postConstruct registers only canMoveAfterBlock
            SkillId::HitAndRun => &["canMoveAfterBlock"],
            SkillId::QuickFoul => &["canMoveAfterFoul"],
            // Java: Pro.postConstruct registers only canRerollOncePerTurn (all editions)
            SkillId::Pro => &["canRerollOncePerTurn"],
            // Java: MultipleBlock.postConstruct registers canBlockMoreThanOnce (bb2016) + canBlockTwoAtOnce (mixed)
            SkillId::MultipleBlock => &["canBlockMoreThanOnce", "canBlockTwoAtOnce"],
            SkillId::Dauntless => &["canRollToMatchOpponentsStrength"],
            SkillId::DisturbingPresence => &["inflictsDisturbingPresence"],
            // Java: FoulAppearance.postConstruct registers forceRollBeforeBeingBlocked
            SkillId::FoulAppearance => &["forceRollBeforeBeingBlocked"],
            // Java: PrehensileTail.postConstruct registers makesDodgingHarder; mixed also makesJumpingHarder
            SkillId::PrehensileTail => &["makesDodgingHarder", "makesJumpingHarder"],
            // Java: Tackle.postConstruct registers 3 CancelSkillProperties
            SkillId::Tackle => &["cancelsCanRerollDodge", "cancelsIgnoreDefenderStumblesResult", "cancelsIgnoresDefenderStumblesResultForFirstBlock"],
            // Java: Wrestle.postConstruct registers canTakeDownPlayersWithHimOnBothDown
            SkillId::Wrestle => &["canTakeDownPlayersWithHimOnBothDown"],
            SkillId::Swoop => &["ttmScattersInSingleDirection"],
            SkillId::KickTeamMate => &["canKickTeamMates"],
            SkillId::ThrowTeamMate => &["canThrowTeamMates"],
            SkillId::RightStuff => &["canBeThrown", "canBeKicked"],
            // Java: BB2020 BallAndChain registers CancelSkillProperty(inflictsConfusion) and CancelSkillProperty(canMoveBeforeBeingBlocked)
            SkillId::BallAndChain => &["movesRandomly", "blocksLikeChainsaw", "cancelsInflictsConfusion", "cancelsCanMoveBeforeBeingBlocked"],
            // Java: BreatheFire.postConstruct registers canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover
            SkillId::BreatheFire => &["canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover"],
            // Java: WildAnimal.postConstruct registers enableStandUpAndEndBlitzAction + needsToRollForActionButKeepsTacklezone
            SkillId::WildAnimal => &["enableStandUpAndEndBlitzAction", "needsToRollForActionButKeepsTacklezone"],
            // Java: Loner.postConstruct registers hasToRollToUseTeamReroll + preventCardRabbitsFoot
            SkillId::Loner => &["hasToRollToUseTeamReroll", "preventCardRabbitsFoot"],
            // Java: Decay.postConstruct registers cancelsAllowsRaisingLineman + requiresSecondCasualtyRoll
            //   (mixed/Decay only has cancelsAllowsRaisingLineman, bb2016 also has requiresSecondCasualtyRoll)
            SkillId::Decay => &["cancelsAllowsRaisingLineman", "requiresSecondCasualtyRoll"],
            // Java: Regeneration.postConstruct registers preventRaiseFromDead + canRollToSaveFromInjury + cancelsAllowsRaisingLineman
            //   (BB2025 Regeneration does NOT have preventRaiseFromDead, but we include it for the union)
            SkillId::Regeneration => &["preventRaiseFromDead", "canRollToSaveFromInjury", "cancelsAllowsRaisingLineman"],
            SkillId::GiveAndGo => &["canMoveAfterQuickPass", "canMoveAfterHandOff"],
            SkillId::RunningPass => &["canMoveAfterQuickPass"],
            SkillId::DivingCatch => &["canAttemptCatchInAdjacentSquares"],
            SkillId::NoHands => &["preventCatch"],
            // Java: SafeThrow.postConstruct registers NamedProperties.canCancelInterceptions
            SkillId::SafeThrow => &["canCancelInterceptions"],
            // Java: VeryLongLegs.postConstruct registers CancelSkillProperty(canCancelInterceptions) (BB2016)
            //   and CancelSkillProperty(canForceInterceptionRerollOfLongPasses) (BB2020) — union of both.
            SkillId::VeryLongLegs => &["cancelsCancelInterceptions", "cancelsCanForceInterceptionRerollOfLongPasses"],
            // Java: CloudBurster (BB2020) registers NamedProperties.canForceInterceptionRerollOfLongPasses
            SkillId::CloudBurster => &["canForceInterceptionRerollOfLongPasses"],
            // Java: FuriousOutburst.postConstruct registers canTeleportBeforeAndAfterAvRollAttack
            SkillId::FuriousOutburst => &["canTeleportBeforeAndAfterAvRollAttack"],
            // Java: SafePass.postConstruct registers NamedProperties.dontDropFumbles
            SkillId::SafePass => &["dontDropFumbles"],
            // Java: Trickster.postConstruct registers NamedProperties.canMoveBeforeBeingBlocked
            SkillId::Trickster => &["canMoveBeforeBeingBlocked"],
            // Java: BlastIt.postConstruct registers NamedProperties.canReRollHmpScatter
            SkillId::BlastIt => &["canReRollHmpScatter"],
            // Java: PutridRegurgitation.postConstruct registers canUseVomitAfterBlock, providesBlockAlternative, canPerformArmourRollInsteadOfBlockThatMightFail
            SkillId::PutridRegurgitation => &[
                "canUseVomitAfterBlock",
                "providesBlockAlternative",
                "canPerformArmourRollInsteadOfBlockThatMightFail",
            ],
            // Java: LethalFlight.postConstruct registers affectsEitherArmourOrInjuryOnTtm + grantsSppWhenHittingOpponentOnTtm
            SkillId::LethalFlight => &["affectsEitherArmourOrInjuryOnTtm", "grantsSppWhenHittingOpponentOnTtm"],
            // Java: ViolentInnovator.postConstruct registers grantsSppFromSpecialActionsCas
            SkillId::ViolentInnovator => &["grantsSppFromSpecialActionsCas"],
            // Java: MaximumCarnage.postConstruct registers canPerformSecondChainsawAttack
            SkillId::MaximumCarnage => &["canPerformSecondChainsawAttack"],
            // Java: BeerBarrelBash.postConstruct registers canThrowKeg
            SkillId::BeerBarrelBash => &["canThrowKeg"],
            // Java: PilingOnBehaviour registers canPileOnOpponent
            SkillId::PilingOn => &["canPileOnOpponent"],
            // Java: bb2016/WeepingDagger.postConstruct registers appliesPoisonOnBadlyHurt
            SkillId::WeepingDagger => &["appliesPoisonOnBadlyHurt"],
            // Java: bb2025/PumpUpTheCrowd.postConstruct registers grantsTeamReRollWhenCausingBlockCas
            // Java: bb2020/PumpUpTheCrowd.postConstruct registers grantsTeamReRollWhenCausingCas
            SkillId::PumpUpTheCrowd => &["grantsTeamReRollWhenCausingBlockCas", "grantsTeamReRollWhenCausingCas"],
            // Java: bb2016+bb2020/SneakyGit.postConstruct + bb2025/PutTheBootIn.postConstruct register canAlwaysAssistFouls
            SkillId::SneakyGit | SkillId::PutTheBootIn => &["canAlwaysAssistFouls"],
            // Java: bb2025/Defensive.postConstruct registers CancelSkillProperty(canAlwaysAssistFouls)
            SkillId::Defensive => &["cancelsCanAlwaysAssistFouls"],
            // Java: bb2020/PileDriver.postConstruct + bb2025/PileDriver.postConstruct register canFoulAfterBlock
            SkillId::PileDriver => &["canFoulAfterBlock"],
            // Java: mixed/SecretWeapon.postConstruct registers getsSentOffAtEndOfDrive
            SkillId::SecretWeapon => &["getsSentOffAtEndOfDrive"],
            // Java: mixed/IronHardSkin.postConstruct registers cancelsReducesArmourToFixedValue + ignores properties
            SkillId::IronHardSkin => &[
                "cancelsReducesArmourToFixedValue",
                "ignoresArmourModifiersFromFouls",
                "ignoresArmourModifiersFromSkills",
                "ignoresArmourModifiersFromSpecialEffects",
            ],
            // Java: bb2020+bb2025/LordOfChaos.postConstruct registers grantsSingleUseTeamRerollWhenOnPitch
            SkillId::LordOfChaos => &["grantsSingleUseTeamRerollWhenOnPitch"],
            // Java: NurglesRot.postConstruct registers allowsRaisingLineman
            SkillId::NurglesRot => &["allowsRaisingLineman"],
            // Java: Stunty.postConstruct registers smallIcon, preventRaiseFromDead, cancelsAllowsRaisingLineman,
            //   ignoreTacklezonesWhenDodging, isHurtMoreEasily; mixed/Stunty also passesAreInterceptedEasier
            SkillId::Stunty => &[
                "smallIcon",
                "preventRaiseFromDead",
                "cancelsAllowsRaisingLineman",
                "ignoreTacklezonesWhenDodging",
                "isHurtMoreEasily",
                "passesAreInterceptedEasier",
            ],
            // Java: StrongPassingGame.postConstruct registers canAddStrengthToPass
            SkillId::StrongPassingGame => &["canAddStrengthToPass"],
            // Java: Yoink.postConstruct registers canInterceptEasily
            SkillId::Yoink => &["canInterceptEasily"],
            // Java: PassBlock.postConstruct registers canMoveWhenOpponentPasses
            SkillId::PassBlock => &["canMoveWhenOpponentPasses"],
            // Java: mixed/OnTheBall.postConstruct registers canMoveDuringKickOffScatter + canMoveWhenOpponentPasses
            SkillId::OnTheBall => &["canMoveDuringKickOffScatter", "canMoveWhenOpponentPasses"],
            // Java: Kick.postConstruct registers canReduceKickDistance
            SkillId::Kick => &["canReduceKickDistance"],
            // Java: Kaboom.postConstruct registers canForceBombExplosion
            SkillId::Kaboom => &["canForceBombExplosion"],
            // Java: Bombardier.postConstruct registers enableThrowBombAction (all editions)
            //   BB2020 additionally: preventStuntyDodgeModifier, cancelsIgnoreTacklezonesWhenDodging
            SkillId::Bombardier => &[
                "enableThrowBombAction",
                "preventStuntyDodgeModifier",
                "cancelsIgnoreTacklezonesWhenDodging",
            ],
            // Java: FrenziedRush.postConstruct registers canGainFrenzyForBlitz
            SkillId::FrenziedRush => &["canGainFrenzyForBlitz"],
            // Java: SlashingNails.postConstruct registers canGainClawsForBlitz
            SkillId::SlashingNails => &["canGainClawsForBlitz"],
            // Java: Incorporeal.postConstruct registers canAvoidDodging
            SkillId::Incorporeal => &["canAvoidDodging"],
            // Java: HailMaryPass.postConstruct registers canGainHailMary
            SkillId::HailMaryPass => &["canGainHailMary"],
            // Java: SafePairOfHands.postConstruct registers canPlaceBallWhenKnockedDownOrPlacedProne
            SkillId::SafePairOfHands => &["canPlaceBallWhenKnockedDownOrPlacedProne"],
            // Java: SaboteurBehaviour registers canSabotageBlockerOnKnockdown
            SkillId::Saboteur => &["canSabotageBlockerOnKnockdown"],
            // Java: mixed/special/QuickBite.postConstruct registers canAttackOpponentForBallAfterCatch
            SkillId::QuickBite => &["canAttackOpponentForBallAfterCatch"],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_name_round_trip() {
        let cases = [SkillId::Block, SkillId::Dodge, SkillId::MightyBlow, SkillId::Wrestle];
        for id in cases {
            let name = id.class_name();
            assert_eq!(SkillId::from_class_name(name), Some(id), "round-trip failed for {name}");
        }
    }

    #[test]
    fn unknown_class_name_returns_none() {
        assert_eq!(SkillId::from_class_name("NonExistentSkill"), None);
    }

    #[test]
    fn properties_steady_footing() {
        assert_eq!(SkillId::SteadyFooting.properties(), &["canAvoidFallingDown"]);
    }

    #[test]
    fn properties_dodge_has_two() {
        let props = SkillId::Dodge.properties();
        assert!(props.contains(&"ignoreDefenderStumblesResult"));
        assert!(props.contains(&"canRerollDodge"));
    }

    #[test]
    fn properties_block() {
        assert!(SkillId::Block.properties().contains(&"preventFallOnBothDown"));
    }

    #[test]
    fn properties_leap() {
        assert!(SkillId::Leap.properties().contains(&"canLeap"));
    }

    #[test]
    fn properties_pogo_includes_leap() {
        assert!(SkillId::PogoStick.properties().contains(&"canLeap"));
    }

    #[test]
    fn properties_kick_has_can_reduce_kick_distance() {
        assert!(SkillId::Kick.properties().contains(&"canReduceKickDistance"));
    }

    #[test]
    fn properties_chainsaw_has_five() {
        assert_eq!(SkillId::Chainsaw.properties().len(), 5);
    }

    #[test]
    fn properties_trickster_has_can_move_before_being_blocked() {
        assert!(SkillId::Trickster.properties().contains(&"canMoveBeforeBeingBlocked"));
    }

    #[test]
    fn properties_safe_pass_has_dont_drop_fumbles() {
        assert!(SkillId::SafePass.properties().contains(&"dontDropFumbles"));
    }

    #[test]
    fn properties_ball_and_chain_cancels_trickster() {
        assert!(SkillId::BallAndChain.properties().contains(&"cancelsCanMoveBeforeBeingBlocked"));
    }

    #[test]
    fn properties_eye_gouge_can_remove_opponent_assists() {
        assert!(SkillId::EyeGouge.properties().contains(&"canRemoveOpponentAssists"));
    }

    #[test]
    fn properties_animosity_has_to_roll_to_pass_ball_on() {
        assert!(SkillId::Animosity.properties().contains(&"hasToRollToPassBallOn"));
    }

    #[test]
    fn properties_blast_it_has_can_reroll_hmp_scatter() {
        assert!(SkillId::BlastIt.properties().contains(&"canReRollHmpScatter"));
    }

    #[test]
    fn properties_putrid_regurgitation_has_three() {
        let props = SkillId::PutridRegurgitation.properties();
        assert!(props.contains(&"canUseVomitAfterBlock"));
        assert!(props.contains(&"providesBlockAlternative"));
        assert!(props.contains(&"canPerformArmourRollInsteadOfBlockThatMightFail"));
    }

    #[test]
    fn properties_lethal_flight_ttm_spp() {
        let props = SkillId::LethalFlight.properties();
        assert!(props.contains(&"affectsEitherArmourOrInjuryOnTtm"));
        assert!(props.contains(&"grantsSppWhenHittingOpponentOnTtm"));
    }

    #[test]
    fn properties_violent_innovator_grants_spp() {
        assert!(SkillId::ViolentInnovator.properties().contains(&"grantsSppFromSpecialActionsCas"));
    }

    #[test]
    fn properties_maximum_carnage_second_chainsaw() {
        assert!(SkillId::MaximumCarnage.properties().contains(&"canPerformSecondChainsawAttack"));
    }
}
