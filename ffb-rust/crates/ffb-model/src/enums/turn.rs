use serde::{Deserialize, Serialize};

/// The current phase/mode of a team's turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnMode {
    Regular,
    Setup,
    Kickoff,
    PerfectDefence,
    SolidDefence,
    QuickSnap,
    HighKick,
    StartGame,
    Blitz,
    Touchback,
    Interception,
    EndGame,
    Swarming,
    KickoffReturn,
    Wizard,
    PassBlock,
    DumpOff,
    NoPlayersToField,
    BombHome,
    BombAway,
    BombHomeBlitz,
    BombAwayBlitz,
    IllegalSubstitution,
    SelectBlitzTarget,
    SelectGazeTarget,
    SafePairOfHands,
    SelectBlockKind,
    BetweenTurns,
    Trickster,
    RaidingParty,
    HitAndRun,
    ThenIStartedBlastin,
}

impl TurnMode {
    pub fn name(self) -> &'static str {
        match self {
            TurnMode::Regular => "regular",
            TurnMode::Setup => "setup",
            TurnMode::Kickoff => "kickoff",
            TurnMode::PerfectDefence => "perfectDefence",
            TurnMode::SolidDefence => "solidDefence",
            TurnMode::QuickSnap => "quickSnap",
            TurnMode::HighKick => "highKick",
            TurnMode::StartGame => "startGame",
            TurnMode::Blitz => "blitz",
            TurnMode::Touchback => "touchback",
            TurnMode::Interception => "interception",
            TurnMode::EndGame => "endGame",
            TurnMode::Swarming => "swarming",
            TurnMode::KickoffReturn => "kickoffReturn",
            TurnMode::Wizard => "wizard",
            TurnMode::PassBlock => "passBlock",
            TurnMode::DumpOff => "dumpOff",
            TurnMode::NoPlayersToField => "noPlayersToField",
            TurnMode::BombHome => "bombHome",
            TurnMode::BombAway => "bombAway",
            TurnMode::BombHomeBlitz => "bombHomeBlitz",
            TurnMode::BombAwayBlitz => "bombAwayBlitz",
            TurnMode::IllegalSubstitution => "illegalSubstitution",
            TurnMode::SelectBlitzTarget => "selectBlitzTarget",
            TurnMode::SelectGazeTarget => "selectGazeTarget",
            TurnMode::SafePairOfHands => "safePairOfHands",
            TurnMode::SelectBlockKind => "selectBlockKind",
            TurnMode::BetweenTurns => "betweenTurns",
            TurnMode::Trickster => "trickster",
            TurnMode::RaidingParty => "raidingParty",
            TurnMode::HitAndRun => "hitAndRun",
            TurnMode::ThenIStartedBlastin => "thenIStartedBlastin",
        }
    }

    /// Java: `TurnMode.forName` matches names case-insensitively.
    pub fn from_name(name: &str) -> Option<TurnMode> {
        Self::all().iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn all() -> &'static [TurnMode] {
        &[
            TurnMode::Regular,
            TurnMode::Setup,
            TurnMode::Kickoff,
            TurnMode::PerfectDefence,
            TurnMode::SolidDefence,
            TurnMode::QuickSnap,
            TurnMode::HighKick,
            TurnMode::StartGame,
            TurnMode::Blitz,
            TurnMode::Touchback,
            TurnMode::Interception,
            TurnMode::EndGame,
            TurnMode::Swarming,
            TurnMode::KickoffReturn,
            TurnMode::Wizard,
            TurnMode::PassBlock,
            TurnMode::DumpOff,
            TurnMode::NoPlayersToField,
            TurnMode::BombHome,
            TurnMode::BombAway,
            TurnMode::BombHomeBlitz,
            TurnMode::BombAwayBlitz,
            TurnMode::IllegalSubstitution,
            TurnMode::SelectBlitzTarget,
            TurnMode::SelectGazeTarget,
            TurnMode::SafePairOfHands,
            TurnMode::SelectBlockKind,
            TurnMode::BetweenTurns,
            TurnMode::Trickster,
            TurnMode::RaidingParty,
            TurnMode::HitAndRun,
            TurnMode::ThenIStartedBlastin,
        ]
    }

    /// Whether to check for active negatrait (BoneHead/ReallyStupid/etc.) at turn start.
    pub fn check_negatraits(self) -> bool {
        !matches!(self, TurnMode::KickoffReturn | TurnMode::PassBlock) && !self.is_bomb_turn()
    }

    pub fn is_bomb_turn(self) -> bool {
        matches!(
            self,
            TurnMode::BombHome | TurnMode::BombHomeBlitz | TurnMode::BombAway | TurnMode::BombAwayBlitz
        )
    }

    pub fn allow_end_player_action(self) -> bool {
        !self.is_bomb_turn() && self != TurnMode::DumpOff
    }

    /// Whether active-player count should be checked (affects re-roll availability).
    pub fn check_for_active_players(self) -> bool {
        matches!(self, TurnMode::Regular | TurnMode::Blitz | TurnMode::BetweenTurns)
    }

    pub fn force_dice_decoration_update(self) -> bool {
        self == TurnMode::Trickster
    }

    pub fn is_basic_mode(self) -> bool {
        matches!(self, TurnMode::Regular | TurnMode::Blitz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [TurnMode; 32] = [
        TurnMode::Regular, TurnMode::Setup, TurnMode::Kickoff, TurnMode::PerfectDefence,
        TurnMode::SolidDefence, TurnMode::QuickSnap, TurnMode::HighKick, TurnMode::StartGame,
        TurnMode::Blitz, TurnMode::Touchback, TurnMode::Interception, TurnMode::EndGame,
        TurnMode::Swarming, TurnMode::KickoffReturn, TurnMode::Wizard, TurnMode::PassBlock,
        TurnMode::DumpOff, TurnMode::NoPlayersToField, TurnMode::BombHome, TurnMode::BombAway,
        TurnMode::BombHomeBlitz, TurnMode::BombAwayBlitz, TurnMode::IllegalSubstitution,
        TurnMode::SelectBlitzTarget, TurnMode::SelectGazeTarget, TurnMode::SafePairOfHands,
        TurnMode::SelectBlockKind, TurnMode::BetweenTurns, TurnMode::Trickster,
        TurnMode::RaidingParty, TurnMode::HitAndRun, TurnMode::ThenIStartedBlastin,
    ];

    // ── TurnModeTest mirrors (ffb-server/.../server/model/TurnModeTest.java) ──

    #[test]
    fn turn_mode_all_have_non_null_name() {
        for m in ALL {
            assert!(!m.name().is_empty());
        }
    }

    #[test]
    fn turn_mode_regular_is_basic_mode() {
        assert!(TurnMode::Regular.is_basic_mode());
    }

    #[test]
    fn turn_mode_blitz_is_basic_mode() {
        assert!(TurnMode::Blitz.is_basic_mode());
    }

    #[test]
    fn turn_mode_setup_is_not_basic_mode() {
        assert!(!TurnMode::Setup.is_basic_mode());
    }

    #[test]
    fn turn_mode_regular_checks_for_active_players() {
        assert!(TurnMode::Regular.check_for_active_players());
    }

    #[test]
    fn turn_mode_setup_does_not_check_for_active_players() {
        assert!(!TurnMode::Setup.check_for_active_players());
    }

    #[test]
    fn turn_mode_bomb_home_is_bomb_turn() {
        assert!(TurnMode::BombHome.is_bomb_turn());
        assert!(TurnMode::BombAway.is_bomb_turn());
        assert!(TurnMode::BombHomeBlitz.is_bomb_turn());
        assert!(TurnMode::BombAwayBlitz.is_bomb_turn());
    }

    #[test]
    fn turn_mode_regular_is_not_bomb_turn() {
        assert!(!TurnMode::Regular.is_bomb_turn());
    }

    #[test]
    fn turn_mode_dump_off_does_not_allow_end_player_action() {
        assert!(!TurnMode::DumpOff.allow_end_player_action());
    }

    #[test]
    fn turn_mode_regular_allows_end_player_action() {
        assert!(TurnMode::Regular.allow_end_player_action());
    }

    #[test]
    fn turn_mode_trickster_forces_dice_decoration_update() {
        assert!(TurnMode::Trickster.force_dice_decoration_update());
    }

    #[test]
    fn turn_mode_regular_does_not_force_dice_decoration_update() {
        assert!(!TurnMode::Regular.force_dice_decoration_update());
    }

    #[test]
    fn turn_mode_for_name_regular() {
        assert_eq!(TurnMode::from_name("regular"), Some(TurnMode::Regular));
    }

    #[test]
    fn turn_mode_for_name_case_insensitive() {
        assert_eq!(TurnMode::from_name("KICKOFF"), Some(TurnMode::Kickoff));
    }

    #[test]
    fn turn_mode_kickoff_return_does_not_check_nega_traits() {
        assert!(!TurnMode::KickoffReturn.check_negatraits());
    }

    #[test]
    fn turn_mode_regular_checks_nega_traits() {
        assert!(TurnMode::Regular.check_negatraits());
    }

    // ── shared with Java (ported Rust extra) ────────────────────────────────

    #[test]
    fn turn_mode_all_from_name_round_trip() {
        for m in ALL {
            assert_eq!(TurnMode::from_name(m.name()), Some(m));
        }
    }

    // ── Rust-only: serde ────────────────────────────────────────────────────

    #[test]
    fn serde_round_trip() {
        let m = TurnMode::SolidDefence;
        let json = serde_json::to_string(&m).unwrap();
        let back: TurnMode = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
