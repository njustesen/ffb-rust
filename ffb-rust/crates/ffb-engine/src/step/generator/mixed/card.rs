/// Builds the special card step sequence (BB2016/BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.mixed.Card`.
use ffb_model::inducement::card::Card as CardModel;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

/// Parameters for the mixed Card sequence.
#[derive(Debug, Clone, Default)]
pub struct CardParams {
    /// The inducement card to play.
    pub card_id: Option<CardModel>,
    /// Whether the card is being played by the home team.
    pub home_team: bool,
}

pub struct Card;

impl Card {
    pub fn new() -> Self { Self }

    /// Build the mixed card step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &CardParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 PLAY_CARD
        seq.add(StepId::PlayCard, vec![
            StepParameter::CardId(params.card_id.clone()),
            StepParameter::HomeTeam(params.home_team),
        ]);
        seq.build()
    }
}

impl Default for Card {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_has_one_step() {
        let steps = Card::build_sequence(&CardParams::default());
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn card_starts_with_play_card() {
        let steps = Card::build_sequence(&CardParams::default());
        assert_eq!(steps[0].step_id, StepId::PlayCard);
    }

    #[test]
    fn card_play_card_has_no_label() {
        let steps = Card::build_sequence(&CardParams::default());
        assert!(steps[0].label.is_none());
    }

    #[test]
    fn params_with_fields_set() {
        let p = CardParams {
            card_id: Some(CardModel::new("card-1", None::<&str>)),
            home_team: true,
        };
        assert_eq!(p.card_id.as_ref().map(|c| c.get_name()), Some("card-1"));
        assert!(p.home_team);
    }

    #[test]
    fn params_clone() {
        let p = CardParams { card_id: Some(CardModel::new("x", None::<&str>)), home_team: true };
        let q = p.clone();
        assert_eq!(q.card_id.as_ref().map(|c| c.get_name()), Some("x"));
        assert!(q.home_team);
    }
}
