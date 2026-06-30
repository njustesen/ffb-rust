use serde::{Deserialize, Serialize};

// ─── PlayerState ─────────────────────────────────────────────────────────────
//
// Java PlayerState is an integer bitmask: the low byte encodes the base state,
// the upper bits encode flags (active, confused, rooted, hypnotized, …).
// We replicate the full bit pattern as a newtype so parity tests can compare
// raw integers. Helpers expose the same predicates as the Java class.

pub const PS_UNKNOWN: u32 = 0x00000;
pub const PS_STANDING: u32 = 0x00001;
pub const PS_MOVING: u32 = 0x00002;
pub const PS_PRONE: u32 = 0x00003;
pub const PS_STUNNED: u32 = 0x00004;
pub const PS_KNOCKED_OUT: u32 = 0x00005;
pub const PS_BADLY_HURT: u32 = 0x00006;
pub const PS_SERIOUS_INJURY: u32 = 0x00007;
pub const PS_RIP: u32 = 0x00008;
pub const PS_RESERVE: u32 = 0x00009;
pub const PS_MISSING: u32 = 0x0000a;
pub const PS_FALLING: u32 = 0x0000b;
pub const PS_BLOCKED: u32 = 0x0000c;
pub const PS_BANNED: u32 = 0x0000d;
pub const PS_EXHAUSTED: u32 = 0x0000e;
pub const PS_BEING_DRAGGED: u32 = 0x0000f;
pub const PS_PICKED_UP: u32 = 0x00010;
pub const PS_HIT_ON_GROUND: u32 = 0x00011;
pub const PS_HIT_BY_FIREBALL: u32 = 0x00011;
pub const PS_HIT_BY_LIGHTNING: u32 = 0x00012;
pub const PS_HIT_BY_BOMB: u32 = 0x00013;
pub const PS_SETUP_PREVENTED: u32 = 0x00014;
pub const PS_IN_THE_AIR: u32 = 0x00015;

const BIT_ACTIVE: u32 = 0x00100;
const BIT_CONFUSED: u32 = 0x00200;
const BIT_ROOTED: u32 = 0x00400;
const BIT_HYPNOTIZED: u32 = 0x00800;
const BIT_SELECTED_STAB_TARGET: u32 = 0x01000;
const BIT_USED_PRO: u32 = 0x02000;
const BIT_SELECTED_BLITZ_TARGET: u32 = 0x04000;
const BIT_SELECTED_BLOCK_TARGET: u32 = 0x08000;
const BIT_SELECTED_GAZE_TARGET: u32 = 0x10000;
const BIT_EYE_GOUGED: u32 = 0x20000;
const BIT_CHOMPED: u32 = 0x40000;

static BASE_MASK: &[u32] = &[
    0x00000, // UNKNOWN
    0xfff00, // STANDING
    0xfff00, // MOVING
    0xfff00, // PRONE
    0xfff00, // STUNNED
    0x00000, // KNOCKED_OUT
    0x00000, // BADLY_HURT
    0x00000, // SERIOUS_INJURY
    0x00000, // RIP
    0x00000, // RESERVE
    0x00000, // MISSING
    0xfff00, // FALLING
    0xfff00, // BLOCKED
    0x00000, // BANNED
    0xfff00, // EXHAUSTED
    0xfff00, // BEING_DRAGGED
    0xfff00, // PICKED_UP
    0xfff00, // HIT_ON_GROUND
    0xfff00, // SETUP_PREVENTED
    0xfff00, // IN_THE_AIR
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerState(pub u32);

impl PlayerState {
    pub fn new(id: u32) -> Self {
        PlayerState(id)
    }

    pub fn id(self) -> u32 {
        self.0
    }

    pub fn base(self) -> u32 {
        self.0 & 0x000ff
    }

    pub fn change_base(self, base: u32) -> PlayerState {
        let mask = if base > 0 && (base as usize) < BASE_MASK.len() {
            BASE_MASK[base as usize]
        } else {
            0
        };
        PlayerState((self.0 & mask) | base)
    }

    fn has_bit(self, mask: u32) -> bool {
        (self.0 & mask) > 0
    }

    fn change_bit(self, mask: u32, set: bool) -> PlayerState {
        if set {
            PlayerState(self.0 | mask)
        } else {
            PlayerState(self.0 & (0xfffff ^ mask))
        }
    }

    pub fn is_active(self) -> bool { self.has_bit(BIT_ACTIVE) }
    pub fn change_active(self, v: bool) -> PlayerState { self.change_bit(BIT_ACTIVE, v) }

    pub fn is_confused(self) -> bool { self.has_bit(BIT_CONFUSED) }
    pub fn change_confused(self, v: bool) -> PlayerState { self.change_bit(BIT_CONFUSED, v) }

    pub fn is_rooted(self) -> bool { self.has_bit(BIT_ROOTED) }
    pub fn change_rooted(self, v: bool) -> PlayerState { self.change_bit(BIT_ROOTED, v) }

    pub fn is_hypnotized(self) -> bool { self.has_bit(BIT_HYPNOTIZED) }
    pub fn change_hypnotized(self, v: bool) -> PlayerState { self.change_bit(BIT_HYPNOTIZED, v) }

    /// 1:1 translation of PlayerState.recoverTacklezones().
    /// Clears hypnotized and confused flags only.
    pub fn recover_tacklezones(self) -> PlayerState {
        self.change_hypnotized(false).change_confused(false)
    }

    pub fn is_eye_gouged(self) -> bool { self.has_bit(BIT_EYE_GOUGED) }
    pub fn change_eye_gouged(self, v: bool) -> PlayerState { self.change_bit(BIT_EYE_GOUGED, v) }
    /// 1:1 translation of PlayerState.clearEyeGouge().
    pub fn clear_eye_gouge(self) -> PlayerState { self.change_eye_gouged(false) }

    pub fn is_chomped(self) -> bool { self.has_bit(BIT_CHOMPED) }
    pub fn change_chomped(self, v: bool) -> PlayerState { self.change_bit(BIT_CHOMPED, v) }

    pub fn is_selected_stab_target(self) -> bool { self.has_bit(BIT_SELECTED_STAB_TARGET) }
    pub fn change_selected_stab_target(self, v: bool) -> PlayerState {
        self.change_bit(BIT_SELECTED_STAB_TARGET, v)
    }

    pub fn is_selected_block_target(self) -> bool { self.has_bit(BIT_SELECTED_BLOCK_TARGET) }
    pub fn change_selected_block_target(self, v: bool) -> PlayerState {
        self.change_bit(BIT_SELECTED_BLOCK_TARGET, v)
    }

    pub fn is_selected_blitz_target(self) -> bool { self.has_bit(BIT_SELECTED_BLITZ_TARGET) }
    pub fn add_selected_blitz_target(self) -> PlayerState {
        self.change_bit(BIT_SELECTED_BLITZ_TARGET, true)
    }
    pub fn remove_selected_blitz_target(self) -> PlayerState {
        self.change_bit(BIT_SELECTED_BLITZ_TARGET, false)
    }

    pub fn is_selected_gaze_target(self) -> bool { self.has_bit(BIT_SELECTED_GAZE_TARGET) }
    pub fn change_selected_gaze_target(self, v: bool) -> PlayerState {
        self.change_bit(BIT_SELECTED_GAZE_TARGET, v)
    }

    pub fn remove_all_target_selections(self) -> PlayerState {
        self.change_selected_gaze_target(false).remove_selected_blitz_target()
    }

    pub fn has_used_pro(self) -> bool { self.has_bit(BIT_USED_PRO) }
    pub fn change_used_pro(self, v: bool) -> PlayerState { self.change_bit(BIT_USED_PRO, v) }

    pub fn is_casualty(self) -> bool {
        let b = self.base();
        b == PS_BADLY_HURT || b == PS_SERIOUS_INJURY || self.is_killed()
    }

    pub fn is_killed(self) -> bool { self.base() == PS_RIP }

    pub fn is_si(self) -> bool { self.base() == PS_SERIOUS_INJURY }

    pub fn can_be_set_up_next_drive(self) -> bool {
        let b = self.base();
        matches!(
            b,
            PS_STANDING | PS_MOVING | PS_PRONE | PS_STUNNED | PS_RESERVE | PS_FALLING
                | PS_HIT_ON_GROUND | PS_BLOCKED
        )
    }

    pub fn can_be_moved_during_setup(self) -> bool {
        let b = self.base();
        b == PS_STANDING || b == PS_RESERVE
    }

    pub fn has_tacklezones(self) -> bool {
        let b = self.base();
        (b == PS_STANDING || b == PS_MOVING || b == PS_BLOCKED)
            && !self.is_confused()
            && !self.is_hypnotized()
            && !self.is_eye_gouged()
    }

    pub fn is_prone_or_stunned(self) -> bool {
        let b = self.base();
        b == PS_PRONE || b == PS_STUNNED
    }

    pub fn is_prone(self) -> bool { self.base() == PS_PRONE }

    pub fn is_ko(self) -> bool { self.base() == PS_KNOCKED_OUT }

    pub fn is_stunned(self) -> bool { self.base() == PS_STUNNED }

    pub fn is_able_to_move(self) -> bool {
        let b = self.base();
        (b == PS_STANDING || b == PS_MOVING || b == PS_PRONE)
            && self.is_active()
            && !self.is_pinned()
    }

    pub fn can_be_blocked(self) -> bool {
        let b = self.base();
        b == PS_STANDING || b == PS_MOVING
    }

    pub fn can_be_fouled(self) -> bool {
        let b = self.base();
        b == PS_PRONE || b == PS_STUNNED
    }

    pub fn is_standing(self) -> bool {
        let b = self.base();
        b == PS_STANDING || b == PS_MOVING || b == PS_BLOCKED
    }

    pub fn is_distracted(self) -> bool {
        self.is_confused() || self.is_hypnotized()
    }

    pub fn is_carried(self) -> bool {
        let b = self.base();
        b == PS_PICKED_UP || b == PS_IN_THE_AIR
    }

    pub fn is_pinned(self) -> bool {
        self.is_chomped() || self.is_rooted()
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState(PS_UNKNOWN)
    }
}

impl std::fmt::Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── PlayerAction ─────────────────────────────────────────────────────────────

/// What action a player is currently performing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerAction {
    Move,
    Block,
    Blitz,
    BlitzMove,
    BlitzSelect,
    HandOver,
    HandOverMove,
    Pass,
    PassMove,
    Foul,
    FoulMove,
    StandUp,
    ThrowTeamMate,
    ThrowTeamMateMove,
    RemoveConfusion,
    Gaze,
    GazeSelect,
    GazeMove,
    MultipleBlock,
    HailMaryPass,
    DumpOff,
    StandUpBlitz,
    ThrowBomb,
    HailMaryBomb,
    Swoop,
    KickTeamMateMove,
    KickTeamMate,
    Treacherous,
    WisdomOfTheWhiteDwarf,
    ThrowKeg,
    RaidingParty,
    MaximumCarnage,
    LookIntoMyEyes,
    BalefulHex,
    AllYouCanEat,
    PutridRegurgitationMove,
    PutridRegurgitationBlitz,
    PutridRegurgitationBlock,
    KickEmBlock,
    KickEmBlitz,
    BlackInk,
    CatchOfTheDay,
    ThenIStartedBlastin,
    TheFlashingBlade,
    ViciousVines,
    FuriousOutburst,
    SecureTheBall,
    BreatheFire,
    Chainsaw,
    Stab,
    ProjectileVomit,
    AutoGazeZoat,
    Forgo,
    Incorporeal,
    Chomp,
    Punt,
    PuntMove,
}

impl PlayerAction {
    pub fn name(self) -> &'static str {
        match self {
            PlayerAction::Move => "move",
            PlayerAction::Block => "block",
            PlayerAction::Blitz => "blitz",
            PlayerAction::BlitzMove => "blitzMove",
            PlayerAction::BlitzSelect => "blitzSelect",
            PlayerAction::HandOver => "handOver",
            PlayerAction::HandOverMove => "handOverMove",
            PlayerAction::Pass => "pass",
            PlayerAction::PassMove => "passMove",
            PlayerAction::Foul => "foul",
            PlayerAction::FoulMove => "foulMove",
            PlayerAction::StandUp => "standUp",
            PlayerAction::ThrowTeamMate => "throwTeamMate",
            PlayerAction::ThrowTeamMateMove => "throwTeamMateMove",
            PlayerAction::RemoveConfusion => "removeConfusion",
            PlayerAction::Gaze => "gaze",
            PlayerAction::GazeSelect => "gazeSelect",
            PlayerAction::GazeMove => "gazeMove",
            PlayerAction::MultipleBlock => "multipleBlock",
            PlayerAction::HailMaryPass => "hailMaryPass",
            PlayerAction::DumpOff => "dumpOff",
            PlayerAction::StandUpBlitz => "standUpBlitz",
            PlayerAction::ThrowBomb => "throwBomb",
            PlayerAction::HailMaryBomb => "hailMaryBomb",
            PlayerAction::Swoop => "swoop",
            PlayerAction::KickTeamMateMove => "kickTeamMateMove",
            PlayerAction::KickTeamMate => "kickTeamMate",
            PlayerAction::Treacherous => "treacherous",
            PlayerAction::WisdomOfTheWhiteDwarf => "wisdomOfTheWhiteDwarf",
            PlayerAction::ThrowKeg => "throwKey",
            PlayerAction::RaidingParty => "raidingParty",
            PlayerAction::MaximumCarnage => "maximumCarnage",
            PlayerAction::LookIntoMyEyes => "lookIntoMyEyes",
            PlayerAction::BalefulHex => "balefulHex",
            PlayerAction::AllYouCanEat => "allYouCanEat",
            PlayerAction::PutridRegurgitationMove => "putridRegurgitationMove",
            PlayerAction::PutridRegurgitationBlitz => "putridRegurgitationBlitz",
            PlayerAction::PutridRegurgitationBlock => "putridRegurgitationBlock",
            PlayerAction::KickEmBlock => "kickEmBlock",
            PlayerAction::KickEmBlitz => "kickEmBlitz",
            PlayerAction::BlackInk => "blackInk",
            PlayerAction::CatchOfTheDay => "catchOfTheDay",
            PlayerAction::ThenIStartedBlastin => "thenIStartedBlastin",
            PlayerAction::TheFlashingBlade => "theFlashingBlade",
            PlayerAction::ViciousVines => "viciousVines",
            PlayerAction::FuriousOutburst => "furiousOutburst",
            PlayerAction::SecureTheBall => "secureTheBall",
            PlayerAction::BreatheFire => "breatheFire",
            PlayerAction::Chainsaw => "chainsaw",
            PlayerAction::Stab => "stab",
            PlayerAction::ProjectileVomit => "projectileVomit",
            PlayerAction::AutoGazeZoat => "autoGazeZoat",
            PlayerAction::Forgo => "forgo",
            PlayerAction::Incorporeal => "incorporeal",
            PlayerAction::Chomp => "chomp",
            PlayerAction::Punt => "punt",
            PlayerAction::PuntMove => "puntMove",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn all() -> &'static [PlayerAction] {
        &[
            PlayerAction::Move, PlayerAction::Block, PlayerAction::Blitz,
            PlayerAction::BlitzMove, PlayerAction::BlitzSelect, PlayerAction::HandOver,
            PlayerAction::HandOverMove, PlayerAction::Pass, PlayerAction::PassMove,
            PlayerAction::Foul, PlayerAction::FoulMove, PlayerAction::StandUp,
            PlayerAction::ThrowTeamMate, PlayerAction::ThrowTeamMateMove,
            PlayerAction::RemoveConfusion, PlayerAction::Gaze, PlayerAction::GazeSelect,
            PlayerAction::GazeMove, PlayerAction::MultipleBlock, PlayerAction::HailMaryPass,
            PlayerAction::DumpOff, PlayerAction::StandUpBlitz, PlayerAction::ThrowBomb,
            PlayerAction::HailMaryBomb, PlayerAction::Swoop, PlayerAction::KickTeamMateMove,
            PlayerAction::KickTeamMate, PlayerAction::Treacherous,
            PlayerAction::WisdomOfTheWhiteDwarf, PlayerAction::ThrowKeg,
            PlayerAction::RaidingParty, PlayerAction::MaximumCarnage,
            PlayerAction::LookIntoMyEyes, PlayerAction::BalefulHex,
            PlayerAction::AllYouCanEat, PlayerAction::PutridRegurgitationMove,
            PlayerAction::PutridRegurgitationBlitz, PlayerAction::PutridRegurgitationBlock,
            PlayerAction::KickEmBlock, PlayerAction::KickEmBlitz, PlayerAction::BlackInk,
            PlayerAction::CatchOfTheDay, PlayerAction::ThenIStartedBlastin,
            PlayerAction::TheFlashingBlade, PlayerAction::ViciousVines,
            PlayerAction::FuriousOutburst, PlayerAction::SecureTheBall,
            PlayerAction::BreatheFire, PlayerAction::Chainsaw, PlayerAction::Stab,
            PlayerAction::ProjectileVomit, PlayerAction::AutoGazeZoat, PlayerAction::Forgo,
            PlayerAction::Incorporeal, PlayerAction::Chomp, PlayerAction::Punt,
            PlayerAction::PuntMove,
        ]
    }

    pub fn is_moving(self) -> bool {
        matches!(
            self,
            PlayerAction::Move
                | PlayerAction::BlitzMove
                | PlayerAction::HandOverMove
                | PlayerAction::PassMove
                | PlayerAction::FoulMove
                | PlayerAction::ThrowTeamMateMove
                | PlayerAction::KickTeamMateMove
                | PlayerAction::GazeMove
                | PlayerAction::PutridRegurgitationMove
                | PlayerAction::KickEmBlitz
                | PlayerAction::SecureTheBall
                | PlayerAction::PuntMove
        )
    }

    pub fn is_passing(self) -> bool {
        matches!(
            self,
            PlayerAction::Pass
                | PlayerAction::DumpOff
                | PlayerAction::HandOver
                | PlayerAction::HailMaryPass
                | PlayerAction::ThrowBomb
                | PlayerAction::HailMaryBomb
        )
    }

    pub fn allows_fumblerooskie(self) -> bool {
        self.is_moving()
    }

    pub fn is_blitzing(self) -> bool {
        self == PlayerAction::Blitz || self == PlayerAction::BlitzSelect || self.is_blitz_move()
    }

    pub fn is_gaze(self) -> bool {
        matches!(self, PlayerAction::Gaze | PlayerAction::GazeMove | PlayerAction::GazeSelect)
    }

    pub fn is_bomb(self) -> bool {
        matches!(self, PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb | PlayerAction::AllYouCanEat)
    }

    pub fn is_putrid(self) -> bool {
        self.is_putrid_block() || self == PlayerAction::PutridRegurgitationMove
    }

    pub fn is_putrid_block(self) -> bool {
        matches!(
            self,
            PlayerAction::PutridRegurgitationBlitz | PlayerAction::PutridRegurgitationBlock
        )
    }

    pub fn is_kicking_downed(self) -> bool {
        matches!(self, PlayerAction::KickEmBlitz | PlayerAction::KickEmBlock)
    }

    pub fn is_blitz_move(self) -> bool {
        matches!(
            self,
            PlayerAction::BlitzMove | PlayerAction::PutridRegurgitationMove | PlayerAction::KickEmBlitz
        )
    }

    pub fn is_standing_up(self) -> bool {
        matches!(self, PlayerAction::StandUp | PlayerAction::StandUpBlitz)
    }

    pub fn is_block_action(self) -> bool {
        matches!(
            self,
            PlayerAction::Block
                | PlayerAction::ViciousVines
                | PlayerAction::BreatheFire
                | PlayerAction::Chainsaw
                | PlayerAction::Stab
                | PlayerAction::ProjectileVomit
                | PlayerAction::Chomp
        )
    }

    pub fn force_dispatch(self) -> bool {
        matches!(self, PlayerAction::FuriousOutburst | PlayerAction::Forgo | PlayerAction::Punt)
    }

    pub fn action_type(self) -> u8 {
        match self {
            PlayerAction::Move => 1,
            PlayerAction::Block => 2,
            PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz => 3,
            PlayerAction::HandOver | PlayerAction::HandOverMove => 5,
            PlayerAction::Pass | PlayerAction::PassMove | PlayerAction::HailMaryPass | PlayerAction::DumpOff
            | PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => 7,
            PlayerAction::Foul | PlayerAction::FoulMove => 9,
            PlayerAction::StandUp => 11,
            PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => 12,
            PlayerAction::RemoveConfusion => 14,
            PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::GazeMove => 15,
            PlayerAction::MultipleBlock => 16,
            PlayerAction::Swoop => 30,
            PlayerAction::KickTeamMateMove | PlayerAction::KickTeamMate => 31,
            _ => 0,
        }
    }
}

// ─── PlayerType ───────────────────────────────────────────────────────────────

/// Classification of the player within a team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlayerType {
    #[default]
    Regular,
    BigGuy,
    Star,
    Irregular,
    RiotousRookie,
    RaisedFromDead,
    Mercenary,
    PlagueRidden,
    InfamousStaff,
}

impl PlayerType {
    pub fn name(self) -> &'static str {
        match self {
            PlayerType::Regular => "Regular",
            PlayerType::BigGuy => "Big Guy",
            PlayerType::Star => "Star",
            PlayerType::Irregular => "Irregular",
            PlayerType::RiotousRookie => "RiotousRookie",
            PlayerType::RaisedFromDead => "RaisedFromDead",
            PlayerType::Mercenary => "Mercenary",
            PlayerType::PlagueRidden => "PlagueRidden",
            PlayerType::InfamousStaff => "Infamous Staff",
        }
    }

    pub fn from_name(name: &str) -> Option<PlayerType> {
        match name {
            "Regular" => Some(PlayerType::Regular),
            "Big Guy" => Some(PlayerType::BigGuy),
            "Star" => Some(PlayerType::Star),
            "Irregular" => Some(PlayerType::Irregular),
            "RiotousRookie" => Some(PlayerType::RiotousRookie),
            "RaisedFromDead" => Some(PlayerType::RaisedFromDead),
            "Mercenary" => Some(PlayerType::Mercenary),
            "PlagueRidden" => Some(PlayerType::PlagueRidden),
            "Infamous Staff" => Some(PlayerType::InfamousStaff),
            _ => None,
        }
    }
}

// ─── PlayerGender ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlayerGender {
    #[default]
    Male,
    Female,
    Nonbinary,
    Neutral,
}

impl PlayerGender {
    pub fn name(self) -> &'static str {
        match self {
            PlayerGender::Male => "male",
            PlayerGender::Female => "female",
            PlayerGender::Nonbinary => "nonbinary",
            PlayerGender::Neutral => "neutral",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [PlayerGender::Male, PlayerGender::Female, PlayerGender::Nonbinary, PlayerGender::Neutral]
            .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn nominative(self) -> &'static str {
        match self {
            PlayerGender::Male => "he",
            PlayerGender::Female => "she",
            PlayerGender::Nonbinary => "they",
            PlayerGender::Neutral => "it",
        }
    }

    pub fn genitive(self) -> &'static str {
        match self {
            PlayerGender::Male => "his",
            PlayerGender::Female => "her",
            PlayerGender::Nonbinary => "their",
            PlayerGender::Neutral => "its",
        }
    }

    pub fn from_ordinal(ordinal: u8) -> PlayerGender {
        match ordinal {
            1 => PlayerGender::Male,
            2 => PlayerGender::Female,
            3 => PlayerGender::Nonbinary,
            _ => PlayerGender::Neutral,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_state_base() {
        let s = PlayerState::new(PS_STANDING);
        assert_eq!(s.base(), PS_STANDING);
    }

    #[test]
    fn player_state_flag_round_trip() {
        let s = PlayerState::new(PS_STANDING).change_active(true).change_confused(true);
        assert!(s.is_active());
        assert!(s.is_confused());
        assert!(!s.is_rooted());
    }

    #[test]
    fn player_state_has_tacklezones() {
        let s = PlayerState::new(PS_STANDING).change_active(true);
        assert!(s.has_tacklezones());
        let confused = s.change_confused(true);
        assert!(!confused.has_tacklezones());
    }

    #[test]
    fn standing_moving_blocked_have_tacklezones() {
        for &base in &[PS_STANDING, PS_MOVING, PS_BLOCKED] {
            assert!(PlayerState::new(base).has_tacklezones(), "base={base:#x}");
        }
    }

    #[test]
    fn prone_or_worse_no_tacklezones() {
        for &base in &[PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_BADLY_HURT,
                       PS_SERIOUS_INJURY, PS_RIP, PS_RESERVE, PS_FALLING] {
            assert!(!PlayerState::new(base).has_tacklezones(), "base={base:#x}");
        }
    }

    #[test]
    fn standing_confused_no_tacklezones() {
        let s = PlayerState::new(PS_STANDING).change_confused(true);
        assert!(s.is_confused());
        assert!(!s.has_tacklezones());
    }

    #[test]
    fn standing_hypnotized_no_tacklezones() {
        let s = PlayerState::new(PS_STANDING).change_hypnotized(true);
        assert!(s.is_hypnotized());
        assert!(!s.has_tacklezones());
    }

    #[test]
    fn prone_and_stunned_are_prone_or_stunned() {
        assert!(PlayerState::new(PS_PRONE).is_prone_or_stunned());
        assert!(PlayerState::new(PS_STUNNED).is_prone_or_stunned());
    }

    #[test]
    fn others_are_not_prone_or_stunned() {
        for &base in &[PS_STANDING, PS_MOVING, PS_KNOCKED_OUT, PS_BADLY_HURT] {
            assert!(!PlayerState::new(base).is_prone_or_stunned(), "base={base:#x}");
        }
    }

    #[test]
    fn removed_from_play_states_are_casualty_or_banned() {
        assert!(PlayerState::new(PS_BADLY_HURT).is_casualty());
        assert!(PlayerState::new(PS_SERIOUS_INJURY).is_casualty());
        assert!(PlayerState::new(PS_RIP).is_casualty());
    }

    #[test]
    fn active_states_not_casualty() {
        for &base in &[PS_STANDING, PS_MOVING, PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_RESERVE] {
            assert!(!PlayerState::new(base).is_casualty(), "base={base:#x}");
        }
    }

    #[test]
    fn confused_is_distracted() {
        let s = PlayerState::new(PS_STANDING).change_confused(true);
        assert!(s.is_confused());
        assert!(s.is_distracted());
    }

    #[test]
    fn hypnotized_is_distracted() {
        let s = PlayerState::new(PS_STANDING).change_hypnotized(true);
        assert!(s.is_hypnotized());
        assert!(s.is_distracted());
    }

    #[test]
    fn normal_standing_not_distracted() {
        let s = PlayerState::new(PS_STANDING);
        assert!(!s.is_distracted());
    }

    #[test]
    fn player_state_change_base_clears_flags() {
        let s = PlayerState::new(PS_STANDING)
            .change_active(true)
            .change_base(PS_KNOCKED_OUT);
        assert!(!s.is_active(), "flags should be cleared when base has no mask");
    }

    #[test]
    fn player_action_is_moving() {
        assert!(PlayerAction::Move.is_moving());
        assert!(PlayerAction::BlitzMove.is_moving());
        assert!(!PlayerAction::Block.is_moving());
    }

    #[test]
    fn player_action_is_block_action() {
        assert!(PlayerAction::Block.is_block_action());
        assert!(PlayerAction::Chainsaw.is_block_action());
        assert!(!PlayerAction::Move.is_block_action());
    }

    #[test]
    fn player_type_round_trip() {
        for pt in &[
            PlayerType::Regular,
            PlayerType::Star,
            PlayerType::InfamousStaff,
        ] {
            assert_eq!(PlayerType::from_name(pt.name()), Some(*pt));
        }
    }

    #[test]
    fn is_prone_and_is_ko() {
        assert!(PlayerState::new(PS_PRONE).is_prone());
        assert!(!PlayerState::new(PS_STANDING).is_prone());
        assert!(!PlayerState::new(PS_STUNNED).is_prone());
        assert!(PlayerState::new(PS_KNOCKED_OUT).is_ko());
        assert!(!PlayerState::new(PS_STANDING).is_ko());
        assert!(!PlayerState::new(PS_PRONE).is_ko());
    }

    #[test]
    fn is_stunned_only_for_stunned_base() {
        assert!(PlayerState::new(PS_STUNNED).is_stunned());
        assert!(!PlayerState::new(PS_PRONE).is_stunned());
        assert!(!PlayerState::new(PS_KNOCKED_OUT).is_stunned());
        assert!(!PlayerState::new(PS_STANDING).is_stunned());
    }

    #[test]
    fn can_be_blocked_only_standing_or_moving() {
        assert!(PlayerState::new(PS_STANDING).can_be_blocked());
        assert!(PlayerState::new(PS_MOVING).can_be_blocked());
        assert!(!PlayerState::new(PS_PRONE).can_be_blocked());
        assert!(!PlayerState::new(PS_STUNNED).can_be_blocked());
        assert!(!PlayerState::new(PS_KNOCKED_OUT).can_be_blocked());
    }

    #[test]
    fn can_be_fouled_only_prone_or_stunned() {
        assert!(PlayerState::new(PS_PRONE).can_be_fouled());
        assert!(PlayerState::new(PS_STUNNED).can_be_fouled());
        assert!(!PlayerState::new(PS_STANDING).can_be_fouled());
        assert!(!PlayerState::new(PS_KNOCKED_OUT).can_be_fouled());
    }

    #[test]
    fn player_action_is_standing_up() {
        assert!(PlayerAction::StandUp.is_standing_up());
        assert!(PlayerAction::StandUpBlitz.is_standing_up());
        assert!(!PlayerAction::Move.is_standing_up());
        assert!(!PlayerAction::Block.is_standing_up());
    }

    #[test]
    fn player_action_is_passing() {
        assert!(PlayerAction::Pass.is_passing());
        assert!(PlayerAction::DumpOff.is_passing());
        assert!(PlayerAction::HandOver.is_passing());
        assert!(!PlayerAction::Move.is_passing());
        assert!(!PlayerAction::Block.is_passing());
    }

    #[test]
    fn player_action_is_blitzing() {
        assert!(PlayerAction::Blitz.is_blitzing());
        assert!(PlayerAction::BlitzMove.is_blitzing());
        assert!(!PlayerAction::Move.is_blitzing());
        assert!(!PlayerAction::Block.is_blitzing());
    }

    #[test]
    fn player_action_type_codes() {
        assert_eq!(PlayerAction::Move.action_type(), 1);
        assert_eq!(PlayerAction::Block.action_type(), 2);
        assert_eq!(PlayerAction::Blitz.action_type(), 3);
        assert_eq!(PlayerAction::Pass.action_type(), 7);
        assert_eq!(PlayerAction::Foul.action_type(), 9);
        assert_eq!(PlayerAction::StandUp.action_type(), 11);
    }

    #[test]
    fn player_action_gaze() {
        assert!(PlayerAction::Gaze.is_gaze());
        assert!(PlayerAction::GazeMove.is_gaze());
        assert!(PlayerAction::GazeSelect.is_gaze());
        assert!(!PlayerAction::Move.is_gaze());
    }

    #[test]
    fn player_action_bomb() {
        assert!(PlayerAction::ThrowBomb.is_bomb());
        assert!(!PlayerAction::Move.is_bomb());
    }

    #[test]
    fn player_gender_count_is_four() {
        let all = [PlayerGender::Male, PlayerGender::Female, PlayerGender::Nonbinary, PlayerGender::Neutral];
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn player_gender_male_nominative_is_he() {
        assert_eq!(PlayerGender::Male.nominative(), "he");
    }

    #[test]
    fn player_gender_female_nominative_is_she() {
        assert_eq!(PlayerGender::Female.nominative(), "she");
    }

    #[test]
    fn player_gender_nonbinary_nominative_is_they() {
        assert_eq!(PlayerGender::Nonbinary.nominative(), "they");
    }

    #[test]
    fn player_gender_neutral_nominative_is_it() {
        assert_eq!(PlayerGender::Neutral.nominative(), "it");
    }

    #[test]
    fn player_gender_nonbinary_genitive_is_their() {
        assert_eq!(PlayerGender::Nonbinary.genitive(), "their");
    }

    #[test]
    fn player_gender_from_ordinal_one_is_male() {
        assert_eq!(PlayerGender::from_ordinal(1), PlayerGender::Male);
    }

    #[test]
    fn player_gender_from_ordinal_default_is_neutral() {
        assert_eq!(PlayerGender::from_ordinal(0), PlayerGender::Neutral);
        assert_eq!(PlayerGender::from_ordinal(99), PlayerGender::Neutral);
    }

    #[test]
    fn player_gender_all_genitives_non_empty() {
        for g in [PlayerGender::Male, PlayerGender::Female, PlayerGender::Nonbinary, PlayerGender::Neutral] {
            assert!(!g.genitive().is_empty());
        }
    }

    #[test]
    fn player_gender_serde_round_trip() {
        let g = PlayerGender::Female;
        let json = serde_json::to_string(&g).unwrap();
        let back: PlayerGender = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }

    #[test]
    fn player_type_count_is_nine() {
        let all = [
            PlayerType::Regular, PlayerType::BigGuy, PlayerType::Star, PlayerType::Irregular,
            PlayerType::RiotousRookie, PlayerType::RaisedFromDead, PlayerType::Mercenary,
            PlayerType::PlagueRidden, PlayerType::InfamousStaff,
        ];
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn player_type_all_have_non_empty_names() {
        for t in [
            PlayerType::Regular, PlayerType::BigGuy, PlayerType::Star, PlayerType::Irregular,
            PlayerType::RiotousRookie, PlayerType::RaisedFromDead, PlayerType::Mercenary,
            PlayerType::PlagueRidden, PlayerType::InfamousStaff,
        ] {
            assert!(!t.name().is_empty());
        }
    }

    #[test]
    fn player_type_big_guy_name() {
        assert_eq!(PlayerType::BigGuy.name(), "Big Guy");
    }

    #[test]
    fn player_type_regular_name() {
        assert_eq!(PlayerType::Regular.name(), "Regular");
    }

    #[test]
    fn player_type_serde_round_trip() {
        let t = PlayerType::Star;
        let json = serde_json::to_string(&t).unwrap();
        let back: PlayerType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}
