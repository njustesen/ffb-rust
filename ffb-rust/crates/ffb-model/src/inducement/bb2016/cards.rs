use crate::inducement::card::Card;
use crate::inducement::cards::Cards as ICards;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2016.Cards`.
///
/// The full set of BB2016 cards (Magic Items + Dirty Tricks).
/// The Java implementation defines 26 anonymous Card subclasses with complex
/// game effects; in Rust we enumerate their names and handler keys only.
pub struct Cards {
    cards: Vec<Card>,
}

impl Default for Cards {
    fn default() -> Self {
        Self::new()
    }
}

impl Cards {
    pub fn new() -> Self {
        let cards = vec![
            // Magic Items (13)
            Card::new("Beguiling Bracers", None::<&str>),
            Card::new("Belt of Invulnerability", None::<&str>),
            Card::new("Fawndough's Headband", None::<&str>),
            Card::new("Force Shield", Some("FORCE_SHIELD")),
            Card::new("Gikta's Strength of da Bear", None::<&str>),
            Card::new("Gloves of Holding", None::<&str>),
            Card::new("Inertia Damper", None::<&str>),
            Card::new("Lucky Charm", None::<&str>),
            Card::new("Magic Gloves of Jark Longarm", None::<&str>),
            Card::new("Good Old Magic Codpiece", None::<&str>),
            Card::new("Rabbit's Foot", Some("RABBITS_FOOT")),
            Card::new("Wand of Smashing", None::<&str>),
            // Dirty Tricks (13)
            Card::new("Blatant Foul", None::<&str>),
            Card::new("Chop Block", Some("CHOP_BLOCK")),
            Card::new("Custard Pie", Some("CUSTARD_PIE")),
            Card::new("Distract", Some("DISTRACT")),
            Card::new("Greased Shoes", None::<&str>),
            Card::new("Gromskull's Exploding Runes", None::<&str>),
            Card::new("Illegal Substitution", Some("ILLEGAL_SUBSTITUTION")),
            Card::new("Kicking Boots", None::<&str>),
            Card::new("Pit Trap", Some("PIT_TRAP")),
            Card::new("Spiked Ball", None::<&str>),
            Card::new("Stolen Playbook", None::<&str>),
            Card::new("Witch's Brew", Some("WITCH_BREW")),
        ];
        Self { cards }
    }
}

impl ICards for Cards {
    fn get_key(&self) -> &str {
        "Cards"
    }

    fn all_cards(&self) -> &[Card] {
        &self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducement::cards::Cards as ICards;

    #[test]
    fn has_24_cards() {
        let c = Cards::new();
        assert_eq!(c.all_cards().len(), 24);
    }

    #[test]
    fn force_shield_has_handler_key() {
        let c = Cards::new();
        let fs = c.all_cards().iter().find(|card| card.get_name() == "Force Shield").unwrap();
        assert_eq!(fs.handler_key_name(), Some("FORCE_SHIELD"));
    }

    #[test]
    fn beguiling_bracers_has_no_handler_key() {
        let c = Cards::new();
        let bb = c.all_cards().iter().find(|card| card.get_name() == "Beguiling Bracers").unwrap();
        assert!(bb.handler_key_name().is_none());
    }
}
