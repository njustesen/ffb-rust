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

    pub fn from_name(name: &str) -> Option<TurnMode> {
        match name {
            "regular" => Some(TurnMode::Regular),
            "setup" => Some(TurnMode::Setup),
            "kickoff" => Some(TurnMode::Kickoff),
            "perfectDefence" => Some(TurnMode::PerfectDefence),
            "solidDefence" => Some(TurnMode::SolidDefence),
            "quickSnap" => Some(TurnMode::QuickSnap),
            "highKick" => Some(TurnMode::HighKick),
            "startGame" => Some(TurnMode::StartGame),
            "blitz" => Some(TurnMode::Blitz),
            "touchback" => Some(TurnMode::Touchback),
            "interception" => Some(TurnMode::Interception),
            "endGame" => Some(TurnMode::EndGame),
            "swarming" => Some(TurnMode::Swarming),
            "kickoffReturn" => Some(TurnMode::KickoffReturn),
            "wizard" => Some(TurnMode::Wizard),
            "passBlock" => Some(TurnMode::PassBlock),
            "dumpOff" => Some(TurnMode::DumpOff),
            "noPlayersToField" => Some(TurnMode::NoPlayersToField),
            "bombHome" => Some(TurnMode::BombHome),
            "bombAway" => Some(TurnMode::BombAway),
            "bombHomeBlitz" => Some(TurnMode::BombHomeBlitz),
            "bombAwayBlitz" => Some(TurnMode::BombAwayBlitz),
            "illegalSubstitution" => Some(TurnMode::IllegalSubstitution),
            "selectBlitzTarget" => Some(TurnMode::SelectBlitzTarget),
            "selectGazeTarget" => Some(TurnMode::SelectGazeTarget),
            "safePairOfHands" => Some(TurnMode::SafePairOfHands),
            "selectBlockKind" => Some(TurnMode::SelectBlockKind),
            "betweenTurns" => Some(TurnMode::BetweenTurns),
            "trickster" => Some(TurnMode::Trickster),
            "raidingParty" => Some(TurnMode::RaidingParty),
            "hitAndRun" => Some(TurnMode::HitAndRun),
            "thenIStartedBlastin" => Some(TurnMode::ThenIStartedBlastin),
            _ => None,
        }
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

    #[test]
    fn round_trip_name() {
        let modes = [TurnMode::Regular, TurnMode::Kickoff, TurnMode::BombHome, TurnMode::ThenIStartedBlastin];
        for m in &modes {
            assert_eq!(TurnMode::from_name(m.name()), Some(*m));
        }
    }

    #[test]
    fn bomb_turn_detection() {
        assert!(TurnMode::BombHome.is_bomb_turn());
        assert!(!TurnMode::Regular.is_bomb_turn());
    }

    #[test]
    fn serde_round_trip() {
        let m = TurnMode::SolidDefence;
        let json = serde_json::to_string(&m).unwrap();
        let back: TurnMode = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn all_have_non_empty_names() {
        let all = [
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
        for m in all {
            assert!(!m.name().is_empty());
        }
    }

    #[test]
    fn all_from_name_round_trip() {
        let all = [
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
        for m in all {
            assert_eq!(TurnMode::from_name(m.name()), Some(m));
        }
    }

    #[test]
    fn regular_is_basic_mode() {
        assert!(TurnMode::Regular.is_basic_mode());
    }

    #[test]
    fn blitz_is_basic_mode() {
        assert!(TurnMode::Blitz.is_basic_mode());
    }

    #[test]
    fn setup_is_not_basic_mode() {
        assert!(!TurnMode::Setup.is_basic_mode());
    }

    #[test]
    fn regular_checks_for_active_players() {
        assert!(TurnMode::Regular.check_for_active_players());
    }

    #[test]
    fn setup_does_not_check_for_active_players() {
        assert!(!TurnMode::Setup.check_for_active_players());
    }

    #[test]
    fn dump_off_does_not_allow_end_player_action() {
        assert!(!TurnMode::DumpOff.allow_end_player_action());
    }

    #[test]
    fn regular_allows_end_player_action() {
        assert!(TurnMode::Regular.allow_end_player_action());
    }

    #[test]
    fn trickster_forces_dice_decoration_update() {
        assert!(TurnMode::Trickster.force_dice_decoration_update());
    }

    #[test]
    fn regular_does_not_force_dice_decoration_update() {
        assert!(!TurnMode::Regular.force_dice_decoration_update());
    }

    #[test]
    fn kickoff_return_does_not_check_negatraits() {
        assert!(!TurnMode::KickoffReturn.check_negatraits());
    }

    #[test]
    fn regular_checks_negatraits() {
        assert!(TurnMode::Regular.check_negatraits());
    }
}
