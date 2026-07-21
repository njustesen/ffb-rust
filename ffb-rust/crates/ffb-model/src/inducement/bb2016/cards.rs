use crate::enums::InducementDuration;
use crate::inducement::card::Card;
use crate::inducement::card_target::CardTarget;
use crate::inducement::cards::Cards as ICards;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2016.Cards`.
///
/// The full set of BB2016 cards (Magic Items + Dirty Tricks).
/// The Java implementation defines 26 anonymous Card subclasses with complex
/// game effects; in Rust we enumerate their names, handler keys, targets, and
/// durations only (the per-card `activationEnhancement`/`rollModifiers`/etc.
/// game-effect logic is not ported here — see each card's Java source for the
/// full effect).
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
            Card::new("Beguiling Bracers", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Belt of Invulnerability", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Fawndough's Headband", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfTurn),
            Card::new("Force Shield", Some("FORCE_SHIELD"))
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::WhileHoldingTheBall),
            Card::new("Gikta's Strength of da Bear", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfDrive),
            Card::new("Gloves of Holding", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Inertia Damper", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfDrive),
            Card::new("Lucky Charm", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilUsed),
            Card::new("Magic Gloves of Jark Longarm", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfDrive),
            Card::new("Good Old Magic Codpiece", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Rabbit's Foot", Some("RABBITS_FOOT"))
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Wand of Smashing", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfTurn),
            // Dirty Tricks (13)
            Card::new("Blatant Foul", None::<&str>)
                .with_target(CardTarget::TURN)
                .with_duration(InducementDuration::UntilEndOfTurn),
            Card::new("Chop Block", Some("CHOP_BLOCK"))
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfTurn)
                .with_requires_blockable_player_selection(true),
            Card::new("Custard Pie", Some("CUSTARD_PIE"))
                .with_target(CardTarget::OPPOSING_PLAYER)
                .with_duration(InducementDuration::UntilEndOfTurn),
            Card::new("Distract", Some("DISTRACT"))
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfOpponentsTurn),
            Card::new("Greased Shoes", None::<&str>)
                .with_target(CardTarget::TURN)
                .with_duration(InducementDuration::UntilEndOfOpponentsTurn),
            Card::new("Gromskull's Exploding Runes", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Illegal Substitution", Some("ILLEGAL_SUBSTITUTION"))
                .with_target(CardTarget::TURN)
                .with_duration(InducementDuration::UntilEndOfTurn),
            Card::new("Kicking Boots", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfGame),
            Card::new("Pit Trap", Some("PIT_TRAP"))
                .with_target(CardTarget::ANY_PLAYER)
                .with_duration(InducementDuration::UntilEndOfTurn),
            Card::new("Spiked Ball", None::<&str>)
                .with_target(CardTarget::TURN)
                .with_duration(InducementDuration::UntilEndOfDrive),
            Card::new("Stolen Playbook", None::<&str>)
                .with_target(CardTarget::OWN_PLAYER)
                .with_duration(InducementDuration::UntilEndOfDrive),
            Card::new("Witch's Brew", Some("WITCH_BREW"))
                .with_target(CardTarget::OPPOSING_PLAYER)
                .with_duration(InducementDuration::UntilEndOfDrive),
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

fn find<'a>(cards: &'a Cards, name: &str) -> &'a Card {
        cards.all_cards().iter().find(|card| card.get_name() == name).unwrap()
    }

    #[test]
    fn all_cards_have_a_duration() {
        // Java: every Cards.java entry passes an explicit InducementDuration to its constructor.
        let c = Cards::new();
        for card in c.all_cards() {
            assert!(card.get_duration().is_some(), "{} is missing a duration", card.get_name());
        }
    }

    #[test]
    fn chop_block_requires_blockable_player_selection() {
        // Java: Chop Block overrides requiresBlockablePlayerSelection() to return true —
        // the only BB2016 card that does.
        let c = Cards::new();
        assert!(find(&c, "Chop Block").requires_blockable_player_selection());
        for card in c.all_cards() {
            if card.get_name() != "Chop Block" {
                assert!(
                    !card.requires_blockable_player_selection(),
                    "{} should not require blockable player selection",
                    card.get_name()
                );
            }
        }
    }

    #[test]
    fn duration_counts_match_java_source() {
        // Sanity check against the Java source's exact duration distribution:
        // 7x UntilEndOfGame, 6x UntilEndOfDrive, 7x UntilEndOfTurn, 1x WhileHoldingTheBall,
        // 1x UntilUsed, 2x UntilEndOfOpponentsTurn.
        let c = Cards::new();
        let count = |d: InducementDuration| {
            c.all_cards().iter().filter(|card| card.get_duration() == Some(d)).count()
        };
        assert_eq!(count(InducementDuration::UntilEndOfGame), 7);
        assert_eq!(count(InducementDuration::UntilEndOfDrive), 6);
        assert_eq!(count(InducementDuration::UntilEndOfTurn), 7);
        assert_eq!(count(InducementDuration::WhileHoldingTheBall), 1);
        assert_eq!(count(InducementDuration::UntilUsed), 1);
        assert_eq!(count(InducementDuration::UntilEndOfOpponentsTurn), 2);
    }
}
