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
            // OncePerHalf (mixed): CatchOfTheDay, FuriousOutburst, ThinkingMansTroll,
            // ViciousVines, WatchOut
            SkillId::CatchOfTheDay | SkillId::FuriousOutburst | SkillId::ThinkingMansTroll |
            SkillId::ViciousVines | SkillId::WatchOut => OncePerHalf,
            // FrenziedRush/PutridRegurgitation: bb2020=OncePerGame, bb2025=OncePerHalf — use bb2025
            SkillId::FrenziedRush | SkillId::PutridRegurgitation => OncePerHalf,

            // OncePerTurn (bb2025): Dodge, Pro, LoneFouler, SureFeet
            SkillId::Dodge | SkillId::Pro | SkillId::LoneFouler | SkillId::SureFeet => OncePerTurn,
            // OncePerTurnByTeamMate: Swoop (bb2025). WisdomOfTheWhiteDwarf is
            // OncePerTurnByTeamMate in bb2020 but OncePerGame in bb2025 — latest edition wins.
            SkillId::Swoop => OncePerTurnByTeamMate,
            SkillId::WisdomOfTheWhiteDwarf => OncePerGame,

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
            SkillId::QuickBite | SkillId::Ram | SkillId::SavageBlow |
            SkillId::SavageMauling | SkillId::ShotToNothing | SkillId::StarOfTheShow |
            SkillId::StrongPassingGame | SkillId::SwiftAsTheBreeze | SkillId::TastyMorsel |
            SkillId::TheFlashingBlade | SkillId::Treacherous | SkillId::Yoink => OncePerGame,

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
    /// Java registers a skill's `NamedProperties` in the PER-EDITION skill class'
    /// `postConstruct`, so the same skill can carry different properties in different rulesets.
    /// `properties()` below is the edition-agnostic UNION, which is right for every skill whose
    /// three Java classes register the same set — but wrong where they diverge.
    ///
    /// `RightStuff` is the first known divergence:
    ///
    /// | Java class | registered properties |
    /// |---|---|
    /// | `skill/bb2016/RightStuff` | `canBeThrown`, `canBeKicked`, `ignoreTackleWhenBlocked` |
    /// | `skill/bb2020/RightStuff` | `canBeThrownIfStrengthIs3orLess`, `ignoreTackleWhenBlocked` |
    /// | `skill/bb2025/RightStuff` | `canBeThrown`, `ignoreTackleWhenBlocked` |
    ///
    /// The union makes a BB2020 Right Stuff player answer `true` to `canBeThrown`, so Rust offered a
    /// Throw Team-Mate target where Java has none at all (chaos_pact bb2020 seed 22: Java's harness
    /// found `nTargets=0` and deselected, Rust threw and injured the thrown player).
    ///
    /// The arms are GENERATED from `skill/{bb2016,bb2020,bb2025,mixed,common}/*.java` by parsing
    /// `registerProperty(NamedProperties.X)`, resolving each edition to its own class if one exists
    /// and falling back to `mixed/` then `common/` otherwise — which is what Java's per-edition skill
    /// factory does. BOTH registration shapes are captured — `registerProperty(NamedProperties.X)`
    /// and `registerProperty(new CancelSkillProperty(NamedProperties.X))`, the latter becoming
    /// `cancelsX`. Scanning only the three edition directories (the first attempt) MISSED every
    /// skill that has no per-edition class, e.g. `Stunty`, `Decay`, `Juggernaut`, `PrehensileTail`,
    /// `SecretWeapon`, `MultipleBlock` and `DivingTackle`.
    ///
    /// Everything not listed falls through to the union in `properties()`.
    /// Java `Skill(String name, SkillCategory category)` - the constructor argument pair every
    /// skill class passes to `super(...)`, resolved per edition the same way the skill classes
    /// themselves are (`skill/<edition>/` first, then `mixed/`, then `common/`).
    ///
    /// Generated from the Java sources. 28 of the 197 skills declare a different category or a
    /// different display name per edition - e.g. Bone-Head is EXTRAORDINARY/"Bone-Head" in BB2016
    /// but TRAIT/"Bone Head" in BB2020+, and Dirty Player moved GENERAL -> DEVIOUS - so this is
    /// edition-aware for the same reason `properties_for` is.
    ///
    /// Needed by the Intensive Training prayer, which offers the coach every skill in the player's
    /// position categories, sorted by NAME (`Comparator.comparing(Skill::getName)`).
    pub fn category_and_name_for(self, rules: crate::enums::Rules) -> (crate::model::skill_category::SkillCategory, &'static str) {
        use crate::enums::Rules;
        use crate::model::skill_category::SkillCategory;
        match self {
            SkillId::ASneakyPair => (SkillCategory::Trait, "A Sneaky Pair"),
            SkillId::Accurate => (SkillCategory::Passing, "Accurate"),
            SkillId::AgilityIncrease => (SkillCategory::StatIncrease, "+AG"),
            SkillId::AllYouCanEat => (SkillCategory::Trait, "All You Can Eat"),
            SkillId::AlwaysHungry => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Always Hungry"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Always Hungry"),
            },
            SkillId::AnimalSavagery => (SkillCategory::Trait, "Animal Savagery"),
            SkillId::Animosity => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Animosity"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Animosity"),
            },
            SkillId::ArmBar => (SkillCategory::Strength, "Arm Bar"),
            SkillId::ArmourIncrease => (SkillCategory::StatIncrease, "+AV"),
            SkillId::BalefulHex => (SkillCategory::Trait, "Baleful Hex"),
            SkillId::BallAndChain => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Ball and Chain"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Ball and Chain"),
            },
            SkillId::BeerBarrelBash => (SkillCategory::Trait, "Beer Barrel Bash!"),
            SkillId::BigHand => (SkillCategory::Mutation, "Big Hand"),
            SkillId::BlackInk => (SkillCategory::Trait, "Black Ink"),
            SkillId::BlastIt => (SkillCategory::Trait, "Blast It!"),
            SkillId::BlindRage => (SkillCategory::Trait, "Blind Rage"),
            SkillId::Block => (SkillCategory::General, "Block"),
            SkillId::BloodLust => (SkillCategory::Extraordinary, "Blood Lust"),
            SkillId::Bombardier => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Bombardier"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Bombardier"),
            },
            SkillId::BoneHead => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Bone-Head"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Bone Head"),
            },
            SkillId::BoundingLeap => (SkillCategory::Trait, "Bounding Leap"),
            SkillId::Brawler => (SkillCategory::Strength, "Brawler"),
            SkillId::BreakTackle => (SkillCategory::Strength, "Break Tackle"),
            SkillId::BreatheFire => (SkillCategory::Trait, "Breathe Fire"),
            SkillId::BrutalBlock => (SkillCategory::Trait, "Brutal Block"),
            SkillId::BugmansXXXXXX => (SkillCategory::Trait, "Bugman's XXXXXX"),
            SkillId::Bullseye => (SkillCategory::Strength, "Bullseye"),
            SkillId::BurstOfSpeed => (SkillCategory::Trait, "Burst of Speed"),
            SkillId::Cannoneer => (SkillCategory::Passing, "Cannoneer"),
            SkillId::Catch => (SkillCategory::Agility, "Catch"),
            SkillId::CatchOfTheDay => (SkillCategory::Trait, "Catch of the Day"),
            SkillId::Chainsaw => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Chainsaw"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Chainsaw"),
            },
            SkillId::Claw => (SkillCategory::Mutation, "Claw"),
            SkillId::CloudBurster => (SkillCategory::Passing, "Cloud Burster"),
            SkillId::ConsummateProfessional => (SkillCategory::Trait, "Consummate Professional"),
            SkillId::CrushingBlow => (SkillCategory::Trait, "Crushing Blow"),
            SkillId::Dauntless => (SkillCategory::General, "Dauntless"),
            SkillId::Decay => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Decay"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Decay"),
            },
            SkillId::Defensive => (SkillCategory::Agility, "Defensive"),
            SkillId::DirtyPlayer => match rules {
                Rules::Bb2016 | Rules::Bb2020 | Rules::Common => (SkillCategory::General, "Dirty Player"),
                Rules::Bb2025 => (SkillCategory::Devious, "Dirty Player"),
            },
            SkillId::Disposable => (SkillCategory::Extraordinary, "Disposable"),
            SkillId::DisturbingPresence => (SkillCategory::Mutation, "Disturbing Presence"),
            SkillId::DivingCatch => (SkillCategory::Agility, "Diving Catch"),
            SkillId::DivingTackle => (SkillCategory::Agility, "Diving Tackle"),
            SkillId::Dodge => (SkillCategory::Agility, "Dodge"),
            SkillId::Drunkard => (SkillCategory::Trait, "Drunkard"),
            SkillId::DumpOff => (SkillCategory::Passing, "Dump-Off"),
            SkillId::DwarfenScourge => (SkillCategory::Trait, "Dwarfen Scourge"),
            SkillId::DwarvenScourge => (SkillCategory::Trait, "Dwarven Scourge"),
            SkillId::ExtraArms => (SkillCategory::Mutation, "Extra Arms"),
            SkillId::EyeGouge => (SkillCategory::Devious, "Eye Gouge"),
            SkillId::FanFavourite => (SkillCategory::Extraordinary, "Fan Favourite"),
            SkillId::Fend => (SkillCategory::General, "Fend"),
            SkillId::FoulAppearance => (SkillCategory::Mutation, "Foul Appearance"),
            SkillId::FrenziedRush => (SkillCategory::Trait, "Frenzied Rush"),
            SkillId::Frenzy => (SkillCategory::General, "Frenzy"),
            SkillId::Fumblerooski => (SkillCategory::Devious, "Fumblerooski"),
            SkillId::Fumblerooskie => (SkillCategory::Passing, "Fumblerooskie"),
            SkillId::FuriousOutburst => (SkillCategory::Trait, "Furious Outburst"),
            SkillId::FuryOfTheBloodGod => (SkillCategory::Trait, "Fury of the Blood God"),
            SkillId::GhostlyFlames => (SkillCategory::Trait, "Ghostly Flames"),
            SkillId::GiveAndGo => (SkillCategory::Passing, "Give and Go"),
            SkillId::GoredByTheBull => (SkillCategory::Trait, "Gored By The Bull"),
            SkillId::Grab => (SkillCategory::Strength, "Grab"),
            SkillId::Guard => (SkillCategory::Strength, "Guard"),
            SkillId::HailMaryPass => (SkillCategory::Passing, "Hail Mary Pass"),
            SkillId::HalflingLuck => (SkillCategory::Trait, "Halfling Luck"),
            SkillId::Hatred => (SkillCategory::Trait, "Hatred"),
            SkillId::HitAndRun => (SkillCategory::Trait, "Hit And Run"),
            SkillId::Horns => (SkillCategory::Mutation, "Horns"),
            SkillId::HypnoticGaze => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Hypnotic Gaze"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Hypnotic Gaze"),
            },
            SkillId::IllBeBack => (SkillCategory::Trait, "I'll be back!"),
            SkillId::Incorporeal => (SkillCategory::Trait, "Incorporeal"),
            SkillId::Indomitable => (SkillCategory::Trait, "Indomitable"),
            SkillId::Insignificant => (SkillCategory::Trait, "Insignificant"),
            SkillId::IronHardSkin => (SkillCategory::Mutation, "Iron Hard Skin"),
            SkillId::Juggernaut => (SkillCategory::Strength, "Juggernaut"),
            SkillId::JumpUp => (SkillCategory::Agility, "Jump Up"),
            SkillId::Kaboom => (SkillCategory::Trait, "Kaboom!"),
            SkillId::KeenPlayer => (SkillCategory::Trait, "Keen Player"),
            SkillId::Kick => (SkillCategory::General, "Kick"),
            SkillId::KickEmWhileTheyReDown => (SkillCategory::Trait, "Kick 'em while they're down!"),
            SkillId::KickOffReturn => (SkillCategory::General, "Kick-Off Return"),
            SkillId::KickTeamMate => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Kick Team-Mate"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Kick Team-Mate"),
            },
            SkillId::KrumpAndSmash => (SkillCategory::Trait, "Krump and Smash"),
            SkillId::Leader => (SkillCategory::Passing, "Leader"),
            SkillId::Leap => (SkillCategory::Agility, "Leap"),
            SkillId::LethalFlight => (SkillCategory::Devious, "Lethal Flight"),
            SkillId::LoneFouler => (SkillCategory::Devious, "Lone Fouler"),
            SkillId::Loner => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Loner"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Loner"),
            },
            SkillId::LookIntoMyEyes => (SkillCategory::Trait, "Look Into My Eyes"),
            SkillId::LordOfChaos => (SkillCategory::Trait, "Lord of Chaos"),
            SkillId::MasterAssassin => (SkillCategory::Trait, "Master Assassin"),
            SkillId::MaximumCarnage => (SkillCategory::Trait, "Maximum Carnage"),
            SkillId::MesmerisingDance => (SkillCategory::Trait, "Mesmerising Dance"),
            SkillId::MesmerizingDance => (SkillCategory::Trait, "Mesmerizing Dance"),
            SkillId::MightyBlow => (SkillCategory::Strength, "Mighty Blow"),
            SkillId::MonstrousMouth => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Monstrous Mouth"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Mutation, "Monstrous Mouth"),
            },
            SkillId::MovementIncrease => (SkillCategory::StatIncrease, "+MA"),
            SkillId::MultipleBlock => (SkillCategory::Strength, "Multiple Block"),
            SkillId::MyBall => (SkillCategory::Trait, "My Ball"),
            SkillId::NervesOfSteel => (SkillCategory::Passing, "Nerves of Steel"),
            SkillId::NoBall => (SkillCategory::Trait, "No Ball"),
            SkillId::NoHands => match rules {
                Rules::Bb2016 | Rules::Bb2025 | Rules::Common => (SkillCategory::Extraordinary, "No Hands"),
                Rules::Bb2020 => (SkillCategory::Trait, "No Hands"),
            },
            SkillId::NurglesRot => (SkillCategory::Extraordinary, "Nurgle's Rot"),
            SkillId::OldPro => (SkillCategory::Trait, "Old Pro"),
            SkillId::OnTheBall => (SkillCategory::Passing, "On The Ball"),
            SkillId::Pass => (SkillCategory::Passing, "Pass"),
            SkillId::PassBlock => (SkillCategory::General, "Pass Block"),
            SkillId::PassingIncrease => (SkillCategory::StatIncrease, "+PA"),
            SkillId::PickMeUp => (SkillCategory::Trait, "Pick-me-up"),
            SkillId::PileDriver => match rules {
                Rules::Bb2016 | Rules::Bb2020 | Rules::Common => (SkillCategory::Strength, "Pile Driver"),
                Rules::Bb2025 => (SkillCategory::Devious, "Pile Driver"),
            },
            SkillId::PilingOn => (SkillCategory::Strength, "Piling On"),
            SkillId::PlagueRidden => (SkillCategory::Trait, "Plague Ridden"),
            SkillId::Pogo => (SkillCategory::Trait, "Pogo"),
            SkillId::PogoStick => (SkillCategory::Trait, "Pogo Stick"),
            SkillId::PrehensileTail => (SkillCategory::Mutation, "Prehensile Tail"),
            SkillId::PrimalSavagery => (SkillCategory::Trait, "Primal Savagery"),
            SkillId::Pro => (SkillCategory::General, "Pro"),
            SkillId::ProjectileVomit => (SkillCategory::Trait, "Projectile Vomit"),
            SkillId::PumpUpTheCrowd => (SkillCategory::Trait, "Pump Up The Crowd"),
            SkillId::Punt => (SkillCategory::Passing, "Punt"),
            SkillId::PutTheBootIn => (SkillCategory::Devious, "Put the Boot In"),
            SkillId::PutridRegurgitation => (SkillCategory::Trait, "Putrid Regurgitation"),
            SkillId::QuickBite => (SkillCategory::Trait, "Quick Bite"),
            SkillId::QuickFoul => (SkillCategory::Devious, "Quick Foul"),
            SkillId::RaidingParty => (SkillCategory::Trait, "Raiding Party"),
            SkillId::Ram => (SkillCategory::Trait, "Ram"),
            SkillId::ReallyStupid => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Really Stupid"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Really Stupid"),
            },
            SkillId::Regeneration => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Regeneration"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Regeneration"),
            },
            SkillId::Reliable => (SkillCategory::Trait, "Reliable"),
            SkillId::RightStuff => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Right Stuff"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Right Stuff"),
            },
            SkillId::RunningPass => (SkillCategory::Passing, "Running Pass"),
            SkillId::Saboteur => (SkillCategory::Devious, "Saboteur"),
            SkillId::SafePairOfHands => (SkillCategory::Agility, "Safe Pair Of Hands"),
            SkillId::SafePass => (SkillCategory::Passing, "Safe Pass"),
            SkillId::SafeThrow => (SkillCategory::Passing, "Safe Throw"),
            SkillId::SavageBlow => (SkillCategory::Trait, "Savage Blow"),
            SkillId::SavageMauling => (SkillCategory::Trait, "Savage Mauling"),
            SkillId::SecretWeapon => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Secret Weapon"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Secret Weapon"),
            },
            SkillId::Shadowing => match rules {
                Rules::Bb2016 | Rules::Bb2020 | Rules::Common => (SkillCategory::General, "Shadowing"),
                Rules::Bb2025 => (SkillCategory::Devious, "Shadowing"),
            },
            SkillId::ShotToNothing => (SkillCategory::Trait, "Shot to Nothing"),
            SkillId::SideStep => (SkillCategory::Agility, "Side Step"),
            SkillId::Sidestep => (SkillCategory::Agility, "Sidestep"),
            SkillId::SlashingNails => (SkillCategory::Trait, "Slashing Nails"),
            SkillId::Slayer => (SkillCategory::Trait, "Slayer"),
            SkillId::SneakiestOfTheLot => (SkillCategory::Trait, "Sneakiest of the Lot"),
            SkillId::SneakyGit => match rules {
                Rules::Bb2016 | Rules::Bb2020 | Rules::Common => (SkillCategory::Agility, "Sneaky Git"),
                Rules::Bb2025 => (SkillCategory::Devious, "Sneaky Git"),
            },
            SkillId::Sprint => (SkillCategory::Agility, "Sprint"),
            SkillId::Stab => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Stab"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Stab"),
            },
            SkillId::Stakes => (SkillCategory::Extraordinary, "Stakes"),
            SkillId::StandFirm => (SkillCategory::Strength, "Stand Firm"),
            SkillId::StarOfTheShow => (SkillCategory::Trait, "Star of the Show"),
            SkillId::SteadyFooting => (SkillCategory::Trait, "Steady Footing"),
            SkillId::StrengthIncrease => (SkillCategory::StatIncrease, "+ST"),
            SkillId::StripBall => (SkillCategory::General, "Strip Ball"),
            SkillId::StrongArm => (SkillCategory::Strength, "Strong Arm"),
            SkillId::StrongPassingGame => (SkillCategory::Trait, "Strong Passing Game"),
            SkillId::Stunty => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Stunty"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Stunty"),
            },
            SkillId::SureFeet => (SkillCategory::Agility, "Sure Feet"),
            SkillId::SureHands => (SkillCategory::General, "Sure Hands"),
            SkillId::Swarming => match rules {
                Rules::Bb2016 | Rules::Bb2025 | Rules::Common => (SkillCategory::Extraordinary, "Swarming"),
                Rules::Bb2020 => (SkillCategory::Trait, "Swarming"),
            },
            SkillId::SwiftAsTheBreeze => (SkillCategory::Trait, "Swift As The Breeze"),
            SkillId::Swoop => match rules {
                Rules::Bb2016 | Rules::Common => (SkillCategory::Extraordinary, "Swoop"),
                Rules::Bb2020 | Rules::Bb2025 => (SkillCategory::Trait, "Swoop"),
            },
            SkillId::Tackle => (SkillCategory::General, "Tackle"),
            SkillId::TakeRoot => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Take Root"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Take Root"),
            },
            SkillId::TastyMorsel => (SkillCategory::Trait, "Tasty Morsel"),
            SkillId::Taunt => (SkillCategory::General, "Taunt"),
            SkillId::TeamCaptain => (SkillCategory::Trait, "Team Captain"),
            SkillId::Tentacles => (SkillCategory::Mutation, "Tentacles"),
            SkillId::TheBallista => (SkillCategory::Trait, "The Ballista"),
            SkillId::TheFlashingBlade => (SkillCategory::Trait, "The Flashing Blade"),
            SkillId::ThickSkull => (SkillCategory::Strength, "Thick Skull"),
            SkillId::ThinkingMansTroll => (SkillCategory::Trait, "Thinking Man's Troll"),
            SkillId::ThrowTeamMate => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Throw Team-Mate"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Throw Team-Mate"),
            },
            SkillId::Timmmber => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Timmm-ber!"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Timmm-ber!"),
            },
            SkillId::Titchy => match rules {
                Rules::Bb2016 => (SkillCategory::Extraordinary, "Titchy"),
                Rules::Bb2020 | Rules::Bb2025 | Rules::Common => (SkillCategory::Trait, "Titchy"),
            },
            SkillId::ToxinConnoisseur => (SkillCategory::Trait, "Toxin Connoisseur"),
            SkillId::Treacherous => (SkillCategory::Trait, "Treacherous"),
            SkillId::Trickster => (SkillCategory::Trait, "Trickster"),
            SkillId::TwoForOne => (SkillCategory::Trait, "Two for One"),
            SkillId::TwoHeads => (SkillCategory::Mutation, "Two Heads"),
            SkillId::UnchannelledFury => (SkillCategory::Trait, "Unchannelled Fury"),
            SkillId::Unsteady => (SkillCategory::Trait, "Unsteady"),
            SkillId::UnstoppableMomentum => (SkillCategory::Trait, "Unstoppable Momentum"),
            SkillId::VeryLongLegs => (SkillCategory::Mutation, "Very Long Legs"),
            SkillId::ViciousVines => (SkillCategory::Trait, "Vicious Vines"),
            SkillId::ViolentInnovator => (SkillCategory::Devious, "Violent Innovator"),
            SkillId::WatchOut => (SkillCategory::Trait, "Watch Out!"),
            SkillId::WeepingDagger => (SkillCategory::Extraordinary, "Weeping Dagger"),
            SkillId::WhirlingDervish => (SkillCategory::Trait, "Whirling Dervish"),
            SkillId::WildAnimal => (SkillCategory::Extraordinary, "Wild Animal"),
            SkillId::WisdomOfTheWhiteDwarf => (SkillCategory::Trait, "Wisdom of the White Dwarf"),
            SkillId::WoodlandFury => (SkillCategory::Trait, "Woodland Fury"),
            SkillId::WorkingInTandem => (SkillCategory::Trait, "Working in Tandem"),
            SkillId::Wrestle => (SkillCategory::General, "Wrestle"),
            SkillId::Yoink => (SkillCategory::Trait, "Yoink!"),
            // The handful of Rust-only ids with no Java skill class carry no category.
            _ => (SkillCategory::Trait, self.class_name()),
        }
    }

    pub fn properties_for(self, rules: crate::enums::Rules) -> &'static [&'static str] {
        use crate::enums::Rules;
        match (self, rules) {
            (SkillId::BallAndChain, Rules::Bb2016) => &["blocksDuringMove", "canBlockMoreThanOnce", "canBlockSameTeamPlayer", "cancelsCanBlockMoreThanOnce", "cancelsCanPileOnOpponent", "cancelsForceRollBeforeBeingBlocked", "cancelsInflictsConfusion", "convertStunToKO", "flipSameTeamOpponentToOtherTeam", "forceFollowup", "forceFullMovement", "goForItAfterBlock", "grabOutsideBlock", "ignoreTacklezonesWhenMoving", "movesRandomly", "placedProneCausesInjuryRoll", "preventAutoMove", "preventKickTeamMateAction", "preventRecoverFromConcusionAction", "preventRecoverFromGazeAction", "preventRegularBlitzAction", "preventRegularBlockAction", "preventRegularFoulAction", "preventRegularHandOverAction", "preventRegularPassAction", "preventStandUpAction", "preventThrowTeamMateAction"],
            (SkillId::BallAndChain, Rules::Bb2020) => &["blocksDuringMove", "canBlockMoreThanOnce", "canBlockSameTeamPlayer", "cancelsCanBlockMoreThanOnce", "cancelsCanMoveBeforeBeingBlocked", "cancelsCanPileOnOpponent", "cancelsForceRollBeforeBeingBlocked", "cancelsInflictsConfusion", "cancelsPreventOpponentFollowingUp", "convertStunToKO", "forceFollowup", "goForItAfterBlock", "ignoreBlockAssists", "ignoreTacklezonesWhenMoving", "movesRandomly", "placedProneCausesInjuryRoll", "preventAutoMove", "preventKickTeamMateAction", "preventPickup", "preventRecoverFromConcusionAction", "preventRecoverFromGazeAction", "preventRegularBlitzAction", "preventRegularBlockAction", "preventRegularFoulAction", "preventRegularHandOverAction", "preventRegularPassAction", "preventStandUpAction", "preventThrowTeamMateAction"],
            (SkillId::BallAndChain, Rules::Bb2025 | Rules::Common) => &["blocksDuringMove", "canBlockMoreThanOnce", "canBlockSameTeamPlayer", "cancelsCanBlockMoreThanOnce", "cancelsCanMoveBeforeBeingBlocked", "cancelsCanPileOnOpponent", "cancelsForceRollBeforeBeingBlocked", "cancelsInflictsConfusion", "cancelsPreventOpponentFollowingUp", "convertStunToKO", "forceFollowup", "goForItAfterBlock", "ignoreBlockAssists", "ignoreTacklezonesWhenMoving", "movesRandomly", "placedProneCausesInjuryRoll", "preventAutoMove", "preventKickTeamMateAction", "preventPickup", "preventRecoverFromConcusionAction", "preventRecoverFromGazeAction", "preventRegularBlitzAction", "preventRegularBlockAction", "preventRegularFoulAction", "preventRegularHandOverAction", "preventRegularPassAction", "preventSecureTheBallAction", "preventStandUpAction", "preventThrowTeamMateAction"],
            (SkillId::Bombardier, Rules::Bb2016) => &["enableThrowBombAction"],
            (SkillId::Bombardier, Rules::Bb2020) => &["cancelsIgnoreTacklezonesWhenDodging", "enableThrowBombAction", "preventStuntyDodgeModifier"],
            (SkillId::Bombardier, Rules::Bb2025 | Rules::Common) => &["enableThrowBombAction"],
            (SkillId::Chainsaw, Rules::Bb2016) => &["blocksLikeChainsaw", "makesStrengthTestObsolete", "needsNoDiceDecorations"],
            (SkillId::Chainsaw, Rules::Bb2020) => &["blocksLikeChainsaw", "cancelsIgnoreTacklezonesWhenDodging", "preventStuntyDodgeModifier", "providesBlockAlternative", "providesChainsawBlockAlternative", "providesChainsawFoulingAlternative", "providesFoulingAlternative"],
            (SkillId::Chainsaw, Rules::Bb2025 | Rules::Common) => &["blocksLikeChainsaw", "providesBlockAlternative", "providesChainsawBlockAlternative", "providesChainsawFoulingAlternative", "providesFoulingAlternative"],
            (SkillId::CloudBurster, Rules::Bb2020) => &["canForceInterceptionRerollOfLongPasses"],
            (SkillId::CloudBurster, Rules::Bb2025 | Rules::Common) => &["passesAreNotIntercepted"],
            (SkillId::Decay, Rules::Bb2016) => &["cancelsAllowsRaisingLineman", "requiresSecondCasualtyRoll"],
            (SkillId::Decay, Rules::Bb2020) => &["cancelsAllowsRaisingLineman"],
            (SkillId::Decay, Rules::Bb2025 | Rules::Common) => &["cancelsAllowsRaisingLineman"],
            (SkillId::Defensive, Rules::Bb2020) => &["cancelsAssistsBlocksInTacklezones", "cancelsAssistsFoulsInTacklezones"],
            (SkillId::Defensive, Rules::Bb2025 | Rules::Common) => &["cancelsAssistsBlocksInTacklezones", "cancelsAssistsFoulsInTacklezones", "cancelsCanAlwaysAssistFouls"],
            (SkillId::DivingTackle, Rules::Bb2016) => &["canAttemptToTackleDodgingPlayer"],
            (SkillId::DivingTackle, Rules::Bb2020) => &["canAttemptToTackleDodgingPlayer", "canAttemptToTackleJumpingPlayer"],
            (SkillId::DivingTackle, Rules::Bb2025 | Rules::Common) => &["canAttemptToTackleDodgingPlayer", "canAttemptToTackleJumpingPlayer"],
            (SkillId::HypnoticGaze, Rules::Bb2016) => &["canGazeDuringMove", "inflictsConfusion"],
            (SkillId::HypnoticGaze, Rules::Bb2020) => &["inflictsConfusion"],
            (SkillId::HypnoticGaze, Rules::Bb2025 | Rules::Common) => &["inflictsConfusion"],
            (SkillId::Juggernaut, Rules::Bb2016) => &["cancelsCanRefuseToBePushed", "cancelsCanTakeDownPlayersWithHimOnBothDown", "cancelsPreventOpponentFollowingUp"],
            (SkillId::Juggernaut, Rules::Bb2020) => &["cancelsCanRefuseToBePushed", "cancelsCanTakeDownPlayersWithHimOnBothDown", "cancelsPreventOpponentFollowingUp"],
            (SkillId::Juggernaut, Rules::Bb2025 | Rules::Common) => &["canConvertBothDownToPush", "cancelsCanRefuseToBePushed", "cancelsCanTakeDownPlayersWithHimOnBothDown", "cancelsPreventOpponentFollowingUp"],
            (SkillId::Leap, Rules::Bb2016) => &["canLeap"],
            (SkillId::Leap, Rules::Bb2020) => &["canLeap", "failedRushForJumpAlwaysLandsInTargetSquare"],
            (SkillId::Leap, Rules::Bb2025 | Rules::Common) => &["canLeap"],
            (SkillId::MonstrousMouth, Rules::Bb2016) => &["cancelsForceOpponentToDropBallOnPushback"],
            (SkillId::MonstrousMouth, Rules::Bb2020) => &["cancelsForceOpponentToDropBallOnPushback"],
            (SkillId::MonstrousMouth, Rules::Bb2025 | Rules::Common) => &["canPinPlayers", "cancelsForceOpponentToDropBallOnPushback", "providesBlockAlternative"],
            (SkillId::MultipleBlock, Rules::Bb2016) => &["canBlockMoreThanOnce"],
            (SkillId::MultipleBlock, Rules::Bb2020) => &["canBlockTwoAtOnce"],
            (SkillId::MultipleBlock, Rules::Bb2025 | Rules::Common) => &["canBlockTwoAtOnce"],
            (SkillId::PilingOn, Rules::Bb2016) => &["canPileOnOpponent"],
            (SkillId::PilingOn, Rules::Bb2020) => &[],
            (SkillId::PrehensileTail, Rules::Bb2016) => &["makesDodgingHarder"],
            (SkillId::PrehensileTail, Rules::Bb2020) => &["makesDodgingHarder", "makesJumpingHarder"],
            (SkillId::PrehensileTail, Rules::Bb2025 | Rules::Common) => &["makesDodgingHarder", "makesJumpingHarder"],
            (SkillId::Regeneration, Rules::Bb2016) => &["canRollToSaveFromInjury", "cancelsAllowsRaisingLineman", "preventRaiseFromDead"],
            (SkillId::Regeneration, Rules::Bb2020) => &["canRollToSaveFromInjury", "cancelsAllowsRaisingLineman", "preventRaiseFromDead"],
            (SkillId::Regeneration, Rules::Bb2025 | Rules::Common) => &["canRollToSaveFromInjury", "cancelsAllowsRaisingLineman"],
            (SkillId::RightStuff, Rules::Bb2016) => &["canBeKicked", "canBeThrown", "ignoreTackleWhenBlocked"],
            (SkillId::RightStuff, Rules::Bb2020) => &["canBeThrownIfStrengthIs3orLess", "ignoreTackleWhenBlocked"],
            (SkillId::RightStuff, Rules::Bb2025 | Rules::Common) => &["canBeThrown", "ignoreTackleWhenBlocked"],
            (SkillId::SecretWeapon, Rules::Bb2016) => &["cancelsIgnoreTacklezonesWhenDodging", "getsSentOffAtEndOfDrive", "preventStuntyDodgeModifier"],
            (SkillId::SecretWeapon, Rules::Bb2020) => &["getsSentOffAtEndOfDrive"],
            (SkillId::SecretWeapon, Rules::Bb2025 | Rules::Common) => &["getsSentOffAtEndOfDrive"],
            (SkillId::SideStep, Rules::Bb2016) => &["canChooseOwnPushedBackSquare", "cancelsCanPushBackToAnySquare"],
            (SkillId::SideStep, Rules::Bb2020) => &["canChooseOwnPushedBackSquare"],
            (SkillId::SneakyGit, Rules::Bb2016) => &["canAlwaysAssistFouls"],
            (SkillId::SneakyGit, Rules::Bb2020) => &["canAlwaysAssistFouls", "canMoveAfterFoul"],
            (SkillId::SneakyGit, Rules::Bb2025 | Rules::Common) => &[],
            (SkillId::Stab, Rules::Bb2016) => &["canPerformArmourRollInsteadOfBlock", "providesBlockAlternative", "providesStabBlockAlternative"],
            (SkillId::Stab, Rules::Bb2020) => &["canPerformArmourRollInsteadOfBlock", "providesBlockAlternative", "providesMultipleBlockAlternative", "providesStabBlockAlternative"],
            (SkillId::Stab, Rules::Bb2025 | Rules::Common) => &["canPerformArmourRollInsteadOfBlock", "providesBlockAlternative", "providesStabBlockAlternative"],
            (SkillId::Stunty, Rules::Bb2016) => &["cancelsAllowsRaisingLineman", "ignoreTacklezonesWhenDodging", "isHurtMoreEasily", "preventRaiseFromDead", "smallIcon"],
            (SkillId::Stunty, Rules::Bb2020) => &["cancelsAllowsRaisingLineman", "ignoreTacklezonesWhenDodging", "isHurtMoreEasily", "passesAreInterceptedEasier", "preventRaiseFromDead", "smallIcon"],
            (SkillId::Stunty, Rules::Bb2025 | Rules::Common) => &["cancelsAllowsRaisingLineman", "ignoreTacklezonesWhenDodging", "isHurtMoreEasily", "passesAreInterceptedEasier", "preventRaiseFromDead", "smallIcon"],
            (SkillId::Swoop, Rules::Bb2016) => &["cancelsIgnoreTacklezonesWhenDodging", "preventStuntyDodgeModifier", "ttmScattersInSingleDirection"],
            (SkillId::Swoop, Rules::Bb2020) => &["cancelsIgnoreTacklezonesWhenDodging", "preventStuntyDodgeModifier", "ttmScattersInSingleDirection"],
            (SkillId::Swoop, Rules::Bb2025 | Rules::Common) => &["ttmScattersInSingleDirection"],
            (SkillId::VeryLongLegs, Rules::Bb2016) => &["cancelsCanCancelInterceptions"],
            (SkillId::VeryLongLegs, Rules::Bb2020) => &["cancelsCanForceInterceptionRerollOfLongPasses"],
            (SkillId::VeryLongLegs, Rules::Bb2025 | Rules::Common) => &["cancelsPassesAreNotIntercepted"],
            _ => self.properties(),
        }
    }

    pub fn properties(self) -> &'static [&'static str] {
        match self {
            // Java mixed/special star skills — each registers exactly one property; the
            // CLIENT_USE_SKILL special-dispatch chain (StepInitSelecting) keys on these.
            SkillId::Treacherous => &["canStabTeamMateForBall"],
            SkillId::RaidingParty => &["canMoveOpenTeamMate"],
            SkillId::LookIntoMyEyes => &["canStealBallFromOpponent"],
            SkillId::BalefulHex => &["canMakeOpponentMissTurn"],
            SkillId::CatchOfTheDay => &["canGetBallOnGround"],
            SkillId::BlackInk => &["canGazeAutomatically"],
            // Java bb2020/special/ThenIStartedBlastin + bb2025/special/BlastinSolvesEverything
            SkillId::ThenIStartedBlastin => &["canBlastRemotePlayer"],
            SkillId::BlastinSolvesEverything => &["canBlastRemotePlayer"],
            // Java bb2025/Punt.postConstruct: registerProperty(canPunt)
            SkillId::Punt => &["canPunt"],
            SkillId::SteadyFooting => &["canAvoidFallingDown"],
            SkillId::TakeRoot => &["becomesImmovable"],
            SkillId::Sprint => &["canMakeAnExtraGfi"],
            SkillId::SureFeet => &["canMakeAnExtraGfiOnce"],
            SkillId::Block => &["preventFallOnBothDown"],
            SkillId::Dodge => &["ignoreDefenderStumblesResult", "canRerollDodge"],
            SkillId::Fend => &["preventOpponentFollowingUp"],
            // Java: Horns.postConstruct registers addStrengthOnBlitz
            SkillId::Horns => &["addStrengthOnBlitz"],
            SkillId::StandFirm => &["canRefuseToBePushed"],
            // Java: bb2016/SideStep.postConstruct registers CancelSkillProperty(canPushBackToAnySquare) + canChooseOwnPushedBackSquare
            SkillId::SideStep => &["cancelsCanPushBackToAnySquare", "canChooseOwnPushedBackSquare"],
            SkillId::Sidestep => &["cancelsCanPushBackToAnySquare", "canChooseOwnPushedBackSquare"],
            // Java: bb2016/Grab.postConstruct registers canPushBackToAnySquare + CancelSkillProperty(canChooseOwnPushedBackSquare)
            SkillId::Grab => &["canPushBackToAnySquare", "cancelsCanChooseOwnPushedBackSquare"],
            SkillId::Shadowing => &["canFollowPlayerLeavingTacklezones"],
            // Java bb2025/EyeGouge.postConstruct: registerProperty(canRemoveOpponentAssists)
            SkillId::EyeGouge => &["canRemoveOpponentAssists"],
            // Java Animosity.postConstruct (all editions): registerProperty(hasToRollToPassBallOn)
            SkillId::Animosity => &["hasToRollToPassBallOn"],
            SkillId::HypnoticGaze => &["inflictsConfusion", "canGazeDuringMove"],
            // Java: bb2020/Leap.postConstruct registers canLeap + failedRushForJumpAlwaysLandsInTargetSquare
            // (the latter was missing entirely from this entry).
            SkillId::Leap => &["canLeap", "failedRushForJumpAlwaysLandsInTargetSquare"],
            // Java: bb2020/PogoStick.postConstruct registers canLeap, ignoreTacklezonesWhenJumping,
            //   failedRushForJumpAlwaysLandsInTargetSquare, CancelSkillProperty(makesJumpingHarder),
            //   and CancelSkillProperty(canAttemptToTackleJumpingPlayer) — the last two cancel
            //   properties were missing from the union.
            SkillId::PogoStick => &[
                "canLeap",
                "ignoreTacklezonesWhenJumping",
                "failedRushForJumpAlwaysLandsInTargetSquare",
                "cancelsMakesJumpingHarder",
                "cancelsCanAttemptToTackleJumpingPlayer",
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
            SkillId::DivingTackle => &["canAttemptToTackleDodgingPlayer", "canAttemptToTackleJumpingPlayer"],
            SkillId::Tentacles => &["canHoldPlayersLeavingTacklezones"],
            SkillId::AlwaysHungry => &["mightEatPlayerToThrow"],
            SkillId::BoneHead => &["appliesConfusion"],
            SkillId::ReallyStupid => &["appliesConfusion", "needsToRollHighToAvoidConfusion"],
            SkillId::MightyBlow => &["affectsEitherArmourOrInjuryOnBlock"],
            // Java: bb2020/Brawler.postConstruct registers canRerollSingleBothDown. Was previously
            // entirely absent from this table, falling through to the empty default.
            SkillId::Brawler => &["canRerollSingleBothDown"],
            SkillId::DirtyPlayer => &["affectsEitherArmourOrInjuryOnFoul"],
            // Java: bb2020/Stab.postConstruct registers 4 properties, including providesMultipleBlockAlternative
            SkillId::Stab => &[
                "canPerformArmourRollInsteadOfBlock",
                "providesBlockAlternative",
                "providesMultipleBlockAlternative",
                "providesStabBlockAlternative",
            ],
            // Java: Chainsaw.postConstruct (union across bb2016/bb2020/bb2025). bb2016-only:
            // makesStrengthTestObsolete, needsNoDiceDecorations. bb2020+bb2025 additionally register
            // providesBlockAlternative + providesFoulingAlternative; bb2020-only additionally adds
            // preventStuntyDodgeModifier + cancelsIgnoreTacklezonesWhenDodging.
            SkillId::Chainsaw => &[
                "makesStrengthTestObsolete",
                "blocksLikeChainsaw",
                "needsNoDiceDecorations",
                "providesBlockAlternative",
                "providesChainsawBlockAlternative",
                "providesChainsawFoulingAlternative",
                "providesFoulingAlternative",
                "preventStuntyDodgeModifier",
                "cancelsIgnoreTacklezonesWhenDodging",
            ],
            SkillId::Claw => &["reducesArmourToFixedValue"],
            SkillId::ThickSkull => &["convertKOToStunOn8"],
            SkillId::AnimalSavagery => &[
                "enableStandUpAndEndBlitzAction",
                "needsToRollForActionBlockingIsEasier",
            ],
            // Java: HitAndRun.postConstruct registers only canMoveAfterBlock
            SkillId::HitAndRun => &["canMoveAfterBlock"],
            // Java: bb2020/Fumblerooskie.postConstruct registers canDropBall. Was previously
            // entirely absent from this table, falling through to the empty default.
            SkillId::Fumblerooskie => &["canDropBall"],
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
            // Java: bb2016/bb2020 Swoop.postConstruct registers preventStuntyDodgeModifier, ttmScattersInSingleDirection,
            //   and CancelSkillProperty(ignoreTacklezonesWhenDodging) (union of both editions).
            SkillId::Swoop => &["preventStuntyDodgeModifier", "ttmScattersInSingleDirection", "cancelsIgnoreTacklezonesWhenDodging"],
            // Java: bb2025/special/WisdomOfTheWhiteDwarf.postConstruct registers canGrantSkillsToTeamMates
            SkillId::WisdomOfTheWhiteDwarf => &["canGrantSkillsToTeamMates"],
            SkillId::KickTeamMate => &["canKickTeamMates"],
            SkillId::ThrowTeamMate => &["canThrowTeamMates"],
            // Java: bb2016/RightStuff.postConstruct registers canBeThrown, canBeKicked, ignoreTackleWhenBlocked;
            //   bb2025/RightStuff.postConstruct registers canBeThrown, ignoreTackleWhenBlocked (no canBeKicked);
            //   bb2020/RightStuff.postConstruct registers canBeThrownIfStrengthIs3orLess (NOT canBeThrown) +
            //   ignoreTackleWhenBlocked — union of all three editions.
            SkillId::RightStuff => &["canBeThrown", "canBeKicked", "ignoreTackleWhenBlocked", "canBeThrownIfStrengthIs3orLess"],
            // Java: BallAndChain.postConstruct (union across bb2016/bb2020/bb2025, confirmed directly
            // against all three Java sources). All editions register the core "can only move, blocks
            // anyone in its path, dies to prone/stun" property set; bb2016-only adds
            // forceFullMovement/grabOutsideBlock/flipSameTeamOpponentToOtherTeam, bb2020+bb2025-only add
            // ignoreBlockAssists/preventPickup (+cancelsPreventOpponentFollowingUp/cancelsCanMoveBeforeBeingBlocked),
            // bb2025-only additionally adds preventSecureTheBallAction. "blocksLikeChainsaw" does NOT
            // exist in any edition's Java source — it was invented/stale and has been removed.
            SkillId::BallAndChain => &[
                "forceFullMovement",
                "grabOutsideBlock",
                "placedProneCausesInjuryRoll",
                "flipSameTeamOpponentToOtherTeam",
                "preventAutoMove",
                "ignoreBlockAssists",
                "preventPickup",
                "preventSecureTheBallAction",
                "preventRegularBlitzAction",
                "preventRegularBlockAction",
                "preventRegularFoulAction",
                "preventRegularHandOverAction",
                "preventRegularPassAction",
                "preventRecoverFromConcusionAction",
                "preventRecoverFromGazeAction",
                "preventStandUpAction",
                "canBlockMoreThanOnce",
                "forceFollowup",
                "canBlockSameTeamPlayer",
                "preventThrowTeamMateAction",
                "preventKickTeamMateAction",
                "goForItAfterBlock",
                "movesRandomly",
                "blocksDuringMove",
                "ignoreTacklezonesWhenMoving",
                "convertStunToKO",
                "cancelsCanBlockMoreThanOnce",
                "cancelsCanPileOnOpponent",
                "cancelsForceRollBeforeBeingBlocked",
                "cancelsInflictsConfusion",
                "cancelsPreventOpponentFollowingUp",
                "cancelsCanMoveBeforeBeingBlocked",
            ],
            // Java: BreatheFire.postConstruct (bb2020 and bb2025) registers
            // canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover + providesBlockAlternative
            SkillId::BreatheFire => &["canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover", "providesBlockAlternative"],
            // Java: WildAnimal.postConstruct registers enableStandUpAndEndBlitzAction + needsToRollForActionButKeepsTacklezone
            SkillId::WildAnimal => &["enableStandUpAndEndBlitzAction", "needsToRollForActionButKeepsTacklezone"],
            // Java: Loner.postConstruct registers hasToRollToUseTeamReroll + preventCardRabbitsFoot
            SkillId::Loner => &["hasToRollToUseTeamReroll", "preventCardRabbitsFoot"],
            // Java: Decay.postConstruct registers cancelsAllowsRaisingLineman + requiresSecondCasualtyRoll
            //   (mixed/Decay only has cancelsAllowsRaisingLineman, bb2016 also has requiresSecondCasualtyRoll)
            SkillId::Decay => &["cancelsAllowsRaisingLineman", "requiresSecondCasualtyRoll"],
            // Java Regeneration.postConstruct: bb2016 + bb2020 register preventRaiseFromDead, but
            // BB2025/Regeneration.java registers ONLY canRollToSaveFromInjury +
            // CancelSkillProperty(allowsRaisingLineman) — it does NOT prevent raise-from-dead. Per this
            // enum's "latest edition wins" convention, omit preventRaiseFromDead so a bb2025 Regeneration
            // player (e.g. a Necromantic Flesh Golem) that dies CAN be raised as a Zombie
            // (InjuryMechanic.canRaiseDead checks !preventRaiseFromDead — necromantic seed 89: Java raised
            // the dead Flesh Golem, Rust wrongly blocked it because the unioned property was present).
            // Stunty still carries preventRaiseFromDead (bb2016 + mixed/Stunty, all editions), unaffected.
            SkillId::Regeneration => &["canRollToSaveFromInjury", "cancelsAllowsRaisingLineman"],
            SkillId::GiveAndGo => &["canMoveAfterQuickPass", "canMoveAfterHandOff"],
            SkillId::RunningPass => &["canMoveAfterQuickPass"],
            // Java: DivingCatch.postConstruct registers canAttemptCatchInAdjacentSquares + addBonusForAccuratePass
            //   (the CatchModifier("Diving Catch", -1, REGULAR) it also registers is deferred — modifier
            //   system is stubbed as String types, see model::skill::skill modifier fields)
            SkillId::DivingCatch => &["canAttemptCatchInAdjacentSquares", "addBonusForAccuratePass"],
            // Java: bb2016/bb2020 NoHands.postConstruct registers preventCatch, preventHoldBall,
            //   preventRegularPassAction, preventRegularHandOverAction (union of both editions)
            SkillId::NoHands => &["preventCatch", "preventHoldBall", "preventRegularPassAction", "preventRegularHandOverAction"],
            // Java: bb2016/Titchy.postConstruct registers hasNoTacklezoneForDodging
            SkillId::Titchy => &["hasNoTacklezoneForDodging"],
            // Java: bb2016/Stakes.postConstruct registers providesStabBlockAlternative, canPerformArmourRollInsteadOfBlock, providesBlockAlternative
            SkillId::Stakes => &["providesStabBlockAlternative", "canPerformArmourRollInsteadOfBlock", "providesBlockAlternative"],
            // Java: bb2016/KickOffReturn.postConstruct registers canMoveDuringKickOffScatter
            SkillId::KickOffReturn => &["canMoveDuringKickOffScatter"],
            // Java: bb2016/Swarming.postConstruct registers canSneakExtraPlayersOntoPitch
            SkillId::Swarming => &["canSneakExtraPlayersOntoPitch"],
            // Java: bb2016/NervesOfSteel.postConstruct registers ignoreTacklezonesWhenPassing + ignoreTacklezonesWhenCatching
            SkillId::NervesOfSteel => &["ignoreTacklezonesWhenPassing", "ignoreTacklezonesWhenCatching"],
            // Java: bb2016/bb2020 MonstrousMouth.postConstruct registers
            // CancelSkillProperty(forceOpponentToDropBallOnPushback) (the CATCH reroll source is
            // handled live by ffb-engine::skill_behaviour::bb2020::monstrous_mouth_behaviour, not
            // through this property table).
            // NOTE: the Strip Ball / forceOpponentToDropBallOnPushback check in
            // ffb-engine::step::action::block::util_block_sequence::init_pushback is itself stubbed
            // out ("NamedProperties not yet implemented"), so this cancel currently has no live effect
            // regardless — pre-existing infra gap outside this audit's scope.
            SkillId::MonstrousMouth => &["cancelsForceOpponentToDropBallOnPushback", "canPinPlayers", "providesBlockAlternative"],
            // Java: bb2016/SafeThrow.postConstruct registers canCancelInterceptions + dontDropFumbles
            SkillId::SafeThrow => &["canCancelInterceptions", "dontDropFumbles"],
            // Java: Timmmber.postConstruct (bb2016 and mixed) registers allowStandUpAssists
            SkillId::Timmmber => &["allowStandUpAssists"],
            // Java: VeryLongLegs.postConstruct registers CancelSkillProperty(canCancelInterceptions) (BB2016)
            //   and CancelSkillProperty(canForceInterceptionRerollOfLongPasses) (BB2020) — union of both.
            SkillId::VeryLongLegs => &["cancelsCancelInterceptions", "cancelsCanForceInterceptionRerollOfLongPasses", "cancelsPassesAreNotIntercepted"],
            // Java: CloudBurster (BB2020) registers canForceInterceptionRerollOfLongPasses;
            // bb2025's CloudBurster registers the differently-named passesAreNotIntercepted instead — union of both.
            SkillId::CloudBurster => &["canForceInterceptionRerollOfLongPasses", "passesAreNotIntercepted"],
            // Java: FuriousOutburst.postConstruct registers canTeleportBeforeAndAfterAvRollAttack + canPerformArmourRollInsteadOfBlock
            SkillId::FuriousOutburst => &["canTeleportBeforeAndAfterAvRollAttack", "canPerformArmourRollInsteadOfBlock"],
            // Java: SafePass.postConstruct registers NamedProperties.dontDropFumbles
            SkillId::SafePass => &["dontDropFumbles"],
            // Java: Trickster.postConstruct registers NamedProperties.canMoveBeforeBeingBlocked
            SkillId::Trickster => &["canMoveBeforeBeingBlocked"],
            // Java: BlastIt.postConstruct registers NamedProperties.canReRollHmpScatter + grantsCatchBonusToReceiver
            SkillId::BlastIt => &["canReRollHmpScatter", "grantsCatchBonusToReceiver"],
            // Java: BurstOfSpeed.postConstruct registers canMakeAnExtraGfiOnce
            SkillId::BurstOfSpeed => &["canMakeAnExtraGfiOnce"],
            // Java: ConsummateProfessional.postConstruct registers canRerollSingleDieOncePerPeriod
            //   (also registers a reroll source, ReRolledActions.SINGLE_DIE/ReRollSources.CONSUMMATE_PROFESSIONAL,
            //   but there is no live reroll-source table to mirror that in — see NOTE in consummate_professional.rs)
            SkillId::ConsummateProfessional => &["canRerollSingleDieOncePerPeriod"],
            // Java: bb2020/ExcuseMeAreYouAZoat.postConstruct registers canGainGaze (bb2025 registers
            //   canGazeAutomaticallyThreeSquaresAway instead) — union of both editions
            SkillId::ExcuseMeAreYouAZoat => &["canGainGaze", "canGazeAutomaticallyThreeSquaresAway"],
            // Java: ThenIStartedBlastin.postConstruct registers canBlastRemotePlayer
            SkillId::ThenIStartedBlastin => &["canBlastRemotePlayer"],
            // Java: TwoForOne.postConstruct registers reducesLonerRollIfPartnerIsHurt
            SkillId::TwoForOne => &["reducesLonerRollIfPartnerIsHurt"],
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
            // Java: Indomitable.postConstruct registers canDoubleStrengthAfterDauntless
            SkillId::Indomitable => &["canDoubleStrengthAfterDauntless"],
            // Java: PilingOnBehaviour registers canPileOnOpponent
            SkillId::PilingOn => &["canPileOnOpponent"],
            // Java: bb2016/WeepingDagger.postConstruct registers appliesPoisonOnBadlyHurt
            SkillId::WeepingDagger => &["appliesPoisonOnBadlyHurt"],
            // Java: bb2025/PumpUpTheCrowd.postConstruct registers grantsTeamReRollWhenCausingBlockCas
            // Java: bb2020/PumpUpTheCrowd.postConstruct registers grantsTeamReRollWhenCausingCas
            SkillId::PumpUpTheCrowd => &["grantsTeamReRollWhenCausingBlockCas", "grantsTeamReRollWhenCausingCas"],
            // Java: bb2016+bb2025/PutTheBootIn.postConstruct register only canAlwaysAssistFouls;
            //   bb2020/SneakyGit.postConstruct additionally registers canMoveAfterFoul — union of both.
            SkillId::SneakyGit | SkillId::PutTheBootIn => &["canAlwaysAssistFouls", "canMoveAfterFoul"],
            // Java: bb2020+bb2025/Defensive.postConstruct both register
            // CancelSkillProperty(assistsBlocksInTacklezones) + CancelSkillProperty(assistsFoulsInTacklezones);
            // bb2025 additionally registers CancelSkillProperty(canAlwaysAssistFouls) — union of both.
            SkillId::Defensive => &[
                "cancelsAssistsBlocksInTacklezones",
                "cancelsAssistsFoulsInTacklezones",
                "cancelsCanAlwaysAssistFouls",
            ],
            // Java: bb2020/PileDriver.postConstruct + bb2025/PileDriver.postConstruct register canFoulAfterBlock
            SkillId::PileDriver => &["canFoulAfterBlock"],
            // Java: bb2016/SecretWeapon.postConstruct registers preventStuntyDodgeModifier, getsSentOffAtEndOfDrive,
            //   and CancelSkillProperty(ignoreTacklezonesWhenDodging); mixed/SecretWeapon.postConstruct
            //   (base class) registers getsSentOffAtEndOfDrive — union of both.
            SkillId::SecretWeapon => &["preventStuntyDodgeModifier", "getsSentOffAtEndOfDrive", "cancelsIgnoreTacklezonesWhenDodging"],
            // Java: mixed/special/KeenPlayer.postConstruct registers canJoinTeamIfLessThanEleven + getsSentOffAtEndOfDrive
            SkillId::KeenPlayer => &["canJoinTeamIfLessThanEleven", "getsSentOffAtEndOfDrive"],
            // Java: mixed/special/AllYouCanEat.postConstruct registers canUseThrowBombActionTwice
            SkillId::AllYouCanEat => &["canUseThrowBombActionTwice"],
            // Java: mixed/special/BalefulHex.postConstruct registers canMakeOpponentMissTurn
            SkillId::BalefulHex => &["canMakeOpponentMissTurn"],
            // Java: mixed/special/BlackInk.postConstruct registers canGazeAutomatically
            SkillId::BlackInk => &["canGazeAutomatically"],
            // Java: mixed/special/BugmansXXXXXX.postConstruct registers canReRollOnesOnKORecovery
            SkillId::BugmansXXXXXX => &["canReRollOnesOnKORecovery"],
            // Java: mixed/special/CatchOfTheDay.postConstruct registers canGetBallOnGround
            SkillId::CatchOfTheDay => &["canGetBallOnGround"],
            // Java: mixed/special/FuryOfTheBloodGod.postConstruct registers canPerformTwoBlocksAfterFailedFury
            SkillId::FuryOfTheBloodGod => &["canPerformTwoBlocksAfterFailedFury"],
            // Java: mixed/special/GoredByTheBull.postConstruct registers canAddBlockDie + providesBlockAlternativeDuringBlitz
            SkillId::GoredByTheBull => &["canAddBlockDie", "providesBlockAlternativeDuringBlitz"],
            // Java: mixed/special/HalflingLuck.postConstruct registers canRerollSingleDieOncePerPeriod
            SkillId::HalflingLuck => &["canRerollSingleDieOncePerPeriod"],
            // Java: mixed/special/IllBeBack.postConstruct registers ignoreFirstSecretWeaponSentOff
            SkillId::IllBeBack => &["ignoreFirstSecretWeaponSentOff"],
            // Java: mixed/special/KickEmWhileTheyReDown.postConstruct registers canUseChainsawOnDownedOpponents
            SkillId::KickEmWhileTheyReDown => &["canUseChainsawOnDownedOpponents"],
            // Java: mixed/special/LookIntoMyEyes.postConstruct registers canStealBallFromOpponent
            SkillId::LookIntoMyEyes => &["canStealBallFromOpponent"],
            // Java: mixed/special/BoundingLeap.postConstruct registers canIgnoreJumpModifiers
            SkillId::BoundingLeap => &["canIgnoreJumpModifiers"],
            // Java: mixed/IronHardSkin.postConstruct registers cancelsReducesArmourToFixedValue + ignores properties
            SkillId::IronHardSkin => &[
                "cancelsReducesArmourToFixedValue",
                "ignoresArmourModifiersFromFouls",
                "ignoresArmourModifiersFromSkills",
                "ignoresArmourModifiersFromSpecialEffects",
            ],
            // Java: bb2020+bb2025/LordOfChaos.postConstruct registers grantsSingleUseTeamRerollWhenOnPitch
            SkillId::LordOfChaos => &["grantsSingleUseTeamRerollWhenOnPitch", "canRerollSingleBlockDieOncePerPeriod"],
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
            // Java: bb2025/Incorporeal.postConstruct registers canAvoidDodging; bb2020/Incorporeal.postConstruct
            //   registers canAddStrengthToDodge instead — union of both editions
            SkillId::Incorporeal => &["canAvoidDodging", "canAddStrengthToDodge"],
            // Java: HailMaryPass.postConstruct registers canPassToAnySquare (canGainHailMary belongs to
            //   the unrelated mixed/special/ShotToNothing skill, not HailMaryPass)
            SkillId::HailMaryPass => &["canPassToAnySquare"],
            // Java: ShotToNothing.postConstruct registers canGainHailMary
            SkillId::ShotToNothing => &["canGainHailMary"],
            // Java: SafePairOfHands.postConstruct registers canPlaceBallWhenKnockedDownOrPlacedProne
            SkillId::SafePairOfHands => &["canPlaceBallWhenKnockedDownOrPlacedProne"],
            // Java: SaboteurBehaviour registers canSabotageBlockerOnKnockdown
            SkillId::Saboteur => &["canSabotageBlockerOnKnockdown"],
            // Java: WatchOut.postConstruct registers ignoresDefenderStumblesResultForFirstBlock
            SkillId::WatchOut => &["ignoresDefenderStumblesResultForFirstBlock"],
            // Java: mixed/special/QuickBite.postConstruct registers canAttackOpponentForBallAfterCatch
            SkillId::QuickBite => &["canAttackOpponentForBallAfterCatch"],
            // Java: bb2020/ProjectileVomit.postConstruct registers providesBlockAlternative +
            //   canPerformArmourRollInsteadOfBlockThatMightFail
            SkillId::ProjectileVomit => &["providesBlockAlternative", "canPerformArmourRollInsteadOfBlockThatMightFail"],
            // Java: common/JumpUp.postConstruct registers canStandUpForFree
            SkillId::JumpUp => &["canStandUpForFree"],
            // Java: common/StripBall.postConstruct registers forceOpponentToDropBallOnPushback
            SkillId::StripBall => &["forceOpponentToDropBallOnPushback"],
            // Java: common/SureHands.postConstruct registers CancelSkillProperty(forceOpponentToDropBallOnPushback)
            SkillId::SureHands => &["cancelsForceOpponentToDropBallOnPushback"],
            // Java: mixed/ArmBar.postConstruct registers affectsEitherArmourOrInjuryOnDodge + ...OnJump
            SkillId::ArmBar => &["affectsEitherArmourOrInjuryOnDodge", "affectsEitherArmourOrInjuryOnJump"],
            // Java: mixed/BigHand and bb2025/BigHand register ignoreTacklezonesWhenPickingUp + ignoreWeatherWhenPickingUp
            SkillId::BigHand => &["ignoreTacklezonesWhenPickingUp", "ignoreWeatherWhenPickingUp"],
            // Java: mixed/Bloodlust.postConstruct registers enableStandUpAndEndBlitzAction + needsToRollForActionBlockingIsEasier
            SkillId::BloodLust => &["enableStandUpAndEndBlitzAction", "needsToRollForActionBlockingIsEasier"],
            // Java: mixed/Leader and bb2025/Leader register grantsTeamReRollWhenOnPitch
            SkillId::Leader => &["grantsTeamReRollWhenOnPitch"],
            // Java: mixed/MyBall.postConstruct registers preventRegularHandOverAction + preventRegularPassAction
            //   + CancelSkillProperty(canDropBall)
            SkillId::MyBall => &["preventRegularHandOverAction", "preventRegularPassAction", "cancelsCanDropBall"],
            // Java: mixed/PickMeUp.postConstruct registers canStandUpTeamMates
            SkillId::PickMeUp => &["canStandUpTeamMates"],
            // Java: mixed/PlagueRidden.postConstruct registers allowsRaisingLineman
            SkillId::PlagueRidden => &["allowsRaisingLineman"],
            // Java: mixed/UnchannelledFury.postConstruct registers enableStandUpAndEndBlitzAction
            //   + needsToRollForActionButKeepsTacklezone
            SkillId::UnchannelledFury => &["enableStandUpAndEndBlitzAction", "needsToRollForActionButKeepsTacklezone"],
            // Java: mixed/special/PrimalSavagery.postConstruct registers canLashOutAgainstOpponents
            SkillId::PrimalSavagery => &["canLashOutAgainstOpponents"],
            // Java: mixed/special/RaidingParty.postConstruct registers canMoveOpenTeamMate
            SkillId::RaidingParty => &["canMoveOpenTeamMate"],
            // Java: mixed/special/Reliable.postConstruct registers fumbledPlayerLandsSafely
            SkillId::Reliable => &["fumbledPlayerLandsSafely"],
            // Java: mixed/special/SavageBlow.postConstruct registers canReRollAnyNumberOfBlockDice
            SkillId::SavageBlow => &["canReRollAnyNumberOfBlockDice"],
            // Java: mixed/special/SneakiestOfTheLot.postConstruct registers allowsAdditionalFoul
            SkillId::SneakiestOfTheLot => &["allowsAdditionalFoul"],
            // Java: mixed/special/StarOfTheShow.postConstruct registers canGrantReRollAfterTouchdown
            SkillId::StarOfTheShow => &["canGrantReRollAfterTouchdown"],
            // Java: mixed/special/SwiftAsTheBreeze.postConstruct registers the three ignore-modifier-after-roll properties
            SkillId::SwiftAsTheBreeze => &["canChooseToIgnoreDodgeModifierAfterRoll", "canChooseToIgnoreRushModifierAfterRoll", "canChooseToIgnoreJumpModifierAfterRoll"],
            // Java: mixed/special/TastyMorsel.postConstruct registers canBiteOpponents
            SkillId::TastyMorsel => &["canBiteOpponents"],
            // Java: mixed/special/TheFlashingBlade.postConstruct registers canPerformArmourRollInsteadOfBlock + canStabAndMoveAfterwards
            SkillId::TheFlashingBlade => &["canPerformArmourRollInsteadOfBlock", "canStabAndMoveAfterwards"],
            // Java: mixed/special/ThinkingMansTroll.postConstruct registers canRerollSingleDieOncePerPeriod
            SkillId::ThinkingMansTroll => &["canRerollSingleDieOncePerPeriod"],
            // Java: mixed/special/UnstoppableMomentum.postConstruct registers canRerollSingleBlockDieDuringBlitz
            SkillId::UnstoppableMomentum => &["canRerollSingleBlockDieDuringBlitz"],
            // Java: mixed/special/ViciousVines.postConstruct registers canBlockOverDistance
            SkillId::ViciousVines => &["canBlockOverDistance"],
            // Java: bb2025/Bullseye.postConstruct registers canSkipTtmScatterOnSuperbThrow
            SkillId::Bullseye => &["canSkipTtmScatterOnSuperbThrow"],
            // Java: bb2025/Hatred.postConstruct registers canRerollSingleSkull + canBeGainedByGettingEven
            SkillId::Hatred => &["canRerollSingleSkull", "canBeGainedByGettingEven"],
            // Java: bb2025/NoBall.postConstruct registers the six prevent-* properties
            SkillId::NoBall => &["preventCatch", "preventHoldBall", "preventRegularPassAction", "preventRegularHandOverAction", "preventSecureTheBallAction", "preventPuntAction"],
            // Java: bb2025/Taunt.postConstruct registers forceOpponentToFollowUp
            SkillId::Taunt => &["forceOpponentToFollowUp"],
            // Java: bb2025/Unsteady.postConstruct registers preventSecureTheBallAction
            SkillId::Unsteady => &["preventSecureTheBallAction"],
            // Java: bb2025/Fumblerooski.postConstruct registers canDropBall (distinct from bb2020 Fumblerooskie)
            SkillId::Fumblerooski => &["canDropBall"],
            // Java: bb2025/special/ASneakyPair.postConstruct registers affectsEitherArmourOrInjuryWithPartner
            SkillId::ASneakyPair => &["affectsEitherArmourOrInjuryWithPartner"],
            // Java: bb2025/special/BlastinSolvesEverything.postConstruct registers canBlastRemotePlayer
            SkillId::BlastinSolvesEverything => &["canBlastRemotePlayer"],
            // Java: bb2025/special/TeamCaptain.postConstruct registers canSaveReRolls + needsToBeSetUp
            SkillId::TeamCaptain => &["canSaveReRolls", "needsToBeSetUp"],
            // Java: bb2025/special/WoodlandFury.postConstruct registers canRerollSingleBlockDieWhenWouldBeKnockedDown
            SkillId::WoodlandFury => &["canRerollSingleBlockDieWhenWouldBeKnockedDown"],
            // Java: bb2025/special/WorkingInTandem.postConstruct registers canRerollSingleBlockDieWhenPartnerIsMarking
            //   + canPassToPartnerWithNoModifiers
            SkillId::WorkingInTandem => &["canRerollSingleBlockDieWhenPartnerIsMarking", "canPassToPartnerWithNoModifiers"],
            _ => &[],
        }
    }

    /// Re-roll sources this skill registers: `(rerolled_action, priority)` pairs.
    ///
    /// 1:1 translation of the Java `Skill.registerRerollSource(ReRolledAction,
    /// ReRollSource)` registrations in each skill's `postConstruct()`, folded into
    /// a static table like `properties()` (the per-skill-struct
    /// `register_reroll_source` map is a faithful but inert translation — this
    /// table is what the engine consults, via
    /// `abstract_step_with_re_roll::find_skill_reroll_source`).
    ///
    /// Action keys use the Rust engine's action vocabulary where a caller exists
    /// (Java `PICK_UP` → `"PICKUP"`; Java `GO_FOR_IT`(bb2016)/`RUSH`(bb2020+) →
    /// `"GFI"`); Java names are kept for actions no Rust step asks about yet
    /// (`SINGLE_DIE`, `SINGLE_BLOCK_DIE`, `SINGLE_SKULL`, `SINGLE_BOTH_DOWN`,
    /// `MULTI_BLOCK_DICE`, `SINGLE_DIE_PER_ACTIVATION`, `KICK_TEAM_MATE`).
    /// The reroll source is always the skill itself; priority mirrors Java
    /// `ReRollSources` (every source is priority 1 except THE_BALLISTA = 2).
    /// Cross-edition union, like `properties()` — per-edition steps only ever ask
    /// for their own edition's action string.
    pub fn reroll_sources(self) -> &'static [(&'static str, i32)] {
        match self {
            // Java: common/Catch registers CATCH → CATCH
            SkillId::Catch => &[("CATCH", 1)],
            // Java: common/Pass registers PASS → PASS
            SkillId::Pass => &[("PASS", 1)],
            // Java: common/SureHands registers PICK_UP → SURE_HANDS
            SkillId::SureHands => &[("PICKUP", 1)],
            // Java: mixed/Dodge and bb2025/Dodge register DODGE → DODGE
            SkillId::Dodge => &[("DODGE", 1)],
            // Java: bb2016/SureFeet GO_FOR_IT → SURE_FEET; bb2020+bb2025 RUSH → SURE_FEET
            SkillId::SureFeet => &[("GFI", 1)],
            // Java: bb2016+bb2020/MonstrousMouth register CATCH → MONSTROUS_MOUTH
            SkillId::MonstrousMouth => &[("CATCH", 1)],
            // Java: bb2020+bb2025 special/WhirlingDervish register DIRECTION → WHIRLING_DERVISH
            SkillId::WhirlingDervish => &[("DIRECTION", 1)],
            // Java: bb2020/special/MesmerizingDance registers HYPNOTIC_GAZE → MESMERIZING_DANCE
            SkillId::MesmerizingDance => &[("HYPNOTIC_GAZE", 1)],
            // Java: bb2025/special/MesmerisingDance registers HYPNOTIC_GAZE → MESMERISING_DANCE
            SkillId::MesmerisingDance => &[("HYPNOTIC_GAZE", 1)],
            // Java: bb2020/special/ConsummateProfessional registers SINGLE_DIE → CONSUMMATE_PROFESSIONAL
            SkillId::ConsummateProfessional => &[("SINGLE_DIE", 1)],
            // Java: bb2020/special/TheBallista PASS+THROW_TEAM_MATE; bb2025 adds KICK_TEAM_MATE.
            // THE_BALLISTA is the only priority-2 source in Java ReRollSources.
            SkillId::TheBallista => &[("PASS", 2), ("THROW_TEAM_MATE", 2), ("KICK_TEAM_MATE", 2)],
            // Java: mixed/special/UnstoppableMomentum registers SINGLE_BLOCK_DIE → UNSTOPPABLE_MOMENTUM
            SkillId::UnstoppableMomentum => &[("SINGLE_BLOCK_DIE", 1)],
            // Java: mixed/special/ThinkingMansTroll registers SINGLE_DIE → THINKING_MANS_TROLL
            SkillId::ThinkingMansTroll => &[("SINGLE_DIE", 1)],
            // Java: mixed/special/SavageBlow registers MULTI_BLOCK_DICE → SAVAGE_BLOW
            SkillId::SavageBlow => &[("MULTI_BLOCK_DICE", 1)],
            // Java: mixed/special/HalflingLuck registers SINGLE_DIE → HALFLING_LUCK
            SkillId::HalflingLuck => &[("SINGLE_DIE", 1)],
            // Java: mixed/special/BoundingLeap registers JUMP → BOUNDING_LEAP
            SkillId::BoundingLeap => &[("JUMP", 1)],
            // Java: mixed/special/BlindRage registers DAUNTLESS → BLIND_RAGE
            SkillId::BlindRage => &[("DAUNTLESS", 1)],
            // Java: bb2025/Pro registers SINGLE_DIE_PER_ACTIVATION → PRO
            SkillId::Pro => &[("SINGLE_DIE_PER_ACTIVATION", 1)],
            // Java: bb2025/Kick registers PUNT_DIRECTION → KICK + PUNT_DISTANCE → KICK
            SkillId::Kick => &[("PUNT_DIRECTION", 1), ("PUNT_DISTANCE", 1)],
            // Java: bb2025/Hatred registers SINGLE_SKULL → HATRED
            SkillId::Hatred => &[("SINGLE_SKULL", 1)],
            // Java: bb2025/Brawler registers SINGLE_BOTH_DOWN → BRAWLER
            SkillId::Brawler => &[("SINGLE_BOTH_DOWN", 1)],
            // Java: bb2025/Swoop registers RIGHT_STUFF → SWOOP
            SkillId::Swoop => &[("RIGHT_STUFF", 1)],
            // Java: bb2025/special/WorkingInTandem registers SINGLE_BLOCK_DIE → WORKING_IN_TANDEM
            SkillId::WorkingInTandem => &[("SINGLE_BLOCK_DIE", 1)],
            // Java: bb2025/special/WoodlandFury registers SINGLE_BLOCK_DIE → WOODLAND_FURY
            SkillId::WoodlandFury => &[("SINGLE_BLOCK_DIE", 1)],
            // Java: bb2025/special/LordOfChaos registers SINGLE_BLOCK_DIE → LORD_OF_CHAOS
            SkillId::LordOfChaos => &[("SINGLE_BLOCK_DIE", 1)],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {

    /// Java registers a skill's NamedProperties in the PER-EDITION skill class' postConstruct, and
    /// `RightStuff` genuinely differs: bb2016 grants canBeThrown + canBeKicked, bb2020 grants
    /// canBeThrownIfStrengthIs3orLess INSTEAD of canBeThrown, bb2025 grants canBeThrown alone.
    /// The edition-agnostic union made a BB2020 Right Stuff player answer true to canBeThrown, so
    /// Rust offered a Throw Team-Mate target where Java's list is empty (chaos_pact bb2020 seed 22).
    #[test]
    fn right_stuff_properties_are_edition_specific() {
        use crate::enums::Rules;

        let bb2016 = SkillId::RightStuff.properties_for(Rules::Bb2016);
        assert!(bb2016.contains(&"canBeThrown"));
        assert!(bb2016.contains(&"canBeKicked"), "only bb2016 grants canBeKicked");

        let bb2020 = SkillId::RightStuff.properties_for(Rules::Bb2020);
        assert!(!bb2020.contains(&"canBeThrown"),
            "bb2020 must NOT grant canBeThrown — it grants canBeThrownIfStrengthIs3orLess");
        assert!(bb2020.contains(&"canBeThrownIfStrengthIs3orLess"));
        assert!(!bb2020.contains(&"canBeKicked"));

        let bb2025 = SkillId::RightStuff.properties_for(Rules::Bb2025);
        assert!(bb2025.contains(&"canBeThrown"));
        assert!(!bb2025.contains(&"canBeKicked"));
        assert!(!bb2025.contains(&"canBeThrownIfStrengthIs3orLess"));

        // Every edition keeps the shared property.
        for r in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert!(SkillId::RightStuff.properties_for(r).contains(&"ignoreTackleWhenBlocked"));
        }
    }

    /// The complete set of skills whose Java `postConstruct` differs across editions, extracted
    /// `category_and_name_for` is generated from the `super("<name>", SkillCategory.<CAT>)` call in
    /// every Java skill class, resolved edition-first then `mixed` then `common`. These pin the
    /// values the Intensive Training prayer depends on: it offers every skill whose category is one
    /// of the player's position categories, sorted by NAME.
    #[test]
    fn category_and_name_match_the_java_skill_constructors() {
        use crate::enums::Rules;
        use crate::model::skill_category::SkillCategory;
        // skill/common/Block.java: super("Block", SkillCategory.GENERAL) - same in every edition.
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025, Rules::Common] {
            assert_eq!(SkillId::Block.category_and_name_for(rules), (SkillCategory::General, "Block"));
        }
        // Sure Hands / Tackle are General too - the three that decide the sorted-first entry.
        assert_eq!(SkillId::SureHands.category_and_name_for(Rules::Bb2020).0, SkillCategory::General);
        assert_eq!(SkillId::Tackle.category_and_name_for(Rules::Bb2020).0, SkillCategory::General);
    }

    /// The table must stay edition-aware: Bone-Head is EXTRAORDINARY/"Bone-Head" in BB2016 and
    /// TRAIT/"Bone Head" in BB2020+ (the hyphen difference that has bitten roster loading before),
    /// and Dirty Player moved GENERAL -> DEVIOUS, which changes whether Intensive Training can
    /// offer it to a General-category lineman at all.
    #[test]
    fn category_and_name_are_edition_specific_where_java_diverges() {
        use crate::enums::Rules;
        use crate::model::skill_category::SkillCategory;
        assert_eq!(SkillId::BoneHead.category_and_name_for(Rules::Bb2016),
                   (SkillCategory::Extraordinary, "Bone-Head"));
        assert_eq!(SkillId::BoneHead.category_and_name_for(Rules::Bb2020),
                   (SkillCategory::Trait, "Bone Head"));
        // skill/bb2016 and skill/bb2020 both declare GENERAL; only skill/bb2025 moves it to DEVIOUS.
        assert_eq!(SkillId::DirtyPlayer.category_and_name_for(Rules::Bb2016).0, SkillCategory::General);
        assert_eq!(SkillId::DirtyPlayer.category_and_name_for(Rules::Bb2020).0, SkillCategory::General);
        assert_eq!(SkillId::DirtyPlayer.category_and_name_for(Rules::Bb2025).0, SkillCategory::Devious);
    }

    /// Java sorts the offered skills with `Comparator.comparing(Skill::getName)`, so for a position
    /// whose only normal category is General the coach is offered Block first - which is exactly
    /// what the parity harness picks (`RandomStrategy` case SELECT_SKILL sends `skills.get(0)`).
    #[test]
    fn block_sorts_first_among_bb2020_general_skills() {
        use crate::enums::Rules;
        use crate::model::skill_category::SkillCategory;
        let mut general: Vec<&str> = [
            SkillId::Block, SkillId::Dauntless, SkillId::Fend, SkillId::Frenzy, SkillId::Kick,
            SkillId::Pro, SkillId::Shadowing, SkillId::StripBall, SkillId::SureHands,
            SkillId::Tackle, SkillId::Wrestle,
        ]
        .into_iter()
        .filter(|s| s.category_and_name_for(Rules::Bb2020).0 == SkillCategory::General)
        .map(|s| s.category_and_name_for(Rules::Bb2020).1)
        .collect();
        general.sort();
        assert_eq!(general.first().copied(), Some("Block"));
    }

    /// mechanically from `skill/bb2016|bb2020|bb2025/*.java`. Spot-checks one representative
    /// divergence per skill so a future edit to the table cannot quietly drop one.
    #[test]
    fn every_edition_divergent_skill_is_tabled() {
        use crate::enums::Rules;

        // BB2016-only extras on Ball & Chain; BB2025-only preventSecureTheBallAction.
        assert!(SkillId::BallAndChain.properties_for(Rules::Bb2016).contains(&"forceFullMovement"));
        assert!(!SkillId::BallAndChain.properties_for(Rules::Bb2020).contains(&"forceFullMovement"));
        assert!(SkillId::BallAndChain.properties_for(Rules::Bb2020).contains(&"preventPickup"));
        assert!(SkillId::BallAndChain.properties_for(Rules::Bb2025).contains(&"preventSecureTheBallAction"));
        assert!(!SkillId::BallAndChain.properties_for(Rules::Bb2020).contains(&"preventSecureTheBallAction"));

        // BB2020-only preventStuntyDodgeModifier on Bombardier and Chainsaw.
        assert!(SkillId::Bombardier.properties_for(Rules::Bb2020).contains(&"preventStuntyDodgeModifier"));
        assert!(!SkillId::Bombardier.properties_for(Rules::Bb2025).contains(&"preventStuntyDodgeModifier"));
        assert!(SkillId::Chainsaw.properties_for(Rules::Bb2016).contains(&"makesStrengthTestObsolete"));
        assert!(!SkillId::Chainsaw.properties_for(Rules::Bb2020).contains(&"makesStrengthTestObsolete"));

        // CloudBurster changes property entirely between BB2020 and BB2025.
        assert!(SkillId::CloudBurster.properties_for(Rules::Bb2020).contains(&"canForceInterceptionRerollOfLongPasses"));
        assert!(SkillId::CloudBurster.properties_for(Rules::Bb2025).contains(&"passesAreNotIntercepted"));
        assert!(!SkillId::CloudBurster.properties_for(Rules::Bb2020).contains(&"passesAreNotIntercepted"));

        // canGazeDuringMove is BB2016-only.
        assert!(SkillId::HypnoticGaze.properties_for(Rules::Bb2016).contains(&"canGazeDuringMove"));
        assert!(!SkillId::HypnoticGaze.properties_for(Rules::Bb2020).contains(&"canGazeDuringMove"));

        // Leap gains a BB2020-only property; MonstrousMouth has properties only in BB2025.
        assert!(SkillId::Leap.properties_for(Rules::Bb2020).contains(&"failedRushForJumpAlwaysLandsInTargetSquare"));
        assert!(!SkillId::Leap.properties_for(Rules::Bb2025).contains(&"failedRushForJumpAlwaysLandsInTargetSquare"));
        assert!(SkillId::MonstrousMouth.properties_for(Rules::Bb2025).contains(&"canPinPlayers"));
        assert!(!SkillId::MonstrousMouth.properties_for(Rules::Bb2020).contains(&"canPinPlayers"));

        // Piling On loses canPileOnOpponent in BB2020; Regeneration loses preventRaiseFromDead in BB2025.
        assert!(SkillId::PilingOn.properties_for(Rules::Bb2016).contains(&"canPileOnOpponent"));
        assert!(!SkillId::PilingOn.properties_for(Rules::Bb2020).contains(&"canPileOnOpponent"));
        assert!(SkillId::Regeneration.properties_for(Rules::Bb2020).contains(&"preventRaiseFromDead"));
        assert!(!SkillId::Regeneration.properties_for(Rules::Bb2025).contains(&"preventRaiseFromDead"));

        // Sneaky Git and Stab and Swoop.
        assert!(SkillId::SneakyGit.properties_for(Rules::Bb2020).contains(&"canMoveAfterFoul"));
        assert!(!SkillId::SneakyGit.properties_for(Rules::Bb2025).contains(&"canAlwaysAssistFouls"));
        assert!(SkillId::Stab.properties_for(Rules::Bb2020).contains(&"providesMultipleBlockAlternative"));
        assert!(!SkillId::Stab.properties_for(Rules::Bb2025).contains(&"providesMultipleBlockAlternative"));
        assert!(SkillId::Swoop.properties_for(Rules::Bb2020).contains(&"preventStuntyDodgeModifier"));
        assert!(!SkillId::Swoop.properties_for(Rules::Bb2025).contains(&"preventStuntyDodgeModifier"));

        // The seven skills the first extraction MISSED because they have no per-edition class and
        // resolve through `skill/mixed/` (or `skill/common/`).
        assert!(SkillId::Decay.properties_for(Rules::Bb2016).contains(&"requiresSecondCasualtyRoll"));
        assert!(!SkillId::Decay.properties_for(Rules::Bb2020).contains(&"requiresSecondCasualtyRoll"));
        assert!(!SkillId::DivingTackle.properties_for(Rules::Bb2016).contains(&"canAttemptToTackleJumpingPlayer"));
        assert!(SkillId::DivingTackle.properties_for(Rules::Bb2020).contains(&"canAttemptToTackleJumpingPlayer"));
        assert!(SkillId::Juggernaut.properties_for(Rules::Bb2025).contains(&"canConvertBothDownToPush"));
        assert!(!SkillId::Juggernaut.properties_for(Rules::Bb2020).contains(&"canConvertBothDownToPush"));
        assert!(SkillId::MultipleBlock.properties_for(Rules::Bb2016).contains(&"canBlockMoreThanOnce"));
        assert!(SkillId::MultipleBlock.properties_for(Rules::Bb2020).contains(&"canBlockTwoAtOnce"));
        assert!(!SkillId::PrehensileTail.properties_for(Rules::Bb2016).contains(&"makesJumpingHarder"));
        assert!(SkillId::PrehensileTail.properties_for(Rules::Bb2020).contains(&"makesJumpingHarder"));
        assert!(SkillId::SecretWeapon.properties_for(Rules::Bb2016).contains(&"preventStuntyDodgeModifier"));
        assert!(!SkillId::SecretWeapon.properties_for(Rules::Bb2020).contains(&"preventStuntyDodgeModifier"));
        assert!(!SkillId::Stunty.properties_for(Rules::Bb2016).contains(&"passesAreInterceptedEasier"));
        assert!(SkillId::Stunty.properties_for(Rules::Bb2020).contains(&"passesAreInterceptedEasier"));
        // Stunty ignores tackle zones when dodging in EVERY edition — this is why Rust's empty
        // dodge-modifier list for a Stunty goblin is correct, not a lookup failure.
        for r in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert!(SkillId::Stunty.properties_for(r).contains(&"ignoreTacklezonesWhenDodging"));
        }
    }

    /// INVARIANT: for every tabled skill, the union of its three per-edition property sets must be a
    /// SUBSET of `properties()`. A tabled arm that lists something the union does not have means the
    /// generator and the union disagree; the reverse (union ⊋ per-edition union) is expected and fine,
    /// since a property can be edition-specific. This is the check that would have caught the first
    /// generator missing `registerProperty(new CancelSkillProperty(..))` entirely — it silently
    /// dropped `cancelsAllowsRaisingLineman` and friends from every tabled skill.
    #[test]
    fn tabled_properties_never_invent_anything_the_union_lacks() {
        use crate::enums::Rules;
        // The tabled skills, listed explicitly (SkillId has no iterator).
        let tabled = [
            SkillId::BallAndChain, SkillId::Bombardier, SkillId::Chainsaw, SkillId::CloudBurster,
            SkillId::Decay, SkillId::DivingTackle, SkillId::HypnoticGaze, SkillId::Juggernaut,
            SkillId::Leap, SkillId::MonstrousMouth, SkillId::MultipleBlock, SkillId::PilingOn,
            SkillId::PrehensileTail, SkillId::Regeneration, SkillId::RightStuff,
            SkillId::SecretWeapon, SkillId::SneakyGit, SkillId::Stab, SkillId::Stunty, SkillId::Swoop,
        ];
        let mut missing: Vec<String> = Vec::new();
        for skill in tabled {
            let union: Vec<&str> = skill.properties().to_vec();
            for r in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
                for p in skill.properties_for(r) {
                    // KNOWN, DELIBERATE trim: the union omits Regeneration's `preventRaiseFromDead`
                    // even though bb2016 and bb2020 register it, because `properties()` is consumed
                    // edition-agnostically and a BB2025 Regeneration player must stay raisable
                    // (see `regeneration_does_not_prevent_raise_from_dead_bb2025`, necromantic seed
                    // 89). The per-edition table is the correct place for the real values; this
                    // exemption goes away once the remaining consumers move to `properties_for`.
                    if skill == SkillId::Regeneration && *p == "preventRaiseFromDead" { continue; }
                    if !union.contains(p) { missing.push(format!("{skill:?}/{r:?}: {p}")); }
                }
            }
        }
        assert!(missing.is_empty(), "per-edition table lists properties the union lacks: {missing:#?}");
    }

    /// Skills with no per-edition arm must fall through to the union unchanged, so adding
    /// `properties_for` cannot silently alter anything else.
    #[test]
    fn properties_for_falls_through_to_the_union_for_other_skills() {
        use crate::enums::Rules;
        for skill in [SkillId::Dodge, SkillId::Block, SkillId::MightyBlow, SkillId::Tackle] {
            for r in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
                assert_eq!(skill.properties_for(r), skill.properties(),
                    "{skill:?} must be unaffected by properties_for under {r:?}");
            }
        }
    }
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
        // Bug: bb2020/bb2025 Chainsaw.postConstruct also registers providesBlockAlternative,
        // providesFoulingAlternative, and (bb2020-only) preventStuntyDodgeModifier +
        // cancelsIgnoreTacklezonesWhenDodging, none of which were present before the fix.
        let props = SkillId::Chainsaw.properties();
        assert_eq!(props.len(), 9);
        assert!(props.contains(&"providesBlockAlternative"));
        assert!(props.contains(&"providesFoulingAlternative"));
        assert!(props.contains(&"preventStuntyDodgeModifier"));
        assert!(props.contains(&"cancelsIgnoreTacklezonesWhenDodging"));
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
    fn properties_leap_has_failed_rush_lands_in_target_square() {
        // Bug: bb2020/Leap.postConstruct also registers failedRushForJumpAlwaysLandsInTargetSquare,
        // which was missing (only canLeap was present).
        assert!(SkillId::Leap.properties().contains(&"failedRushForJumpAlwaysLandsInTargetSquare"));
    }

    #[test]
    fn properties_no_hands_has_full_property_set() {
        // Bug: bb2020/NoHands.postConstruct registers 4 properties; only preventCatch was present.
        let props = SkillId::NoHands.properties();
        assert!(props.contains(&"preventHoldBall"));
        assert!(props.contains(&"preventRegularPassAction"));
        assert!(props.contains(&"preventRegularHandOverAction"));
    }

    #[test]
    fn properties_monstrous_mouth_cancels_strip_ball() {
        // Bug: SkillId::MonstrousMouth had no match arm at all, falling through to `_ => &[]`.
        assert!(SkillId::MonstrousMouth.properties().contains(&"cancelsForceOpponentToDropBallOnPushback"));
    }

    #[test]
    fn properties_fumblerooskie_has_can_drop_ball() {
        // Bug: SkillId::Fumblerooskie had no match arm at all, falling through to `_ => &[]`.
        assert!(SkillId::Fumblerooskie.properties().contains(&"canDropBall"));
    }

    #[test]
    fn properties_brawler_has_can_reroll_single_both_down() {
        // Bug: SkillId::Brawler had no match arm at all, falling through to `_ => &[]`.
        assert!(SkillId::Brawler.properties().contains(&"canRerollSingleBothDown"));
    }

    #[test]
    fn properties_cloud_burster_has_bb2025_property() {
        // Bug: only the bb2020 property was present; bb2025's differently-named
        // passesAreNotIntercepted was missing entirely.
        assert!(SkillId::CloudBurster.properties().contains(&"passesAreNotIntercepted"));
    }

    #[test]
    fn properties_defensive_cancels_tacklezone_assists() {
        // Bug: bb2020/bb2025 Defensive.postConstruct cancels assistsBlocksInTacklezones and
        // assistsFoulsInTacklezones, but the table only had cancelsCanAlwaysAssistFouls (bb2025-only).
        let props = SkillId::Defensive.properties();
        assert!(props.contains(&"cancelsAssistsBlocksInTacklezones"));
        assert!(props.contains(&"cancelsAssistsFoulsInTacklezones"));
    }

    #[test]
    fn properties_ball_and_chain_cancels_trickster() {
        assert!(SkillId::BallAndChain.properties().contains(&"cancelsCanMoveBeforeBeingBlocked"));
    }

    #[test]
    fn properties_ball_and_chain_has_full_bb2020_property_set() {
        // Bug: entry previously listed a nonexistent "blocksLikeChainsaw" property and was missing
        // most of the properties registered by Java bb2020/BallAndChain.postConstruct.
        let props = SkillId::BallAndChain.properties();
        assert!(!props.contains(&"blocksLikeChainsaw"));
        assert!(props.contains(&"ignoreBlockAssists"));
        assert!(props.contains(&"preventPickup"));
        assert!(props.contains(&"blocksDuringMove"));
        assert!(props.contains(&"ignoreTacklezonesWhenMoving"));
        assert!(props.contains(&"convertStunToKO"));
        assert!(props.contains(&"preventRegularBlockAction"));
        assert!(props.contains(&"preventRegularBlitzAction"));
        assert!(props.contains(&"cancelsPreventOpponentFollowingUp"));
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

    // ── Phase AJ bb2016 audit: previously-missing property-table entries ──────

    #[test]
    fn properties_nerves_of_steel_ignores_tacklezones() {
        let props = SkillId::NervesOfSteel.properties();
        assert!(props.contains(&"ignoreTacklezonesWhenPassing"));
        assert!(props.contains(&"ignoreTacklezonesWhenCatching"));
    }

    #[test]
    fn properties_grab_can_push_back_to_any_square() {
        let props = SkillId::Grab.properties();
        assert!(props.contains(&"canPushBackToAnySquare"));
        assert!(props.contains(&"cancelsCanChooseOwnPushedBackSquare"));
    }

    #[test]
    fn properties_side_step_cancels_grab() {
        assert!(SkillId::SideStep.properties().contains(&"cancelsCanPushBackToAnySquare"));
    }

    #[test]
    fn properties_swarming_can_sneak_extra_players() {
        assert!(SkillId::Swarming.properties().contains(&"canSneakExtraPlayersOntoPitch"));
    }

    #[test]
    fn properties_no_hands_has_four() {
        let props = SkillId::NoHands.properties();
        assert!(props.contains(&"preventCatch"));
        assert!(props.contains(&"preventHoldBall"));
        assert!(props.contains(&"preventRegularPassAction"));
        assert!(props.contains(&"preventRegularHandOverAction"));
    }

    #[test]
    fn properties_secret_weapon_prevents_stunty_dodge_modifier() {
        let props = SkillId::SecretWeapon.properties();
        assert!(props.contains(&"preventStuntyDodgeModifier"));
        assert!(props.contains(&"cancelsIgnoreTacklezonesWhenDodging"));
    }

    #[test]
    fn properties_swoop_prevents_stunty_dodge_modifier() {
        let props = SkillId::Swoop.properties();
        assert!(props.contains(&"preventStuntyDodgeModifier"));
        assert!(props.contains(&"cancelsIgnoreTacklezonesWhenDodging"));
    }

    #[test]
    fn properties_stab_has_multiple_block_alternative() {
        // Bug: bb2020/Stab.postConstruct registers 4 properties but the union was missing
        // providesMultipleBlockAlternative.
        assert!(SkillId::Stab.properties().contains(&"providesMultipleBlockAlternative"));
    }

    #[test]
    fn properties_swoop_has_full_bb2020_property_set() {
        // Bug: bb2020/Swoop.postConstruct registers 3 properties but only ttmScattersInSingleDirection
        // was present in the union.
        let props = SkillId::Swoop.properties();
        assert!(props.contains(&"preventStuntyDodgeModifier"));
        assert!(props.contains(&"ttmScattersInSingleDirection"));
        assert!(props.contains(&"cancelsIgnoreTacklezonesWhenDodging"));
    }

    #[test]
    fn properties_titchy_has_no_tacklezone_for_dodging() {
        assert!(SkillId::Titchy.properties().contains(&"hasNoTacklezoneForDodging"));
    }

    #[test]
    fn properties_stakes_provides_stab_block_alternative() {
        let props = SkillId::Stakes.properties();
        assert!(props.contains(&"providesStabBlockAlternative"));
        assert!(props.contains(&"canPerformArmourRollInsteadOfBlock"));
        assert!(props.contains(&"providesBlockAlternative"));
    }

    #[test]
    fn properties_kick_off_return_can_move_during_scatter() {
        assert!(SkillId::KickOffReturn.properties().contains(&"canMoveDuringKickOffScatter"));
    }

    #[test]
    fn properties_monstrous_mouth_cancels_drop_ball_on_pushback() {
        assert!(SkillId::MonstrousMouth.properties().contains(&"cancelsForceOpponentToDropBallOnPushback"));
    }

    #[test]
    fn properties_ball_and_chain_bb2016_union_is_complete() {
        let props = SkillId::BallAndChain.properties();
        for expected in [
            "forceFullMovement",
            "grabOutsideBlock",
            "placedProneCausesInjuryRoll",
            "flipSameTeamOpponentToOtherTeam",
            "preventAutoMove",
            "preventRegularBlitzAction",
            "preventRegularBlockAction",
            "preventRegularFoulAction",
            "preventRegularHandOverAction",
            "preventRegularPassAction",
            "preventRecoverFromConcusionAction",
            "preventRecoverFromGazeAction",
            "preventStandUpAction",
            "canBlockMoreThanOnce",
            "forceFollowup",
            "canBlockSameTeamPlayer",
            "preventThrowTeamMateAction",
            "preventKickTeamMateAction",
            "goForItAfterBlock",
            "movesRandomly",
            "blocksDuringMove",
            "ignoreTacklezonesWhenMoving",
            "convertStunToKO",
            "cancelsCanBlockMoreThanOnce",
            "cancelsCanPileOnOpponent",
            "cancelsForceRollBeforeBeingBlocked",
        ] {
            assert!(props.contains(&expected), "BallAndChain missing property {expected}");
        }
    }

    #[test]
    fn properties_sneaky_git_has_can_move_after_foul() {
        // Bug: bb2020/SneakyGit.postConstruct additionally registers canMoveAfterFoul (unlike
        // bb2016/SneakyGit and bb2025/PutTheBootIn, which only register canAlwaysAssistFouls).
        let props = SkillId::SneakyGit.properties();
        assert!(props.contains(&"canAlwaysAssistFouls"));
        assert!(props.contains(&"canMoveAfterFoul"));
    }

    #[test]
    fn properties_right_stuff_has_bb2020_conditional_throw_property() {
        // Bug: bb2020/RightStuff.postConstruct registers canBeThrownIfStrengthIs3orLess instead of
        // canBeThrown, but the union only had the bb2016/bb2025 property names.
        let props = SkillId::RightStuff.properties();
        assert!(props.contains(&"canBeThrownIfStrengthIs3orLess"));
        assert!(props.contains(&"ignoreTackleWhenBlocked"));
    }

    #[test]
    fn properties_projectile_vomit_registered() {
        // Bug: SkillId::ProjectileVomit had no entry at all in properties(), so it fell through
        // to the `_ => &[]` default despite Java registering 2 properties in postConstruct.
        let props = SkillId::ProjectileVomit.properties();
        assert!(props.contains(&"providesBlockAlternative"));
        assert!(props.contains(&"canPerformArmourRollInsteadOfBlockThatMightFail"));
    }

    #[test]
    fn properties_swarming_registered() {
        // Bug: SkillId::Swarming had no entry at all in properties(), so it fell through to
        // the `_ => &[]` default despite Java registering canSneakExtraPlayersOntoPitch.
        assert!(SkillId::Swarming.properties().contains(&"canSneakExtraPlayersOntoPitch"));
    }

    #[test]
    fn properties_hail_mary_pass_can_pass_to_any_square() {
        // Java HailMaryPass.postConstruct registers canPassToAnySquare, not canGainHailMary
        // (canGainHailMary belongs to the unrelated ShotToNothing skill).
        let props = SkillId::HailMaryPass.properties();
        assert!(props.contains(&"canPassToAnySquare"));
        assert!(!props.contains(&"canGainHailMary"));
    }

    #[test]
    fn properties_horns_add_strength_on_blitz() {
        assert!(SkillId::Horns.properties().contains(&"addStrengthOnBlitz"));
    }

    #[test]
    fn properties_diving_catch_has_both_properties() {
        let props = SkillId::DivingCatch.properties();
        assert!(props.contains(&"canAttemptCatchInAdjacentSquares"));
        assert!(props.contains(&"addBonusForAccuratePass"));
    }

    // ── mixed/special properties() gaps (bug fixes) ────────────────────────
    // Java postConstruct() registrations for these skills were never mirrored into this
    // table, silently breaking every live call site that checks `id.properties().contains(...)`
    // (e.g. GoredByTheBull's canAddBlockDie in step_init_moving.rs/step_end_moving.rs,
    // HalflingLuck's canRerollSingleDieOncePerPeriod in step_block_roll.rs).

    #[test]
    fn properties_all_you_can_eat() {
        assert!(SkillId::AllYouCanEat.properties().contains(&"canUseThrowBombActionTwice"));
    }

    #[test]
    fn properties_baleful_hex() {
        assert!(SkillId::BalefulHex.properties().contains(&"canMakeOpponentMissTurn"));
    }

    #[test]
    fn properties_black_ink() {
        assert!(SkillId::BlackInk.properties().contains(&"canGazeAutomatically"));
    }

    #[test]
    fn properties_bugmans_xxxxxx() {
        assert!(SkillId::BugmansXXXXXX.properties().contains(&"canReRollOnesOnKORecovery"));
    }

    #[test]
    fn properties_catch_of_the_day() {
        assert!(SkillId::CatchOfTheDay.properties().contains(&"canGetBallOnGround"));
    }

    #[test]
    fn properties_furious_outburst_has_both() {
        let props = SkillId::FuriousOutburst.properties();
        assert!(props.contains(&"canTeleportBeforeAndAfterAvRollAttack"));
        assert!(props.contains(&"canPerformArmourRollInsteadOfBlock"));
    }

    #[test]
    fn properties_fury_of_the_blood_god() {
        assert!(SkillId::FuryOfTheBloodGod.properties().contains(&"canPerformTwoBlocksAfterFailedFury"));
    }

    #[test]
    fn properties_gored_by_the_bull_has_both() {
        let props = SkillId::GoredByTheBull.properties();
        assert!(props.contains(&"canAddBlockDie"));
        assert!(props.contains(&"providesBlockAlternativeDuringBlitz"));
    }

    #[test]
    fn properties_halfling_luck() {
        assert!(SkillId::HalflingLuck.properties().contains(&"canRerollSingleDieOncePerPeriod"));
    }

    #[test]
    fn properties_ill_be_back() {
        assert!(SkillId::IllBeBack.properties().contains(&"ignoreFirstSecretWeaponSentOff"));
    }

    #[test]
    fn properties_keen_player_has_both() {
        let props = SkillId::KeenPlayer.properties();
        assert!(props.contains(&"canJoinTeamIfLessThanEleven"));
        assert!(props.contains(&"getsSentOffAtEndOfDrive"));
    }

    #[test]
    fn properties_kick_em_while_they_re_down() {
        assert!(SkillId::KickEmWhileTheyReDown.properties().contains(&"canUseChainsawOnDownedOpponents"));
    }

    #[test]
    fn properties_look_into_my_eyes() {
        assert!(SkillId::LookIntoMyEyes.properties().contains(&"canStealBallFromOpponent"));
    }

    #[test]
    fn properties_bounding_leap() {
        assert!(SkillId::BoundingLeap.properties().contains(&"canIgnoreJumpModifiers"));
    }

    /// BB2025/Regeneration.java registers only canRollToSaveFromInjury +
    /// CancelSkillProperty(allowsRaisingLineman) — NOT preventRaiseFromDead (bb2016/bb2020 only).
    /// A bb2025 Regeneration player (e.g. a Necromantic Flesh Golem) that dies must be raisable
    /// (necromantic seed 89). Stunty separately keeps preventRaiseFromDead in all editions.
    #[test]
    fn regeneration_does_not_prevent_raise_from_dead_bb2025() {
        let props = SkillId::Regeneration.properties();
        assert!(!props.contains(&"preventRaiseFromDead"),
            "bb2025 Regeneration must NOT prevent raise-from-dead");
        assert!(props.contains(&"canRollToSaveFromInjury"));
        assert!(props.contains(&"cancelsAllowsRaisingLineman"));
        // Stunty still carries it (all editions).
        assert!(SkillId::Stunty.properties().contains(&"preventRaiseFromDead"));
    }

    // ── reroll_sources(): exhaustive pin of the live table ────────────────────

    #[test]
    fn reroll_sources_exhaustive_table() {
        use std::collections::HashSet;

        // Every action string the table is allowed to use (Rust engine vocabulary:
        // Java PICK_UP → "PICKUP", GO_FOR_IT/RUSH → "GFI"; all others keep Java names).
        const ALLOWED_ACTIONS: &[&str] = &[
            "CATCH", "PASS", "PICKUP", "DODGE", "GFI", "DIRECTION", "HYPNOTIC_GAZE",
            "SINGLE_DIE", "THROW_TEAM_MATE", "KICK_TEAM_MATE", "SINGLE_BLOCK_DIE",
            "MULTI_BLOCK_DICE", "JUMP", "DAUNTLESS", "SINGLE_DIE_PER_ACTIVATION",
            "PUNT_DIRECTION", "PUNT_DISTANCE", "SINGLE_SKULL", "SINGLE_BOTH_DOWN",
            "RIGHT_STUFF",
        ];

        // Exactly these 25 skills call registerRerollSource somewhere in Java
        // (union across bb2016/bb2020/bb2025, see per-arm citations in reroll_sources()).
        let expected_non_empty: HashSet<SkillId> = [
            SkillId::Catch,
            SkillId::Pass,
            SkillId::SureHands,
            SkillId::Dodge,
            SkillId::SureFeet,
            SkillId::MonstrousMouth,
            SkillId::WhirlingDervish,
            SkillId::MesmerizingDance,
            SkillId::MesmerisingDance,
            SkillId::ConsummateProfessional,
            SkillId::TheBallista,
            SkillId::UnstoppableMomentum,
            SkillId::ThinkingMansTroll,
            SkillId::SavageBlow,
            SkillId::HalflingLuck,
            SkillId::BoundingLeap,
            SkillId::BlindRage,
            SkillId::Pro,
            SkillId::Kick,
            SkillId::Hatred,
            SkillId::Brawler,
            SkillId::Swoop,
            SkillId::WorkingInTandem,
            SkillId::WoodlandFury,
            SkillId::LordOfChaos,
        ]
        .into_iter()
        .collect();

        // SkillFactory's map is built from the declaration-order ALL_SKILL_IDS list,
        // giving us every SkillId variant (deduped via HashSet — the map holds
        // BallAndChain under two extra alias keys).
        let all_ids: HashSet<SkillId> =
            crate::factory::skill_factory::SkillFactory::new().get_skills().collect();
        assert!(all_ids.len() >= 190, "factory should enumerate every SkillId variant");

        let mut non_empty: HashSet<SkillId> = HashSet::new();
        let mut total_pairs = 0usize;
        for &id in &all_ids {
            let sources = id.reroll_sources();
            if !sources.is_empty() {
                non_empty.insert(id);
            }
            total_pairs += sources.len();
            for (action, priority) in sources {
                assert!(
                    ALLOWED_ACTIONS.contains(action),
                    "unexpected reroll action {action:?} for {id:?}"
                );
                // Java ReRollSources: every source is priority 1 except THE_BALLISTA (2).
                let expected_priority = if id == SkillId::TheBallista { 2 } else { 1 };
                assert_eq!(
                    *priority, expected_priority,
                    "wrong priority for {id:?} action {action}"
                );
            }
        }

        assert_eq!(non_empty, expected_non_empty, "set of skills with reroll sources changed");
        // 23 single-pair skills + Kick (2) + TheBallista (3) = 28 pairs.
        assert_eq!(total_pairs, 28, "total (action, priority) pair count changed");
        assert_eq!(SkillId::TheBallista.reroll_sources().len(), 3);
        assert_eq!(SkillId::Kick.reroll_sources().len(), 2);
    }
}
