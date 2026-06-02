use serde::{Deserialize, Serialize};

/// How long a card effect lasts (maps to Java's InducementDuration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InducementDuration {
    UntilEndOfGame,
    UntilEndOfDrive,
    UntilEndOfTurn,
    WhileHoldingTheBall,
    UntilUsed,
    UntilEndOfOpponentsTurn,
    UntilEndOfHalf,
}

impl InducementDuration {
    pub fn id(self) -> u8 {
        match self {
            InducementDuration::UntilEndOfGame => 1,
            InducementDuration::UntilEndOfDrive => 2,
            InducementDuration::UntilEndOfTurn => 3,
            InducementDuration::WhileHoldingTheBall => 4,
            InducementDuration::UntilUsed => 5,
            InducementDuration::UntilEndOfOpponentsTurn => 6,
            InducementDuration::UntilEndOfHalf => 7,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::UntilEndOfGame),
            2 => Some(Self::UntilEndOfDrive),
            3 => Some(Self::UntilEndOfTurn),
            4 => Some(Self::WhileHoldingTheBall),
            5 => Some(Self::UntilUsed),
            6 => Some(Self::UntilEndOfOpponentsTurn),
            7 => Some(Self::UntilEndOfHalf),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            InducementDuration::UntilEndOfGame => "untilEndOfGame",
            InducementDuration::UntilEndOfDrive => "untilEndOfDrive",
            InducementDuration::UntilEndOfTurn => "untilEndOfTurn",
            InducementDuration::WhileHoldingTheBall => "whileHoldingTheBall",
            InducementDuration::UntilUsed => "untilUsed",
            InducementDuration::UntilEndOfOpponentsTurn => "untilEndOfOpponentsTurn",
            InducementDuration::UntilEndOfHalf => "untilEndOfHalf",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "untilEndOfGame" => Some(Self::UntilEndOfGame),
            "untilEndOfDrive" => Some(Self::UntilEndOfDrive),
            "untilEndOfTurn" => Some(Self::UntilEndOfTurn),
            "whileHoldingTheBall" => Some(Self::WhileHoldingTheBall),
            "untilUsed" => Some(Self::UntilUsed),
            "untilEndOfOpponentsTurn" => Some(Self::UntilEndOfOpponentsTurn),
            "untilEndOfHalf" => Some(Self::UntilEndOfHalf),
            _ => None,
        }
    }
}

/// When a card may be played during a game (maps to Java's InducementPhase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InducementPhase {
    EndOfOpponentTurn,
    EndOfOwnTurn,
    StartOfOwnTurn,
    AfterKickoffToOpponent,
    AfterInducementsPurchased,
    BeforeKickoffScatter,
    EndOfTurnNotHalf,
    BeforeSetup,
}

impl InducementPhase {
    pub fn name(self) -> &'static str {
        match self {
            InducementPhase::EndOfOpponentTurn => "endOfOpponentTurn",
            InducementPhase::EndOfOwnTurn => "endOfOwnTurn",
            InducementPhase::StartOfOwnTurn => "startOfOwnTurn",
            InducementPhase::AfterKickoffToOpponent => "afterKickoffToOpponent",
            InducementPhase::AfterInducementsPurchased => "afterInducementsPurchased",
            InducementPhase::BeforeKickoffScatter => "beforeKickoffScatter",
            InducementPhase::EndOfTurnNotHalf => "endOfTurnNotHalf",
            InducementPhase::BeforeSetup => "beforeSetup",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "endOfOpponentTurn" => Some(Self::EndOfOpponentTurn),
            "endOfOwnTurn" => Some(Self::EndOfOwnTurn),
            "startOfOwnTurn" => Some(Self::StartOfOwnTurn),
            "afterKickoffToOpponent" => Some(Self::AfterKickoffToOpponent),
            "afterInducementsPurchased" => Some(Self::AfterInducementsPurchased),
            "beforeKickoffScatter" => Some(Self::BeforeKickoffScatter),
            "endOfTurnNotHalf" => Some(Self::EndOfTurnNotHalf),
            "beforeSetup" => Some(Self::BeforeSetup),
            _ => None,
        }
    }
}

/// Effect types a card can apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardEffect {
    Distracted,
    IllegallySubstituted,
    MadCapMushroomPotion,
    Sedative,
    Poisoned,
}

impl CardEffect {
    pub fn name(self) -> &'static str {
        match self {
            CardEffect::Distracted => "Distracted",
            CardEffect::IllegallySubstituted => "IllegallySubstituted",
            CardEffect::MadCapMushroomPotion => "MadCapMushroomPotion",
            CardEffect::Sedative => "Sedative",
            CardEffect::Poisoned => "Poisoned",
        }
    }

    pub fn from_name(name: &str) -> Option<CardEffect> {
        match name {
            "Distracted" => Some(CardEffect::Distracted),
            "IllegallySubstituted" => Some(CardEffect::IllegallySubstituted),
            "MadCapMushroomPotion" => Some(CardEffect::MadCapMushroomPotion),
            "Sedative" => Some(CardEffect::Sedative),
            "Poisoned" => Some(CardEffect::Poisoned),
            _ => None,
        }
    }
}

/// Who a card is played on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CardTarget {
    Turn,
    OwnPlayer,
    OpposingPlayer,
    AnyPlayer,
}

impl CardTarget {
    pub fn id(self) -> u8 {
        match self {
            CardTarget::Turn => 1,
            CardTarget::OwnPlayer => 2,
            CardTarget::OpposingPlayer => 3,
            CardTarget::AnyPlayer => 4,
        }
    }

    pub fn from_id(id: u8) -> Option<CardTarget> {
        match id {
            1 => Some(CardTarget::Turn),
            2 => Some(CardTarget::OwnPlayer),
            3 => Some(CardTarget::OpposingPlayer),
            4 => Some(CardTarget::AnyPlayer),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CardTarget::Turn => "turn",
            CardTarget::OwnPlayer => "ownPlayer",
            CardTarget::OpposingPlayer => "opposingPlayer",
            CardTarget::AnyPlayer => "anyPlayer",
        }
    }

    pub fn is_played_on_player(self) -> bool {
        matches!(
            self,
            CardTarget::OwnPlayer | CardTarget::OpposingPlayer | CardTarget::AnyPlayer
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_target_round_trip_id() {
        for id in 1u8..=4 {
            let t = CardTarget::from_id(id).unwrap();
            assert_eq!(t.id(), id);
        }
    }

    #[test]
    fn card_effect_round_trip_name() {
        for e in &[
            CardEffect::Distracted,
            CardEffect::IllegallySubstituted,
            CardEffect::MadCapMushroomPotion,
            CardEffect::Sedative,
            CardEffect::Poisoned,
        ] {
            assert_eq!(CardEffect::from_name(e.name()), Some(*e));
        }
    }

    #[test]
    fn card_effect_count_is_five() {
        let all = [
            CardEffect::Distracted, CardEffect::IllegallySubstituted,
            CardEffect::MadCapMushroomPotion, CardEffect::Sedative, CardEffect::Poisoned,
        ];
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn card_effect_all_have_non_empty_names() {
        for e in [
            CardEffect::Distracted, CardEffect::IllegallySubstituted,
            CardEffect::MadCapMushroomPotion, CardEffect::Sedative, CardEffect::Poisoned,
        ] {
            assert!(!e.name().is_empty());
        }
    }

    #[test]
    fn card_effect_distracted_name() {
        assert_eq!(CardEffect::Distracted.name(), "Distracted");
    }

    #[test]
    fn card_target_count_is_four() {
        let all = [CardTarget::Turn, CardTarget::OwnPlayer, CardTarget::OpposingPlayer, CardTarget::AnyPlayer];
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn card_target_full_round_trip_id() {
        for id in 1u8..=4 {
            let t = CardTarget::from_id(id).unwrap();
            assert_eq!(t.id(), id);
        }
    }

    #[test]
    fn card_target_turn_is_not_played_on_player() {
        assert!(!CardTarget::Turn.is_played_on_player());
    }

    #[test]
    fn card_target_own_player_is_played_on_player() {
        assert!(CardTarget::OwnPlayer.is_played_on_player());
        assert!(CardTarget::AnyPlayer.is_played_on_player());
    }

    #[test]
    fn card_effect_illegally_substituted_name() {
        assert_eq!(CardEffect::IllegallySubstituted.name(), "IllegallySubstituted");
    }

    #[test]
    fn card_effect_mad_cap_name() {
        assert_eq!(CardEffect::MadCapMushroomPotion.name(), "MadCapMushroomPotion");
    }

    #[test]
    fn card_effect_sedative_name() {
        assert_eq!(CardEffect::Sedative.name(), "Sedative");
    }

    #[test]
    fn card_effect_poisoned_name() {
        assert_eq!(CardEffect::Poisoned.name(), "Poisoned");
    }

    #[test]
    fn card_target_opposing_player_is_played_on_player() {
        assert!(CardTarget::OpposingPlayer.is_played_on_player());
    }

    #[test]
    fn card_target_own_player_name() {
        assert_eq!(CardTarget::OwnPlayer.name(), "ownPlayer");
    }

    #[test]
    fn card_target_opposing_player_name() {
        assert_eq!(CardTarget::OpposingPlayer.name(), "opposingPlayer");
    }

    #[test]
    fn card_target_any_player_name() {
        assert_eq!(CardTarget::AnyPlayer.name(), "anyPlayer");
    }

    #[test]
    fn inducement_duration_round_trip_id() {
        for id in 1u8..=7 {
            let d = InducementDuration::from_id(id).unwrap();
            assert_eq!(d.id(), id);
        }
        assert!(InducementDuration::from_id(0).is_none());
        assert!(InducementDuration::from_id(8).is_none());
    }

    #[test]
    fn inducement_duration_round_trip_name() {
        let all = [
            InducementDuration::UntilEndOfGame,
            InducementDuration::UntilEndOfDrive,
            InducementDuration::UntilEndOfTurn,
            InducementDuration::WhileHoldingTheBall,
            InducementDuration::UntilUsed,
            InducementDuration::UntilEndOfOpponentsTurn,
            InducementDuration::UntilEndOfHalf,
        ];
        for d in all {
            assert_eq!(InducementDuration::from_name(d.name()), Some(d));
        }
    }

    #[test]
    fn inducement_duration_count_is_seven() {
        let all = [
            InducementDuration::UntilEndOfGame, InducementDuration::UntilEndOfDrive,
            InducementDuration::UntilEndOfTurn, InducementDuration::WhileHoldingTheBall,
            InducementDuration::UntilUsed, InducementDuration::UntilEndOfOpponentsTurn,
            InducementDuration::UntilEndOfHalf,
        ];
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn inducement_phase_round_trip_name() {
        let all = [
            InducementPhase::EndOfOpponentTurn, InducementPhase::EndOfOwnTurn,
            InducementPhase::StartOfOwnTurn, InducementPhase::AfterKickoffToOpponent,
            InducementPhase::AfterInducementsPurchased, InducementPhase::BeforeKickoffScatter,
            InducementPhase::EndOfTurnNotHalf, InducementPhase::BeforeSetup,
        ];
        for p in all {
            assert_eq!(InducementPhase::from_name(p.name()), Some(p));
        }
    }

    #[test]
    fn inducement_phase_count_is_eight() {
        let all = [
            InducementPhase::EndOfOpponentTurn, InducementPhase::EndOfOwnTurn,
            InducementPhase::StartOfOwnTurn, InducementPhase::AfterKickoffToOpponent,
            InducementPhase::AfterInducementsPurchased, InducementPhase::BeforeKickoffScatter,
            InducementPhase::EndOfTurnNotHalf, InducementPhase::BeforeSetup,
        ];
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn inducement_phase_start_of_own_turn_name() {
        assert_eq!(InducementPhase::StartOfOwnTurn.name(), "startOfOwnTurn");
    }
}
