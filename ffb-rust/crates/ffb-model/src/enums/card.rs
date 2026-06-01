use serde::{Deserialize, Serialize};

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
}
