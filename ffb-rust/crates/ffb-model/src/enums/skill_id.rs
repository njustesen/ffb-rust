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
            SkillId::DivingTackle => &["canAttemptToTackleDodgingPlayer"],
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
            SkillId::MonstrousMouth => &["cancelsForceOpponentToDropBallOnPushback"],
            // Java: SafeThrow.postConstruct registers NamedProperties.canCancelInterceptions
            SkillId::SafeThrow => &["canCancelInterceptions"],
            // Java: VeryLongLegs.postConstruct registers CancelSkillProperty(canCancelInterceptions) (BB2016)
            //   and CancelSkillProperty(canForceInterceptionRerollOfLongPasses) (BB2020) — union of both.
            SkillId::VeryLongLegs => &["cancelsCancelInterceptions", "cancelsCanForceInterceptionRerollOfLongPasses"],
            // Java: CloudBurster (BB2020) registers canForceInterceptionRerollOfLongPasses;
            // bb2025's CloudBurster registers the differently-named passesAreNotIntercepted instead — union of both.
            SkillId::CloudBurster => &["canForceInterceptionRerollOfLongPasses", "passesAreNotIntercepted"],
            // Java: FuriousOutburst.postConstruct registers canTeleportBeforeAndAfterAvRollAttack
            SkillId::FuriousOutburst => &["canTeleportBeforeAndAfterAvRollAttack"],
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
            //   and CancelSkillProperty(ignoreTacklezonesWhenDodging).
            SkillId::SecretWeapon => &["preventStuntyDodgeModifier", "getsSentOffAtEndOfDrive", "cancelsIgnoreTacklezonesWhenDodging"],
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
            // Java: bb2020/Swarming.postConstruct registers canSneakExtraPlayersOntoPitch
            SkillId::Swarming => &["canSneakExtraPlayersOntoPitch"],
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
}
