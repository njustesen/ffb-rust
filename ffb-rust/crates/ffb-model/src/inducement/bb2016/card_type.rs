use crate::inducement::card_type::CardType as ICardType;
use crate::option::game_option_id;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2016.CardType`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    MAGIC_ITEM,
    DIRTY_TRICK,
}

impl ICardType for CardType {
    fn get_name(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "magicItem",
            CardType::DIRTY_TRICK => "dirtyTrick",
        }
    }

    fn get_deck_name(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "Magic Items Deck",
            CardType::DIRTY_TRICK => "Dirty Tricks Deck",
        }
    }

    fn get_inducement_name_single(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "Magic Item Card",
            CardType::DIRTY_TRICK => "Dirty Trick Card",
        }
    }

    fn get_inducement_name_multiple(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "Magic Item Cards",
            CardType::DIRTY_TRICK => "Dirty Trick Cards",
        }
    }

    fn get_max_id(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => game_option_id::CARDS_MAGIC_ITEM_MAX,
            CardType::DIRTY_TRICK => game_option_id::CARDS_DIRTY_TRICK_MAX,
        }
    }

    fn get_cost_id(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => game_option_id::CARDS_MAGIC_ITEM_COST,
            CardType::DIRTY_TRICK => game_option_id::CARDS_DIRTY_TRICK_COST,
        }
    }

    fn get_card_front(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "cardMagicItemFront",
            CardType::DIRTY_TRICK => "cardDirtyTrickFront",
        }
    }

    fn get_card_back(&self) -> &str {
        match self {
            CardType::MAGIC_ITEM => "cardMagicItemBack",
            CardType::DIRTY_TRICK => "cardDirtyTrickBack",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducement::card_type::CardType as ICardType;

    #[test]
    fn magic_item_name() {
        assert_eq!(CardType::MAGIC_ITEM.get_name(), "magicItem");
    }

    #[test]
    fn dirty_trick_deck_name() {
        assert_eq!(CardType::DIRTY_TRICK.get_deck_name(), "Dirty Tricks Deck");
    }

    #[test]
    fn both_variants_have_non_empty_ids() {
        for ct in &[CardType::MAGIC_ITEM, CardType::DIRTY_TRICK] {
            assert!(!ct.get_max_id().is_empty());
            assert!(!ct.get_cost_id().is_empty());
        }
    }
}
