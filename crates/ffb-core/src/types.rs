use serde::{Deserialize, Serialize};

// ── Field coordinates ────────────────────────────────────────────────────────

/// A position on the 26×17 Blood Bowl pitch (x ∈ 0..=25, y ∈ 0..=16).
/// `NONE` (255, 255) is used as an out-of-bounds sentinel.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct FieldCoordinate {
    pub x: u8,
    pub y: u8,
}

pub const PITCH_WIDTH: u8 = 26;
pub const PITCH_HEIGHT: u8 = 17;
pub const PITCH_SQUARES: usize = PITCH_WIDTH as usize * PITCH_HEIGHT as usize;

impl FieldCoordinate {
    pub const NONE: Self = Self { x: 255, y: 255 };

    #[inline]
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.x < PITCH_WIDTH && self.y < PITCH_HEIGHT
    }

    /// Flat index into a [T; PITCH_SQUARES] array.
    #[inline]
    pub fn index(self) -> usize {
        self.y as usize * PITCH_WIDTH as usize + self.x as usize
    }

    /// All valid 8-directional neighbors.
    pub fn neighbors(self) -> impl Iterator<Item = FieldCoordinate> {
        let x = self.x as i16;
        let y = self.y as i16;
        [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),           (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ]
        .into_iter()
        .filter_map(move |(dx, dy)| {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < PITCH_WIDTH as i16 && ny >= 0 && ny < PITCH_HEIGHT as i16 {
                Some(FieldCoordinate::new(nx as u8, ny as u8))
            } else {
                None
            }
        })
    }

    /// Manhattan distance (not used for movement, but handy for eval).
    pub fn manhattan(self, other: FieldCoordinate) -> u8 {
        (self.x as i16 - other.x as i16).unsigned_abs() as u8
            + (self.y as i16 - other.y as i16).unsigned_abs() as u8
    }

    /// True if the coordinate is off the pitch (for crowd pushback detection).
    pub fn is_off_pitch(self) -> bool {
        !self.is_valid()
    }
}

// ── Player state ─────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PlayerState {
    Reserve,
    Standing,
    Moving,
    Prone,
    Stunned,
    Ko,
    Injured,
    /// Player is held in place (TakeRoot, Tentacles, etc.)
    Rooted,
}

impl PlayerState {
    pub fn is_on_pitch(self) -> bool {
        matches!(self, Self::Standing | Self::Moving | Self::Prone | Self::Stunned | Self::Rooted)
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Standing | Self::Moving | Self::Rooted)
    }

    pub fn can_be_blocked(self) -> bool {
        self.is_on_pitch()
    }

    pub fn is_prone_or_stunned(self) -> bool {
        matches!(self, Self::Prone | Self::Stunned)
    }
}

// ── Turn modes (31 variants — mirrors Java TurnMode enum) ───────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum TurnMode {
    Regular,
    Setup,
    Kickoff,
    StartGame,
    EndGame,
    PerfectDefence,
    SolidDefence,
    QuickSnap,
    HighKick,
    Blitz,
    PassBlock,
    DumpOff,
    Touchback,
    Interception,
    Swarming,
    KickoffReturn,
    Wizard,
    BombHome,
    BombAway,
    BombHomeBlitz,
    BombAwayBlitz,
    SelectBlitzTarget,
    SelectGazeTarget,
    SelectBlockKind,
    IllegalSubstitution,
    SafePairOfHands,
    BetweenTurns,
    Trickster,
    RaidingParty,
    HitAndRun,
    ThenIStartedBlastin,
}

impl TurnMode {
    pub fn check_for_active_players(self) -> bool {
        matches!(self, Self::Regular | Self::Blitz | Self::BetweenTurns)
    }

    pub fn is_bomb_turn(self) -> bool {
        matches!(
            self,
            Self::BombHome | Self::BombAway | Self::BombHomeBlitz | Self::BombAwayBlitz
        )
    }

    pub fn allow_end_player_action(self) -> bool {
        !matches!(self, Self::BombHome | Self::BombAway | Self::BombHomeBlitz | Self::BombAwayBlitz | Self::DumpOff)
    }
}

// ── Block outcomes ───────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BlockResult {
    /// Attacker knocked down
    Skull,
    /// Both knocked down
    BothDown,
    /// Defender pushed back, no knockdown
    Pushback,
    /// Defender pushed back AND knocked down
    PowPushback,
    /// Defender knocked down, no push
    Pow,
}

impl BlockResult {
    pub fn from_d6(roll: u8) -> Self {
        match roll {
            1 => Self::Skull,
            2 => Self::BothDown,
            3 | 4 => Self::Pushback,
            5 => Self::PowPushback,
            _ => Self::Pow,
        }
    }

    pub fn knocks_down_defender(self) -> bool {
        matches!(self, Self::PowPushback | Self::Pow)
    }

    pub fn knocks_down_attacker(self) -> bool {
        matches!(self, Self::Skull | Self::BothDown)
    }

    pub fn causes_push(self) -> bool {
        matches!(self, Self::Pushback | Self::PowPushback)
    }
}

// ── Player actions ───────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Move,
    Block,
    Blitz,
    Pass,
    HandOff,
    Foul,
    Unused,
}

// ── Half ─────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum Half {
    #[default]
    First,
    Second,
}

// ── Weather ──────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum Weather {
    #[default]
    NiceDayForAFootballGame,
    VerySunny,
    PouringRain,
    BlowingAGale,
    SwelteringHeat,
    Blizzard,
}

impl Weather {
    /// Some kickoff events use 2d6; this maps the sum to weather.
    /// BB2025 table: 2=Sweltering Heat, 3=Very Sunny, 4-10=Nice, 11=Pouring Rain, 12=Blizzard
    pub fn from_kickoff_roll(roll: u8) -> Self {
        match roll {
            2 => Self::SwelteringHeat,
            3 => Self::VerySunny,
            4..=10 => Self::NiceDayForAFootballGame,
            11 => Self::PouringRain,
            _ => Self::Blizzard,
        }
    }

    pub fn affects_passing(self) -> bool {
        matches!(self, Self::PouringRain | Self::BlowingAGale | Self::Blizzard)
    }
}

// ── Pass range ───────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PassRange {
    HandOff,
    /// ≤3 squares
    Short,
    /// 4–6 squares
    Long,
    /// 7–10 squares
    LongBomb,
}

impl PassRange {
    pub fn for_distance(dist: u8) -> Self {
        match dist {
            0 | 1 => Self::HandOff,
            2..=3 => Self::Short,
            4..=6 => Self::Long,
            _ => Self::LongBomb,
        }
    }
}

// ── Injury outcomes ──────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum InjuryOutcome {
    Stunned,
    KnockedOut,
    Casualty,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum CasualtyType {
    BadlyHurt,
    BrokenRibs,
    GashingWound,
    TwistedAnkle,
    HamstringPull,
    GroinStrain,
    PinchedNerve,
    DamagedBack,
    SmashedKnee,
    SmashedHip,
    SmashedCollarBone,
    BrokenNeck,
    SmashedShoulder,
    BrokenArm,
    SmashedHand,
    LostEye,
    SmashedKneeAgain,
    LostEar,
    BrokenNose,
    MissingTeeth,
    FracturedSkull,
    SeriesFracturedSkull,
    Squashed,
    Dead,
}

impl CasualtyType {
    pub fn from_d16(roll: u8) -> Self {
        match roll {
            1..=6 => Self::BadlyHurt,
            7 => Self::BrokenRibs,
            8 => Self::GashingWound,
            9 => Self::TwistedAnkle,
            10 => Self::HamstringPull,
            11 => Self::GroinStrain,
            12 => Self::PinchedNerve,
            13 => Self::DamagedBack,
            14 => Self::SmashedKnee,
            15 => Self::SmashedHip,
            16 => Self::SmashedCollarBone,
            17 => Self::BrokenNeck,
            18 => Self::SmashedShoulder,
            19 => Self::BrokenArm,
            20 => Self::SmashedHand,
            21 => Self::LostEye,
            22 => Self::LostEar,
            23 => Self::BrokenNose,
            24 => Self::MissingTeeth,
            25 => Self::FracturedSkull,
            26 => Self::SeriesFracturedSkull,
            27..=30 => Self::Squashed,
            _ => Self::Dead,
        }
    }

    pub fn is_miss_next_game(self) -> bool {
        !matches!(self, Self::BadlyHurt)
    }

    /// Severity rank: higher number = worse outcome.
    pub fn severity(self) -> u8 {
        match self {
            Self::BadlyHurt => 0,
            Self::BrokenRibs | Self::GashingWound | Self::TwistedAnkle
            | Self::HamstringPull | Self::GroinStrain | Self::PinchedNerve
            | Self::DamagedBack | Self::SmashedKnee | Self::SmashedHip
            | Self::SmashedCollarBone | Self::BrokenNeck | Self::SmashedShoulder
            | Self::BrokenArm | Self::SmashedHand | Self::LostEye
            | Self::SmashedKneeAgain | Self::LostEar | Self::BrokenNose
            | Self::MissingTeeth | Self::FracturedSkull | Self::SeriesFracturedSkull => 1,
            Self::Squashed => 2,
            Self::Dead => 3,
        }
    }

    /// Returns true if `self` is a worse outcome than `other`.
    pub fn is_worse_than(self, other: Self) -> bool {
        self.severity() > other.severity()
    }
}

// ── Serious injuries (lasting) ───────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SeriousInjury {
    NigglingInjury,
    MinusOneMovement,
    MinusOneStrength,
    MinusOneAgility,
    MinusOnePassing,
    MinusOneArmour,
    Dead,
}

// ── Kickoff events ───────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum KickoffEvent {
    GetTheRef,
    Riot,
    PerfectDefence,
    HighKick,
    CheeringFans,
    ChangingWeather,
    BrilliantCoaching,
    QuickSnap,
    Blitz,
    ThrowARock,
    PitchInvasion,
    SwelteringHeat,
}

impl KickoffEvent {
    pub fn from_2d6(roll: u8) -> Self {
        // BB2025 kickoff event table (matches Java KickoffResultMapping.java)
        match roll {
            2 => Self::GetTheRef,
            3 => Self::Riot,         // TIME_OUT in BB2025
            4 => Self::PerfectDefence, // SOLID_DEFENCE in BB2025 (rolls d3 for player count)
            5 => Self::HighKick,
            6 => Self::CheeringFans,
            7 => Self::BrilliantCoaching, // BB2025: BRILLIANT_COACHING at 7
            8 => Self::ChangingWeather,   // BB2025: WEATHER_CHANGE at 8
            9 => Self::QuickSnap,
            10 => Self::Blitz,       // CHARGE in BB2025
            11 => Self::ThrowARock,  // DODGY_SNACK in BB2025
            _ => Self::PitchInvasion,
        }
    }
}

// ── Player ID ─────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PlayerId(pub String);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Team ID ──────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum TeamId {
    #[default]
    Home,
    Away,
}

impl TeamId {
    pub fn opponent(self) -> Self {
        match self {
            Self::Home => Self::Away,
            Self::Away => Self::Home,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Away => "away",
        }
    }
}

// ── Inducement state ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct InducementState {
    pub wizard_used: bool,
    pub bribes_remaining: u8,
}

// ── Player gender ─────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum PlayerGender {
    #[default]
    Male,
    Female,
    Other,
}

// ── Team special rules ────────────────────────────────────────────────────────

/// Team-level special rules that affect in-game mechanics.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SpecialRule {
    /// After each opponent casualty, the Necromantic/Undead team may raise a Zombie.
    MastersOfUndeath,
    /// Team may field more than 11 players (up to count of own players off-pitch).
    Swarming,
    /// Halfling team receives a free Master Chef inducement.
    HalflingThimbleCup,
    /// Team consists of mixed race players.
    UnderworldChallenge,
    /// Chaos Undivided blessing: re-roll one block die per match.
    FavouredOfChaosUndivided,
    /// Khorne blessing: +1 to injury rolls.
    FavouredOfKhorne,
    /// Nurgle blessing: -1 to opponent's AG rolls.
    FavouredOfNurgle,
    /// Slaanesh blessing: once per match, force a re-roll.
    FavouredOfSlaanesh,
    /// Tzeentch blessing: once per match, change a die result.
    FavouredOfTzeentch,
    /// Elven kingdoms league eligibility.
    ElvenKingdomsLeague,
    /// Lustrian superleague eligibility.
    LustrianSuperleague,
    /// Old world classic eligibility.
    OldWorldClassic,
    /// Underworld denizens eligibility.
    UnderworldDenizens,
    /// Favoured of chaos (generic, covers Chaos team ruleset).
    FavouredOfChaos,
    /// Sylvanian spotlight eligibility.
    SylvanianSpotlight,
    /// Specialists available to this race.
    SpecialistAvailable,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_coordinate_neighbors_count() {
        // Corner squares have 3 neighbors
        let corner = FieldCoordinate::new(0, 0);
        assert_eq!(corner.neighbors().count(), 3);

        // Edge squares (non-corner) have 5 neighbors
        let edge = FieldCoordinate::new(5, 0);
        assert_eq!(edge.neighbors().count(), 5);

        // Interior squares have 8 neighbors
        let interior = FieldCoordinate::new(5, 5);
        assert_eq!(interior.neighbors().count(), 8);
    }

    #[test]
    fn field_coordinate_neighbors_all_valid() {
        for x in 0..PITCH_WIDTH {
            for y in 0..PITCH_HEIGHT {
                let coord = FieldCoordinate::new(x, y);
                for n in coord.neighbors() {
                    assert!(n.is_valid(), "neighbor {n:?} of {coord:?} out of bounds");
                }
            }
        }
    }

    #[test]
    fn field_coordinate_index_unique() {
        let mut seen = std::collections::HashSet::new();
        for x in 0..PITCH_WIDTH {
            for y in 0..PITCH_HEIGHT {
                let idx = FieldCoordinate::new(x, y).index();
                assert!(seen.insert(idx), "duplicate index {idx} for ({x},{y})");
            }
        }
        assert_eq!(seen.len(), PITCH_SQUARES);
    }

    #[test]
    fn block_result_from_d6_all_covered() {
        for roll in 1..=6 {
            let _ = BlockResult::from_d6(roll);
        }
        assert!(BlockResult::Skull.knocks_down_attacker());
        assert!(BlockResult::BothDown.knocks_down_attacker());
        assert!(!BlockResult::Pushback.knocks_down_attacker());
        assert!(BlockResult::Pow.knocks_down_defender());
        assert!(!BlockResult::Skull.knocks_down_defender());
    }

    #[test]
    fn kickoff_event_all_sums_covered() {
        for sum in 2u8..=12 {
            let _ = KickoffEvent::from_2d6(sum);
        }
    }

    #[test]
    fn weather_from_kickoff() {
        assert_eq!(Weather::from_kickoff_roll(2), Weather::SwelteringHeat);
        assert_eq!(Weather::from_kickoff_roll(3), Weather::VerySunny);
        assert_eq!(Weather::from_kickoff_roll(7), Weather::NiceDayForAFootballGame);
        assert_eq!(Weather::from_kickoff_roll(11), Weather::PouringRain);
        assert_eq!(Weather::from_kickoff_roll(12), Weather::Blizzard);
    }

    #[test]
    fn team_id_opponent() {
        assert_eq!(TeamId::Home.opponent(), TeamId::Away);
        assert_eq!(TeamId::Away.opponent(), TeamId::Home);
    }
}
