use serde::{Deserialize, Serialize};
use ffb_model::enums::{Rules, CardEffect, CardTarget, InducementDuration, InducementPhase};

/// A parsed inducement record from the JSON data files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InducementDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_singular: String,
    pub cost: u32,
    pub max_count: u32,
    #[serde(default)]
    pub usage: String,
    #[serde(default)]
    pub availability: Option<String>,
}

/// A team's purchased inducement (type + remaining uses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inducement {
    pub id: String,
    pub uses_remaining: u32,
}

impl Inducement {
    pub fn new(id: impl Into<String>, uses: u32) -> Self {
        Inducement { id: id.into(), uses_remaining: uses }
    }

    pub fn is_used_up(&self) -> bool {
        self.uses_remaining == 0
    }

    pub fn use_one(&mut self) {
        if self.uses_remaining > 0 {
            self.uses_remaining -= 1;
        }
    }
}

/// The team's full set of purchased inducements for a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InducementSet {
    pub items: Vec<Inducement>,
}

impl InducementSet {
    pub fn add(&mut self, id: impl Into<String>, uses: u32) {
        self.items.push(Inducement::new(id, uses));
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Inducement> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    pub fn has_available(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id && !i.is_used_up())
    }

    pub fn count_available(&self, id: &str) -> u32 {
        self.items.iter()
            .filter(|i| i.id == id)
            .map(|i| i.uses_remaining)
            .sum()
    }
}

/// A card that can be played as an inducement (Magic Item or Dirty Trick deck).
/// Maps to Java's `com.fumbbl.ffb.inducement.Card`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub short_name: String,
    pub card_type: CardType,
    pub target: CardTarget,
    pub remains_in_play: bool,
    pub phases: Vec<InducementPhase>,
    pub duration: InducementDuration,
    pub description: String,
    /// Engine-side effect applied when the card activates (None = metadata only).
    pub effect: Option<CardEffect>,
}

/// The deck a card belongs to (per-edition).
/// Maps to Java's `com.fumbbl.ffb.inducement.CardType` interface / BB2016+BB2020 enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    MagicItem,
    DirtyTrick,
}

impl CardType {
    pub fn name(self) -> &'static str {
        match self {
            CardType::MagicItem => "magicItem",
            CardType::DirtyTrick => "dirtyTrick",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "magicItem" => Some(Self::MagicItem),
            "dirtyTrick" => Some(Self::DirtyTrick),
            _ => None,
        }
    }
}

/// Whether a bribe inducement can be used at the point of a foul referee check.
pub fn can_use_bribe(set: &InducementSet, _rules: Rules) -> bool {
    set.has_available("bribes")
}

/// Whether the Halfling Master Chef event has been purchased.
pub fn has_master_chef(set: &InducementSet) -> bool {
    set.has_available("halflingMasterChef")
}

/// Bloodweiser Keg: bonus to KO recovery roll.
/// Returns +1 per keg purchased (stacking up to 3).
pub fn bloodweiser_keg_bonus(set: &InducementSet) -> i32 {
    set.count_available("bloodweiserKegs") as i32
}

/// Brawler's Kegs (BB2025 rename of Bloodweiser Kegs).
pub fn brawlers_kegs_bonus(set: &InducementSet) -> i32 {
    (set.count_available("bloodweiserKegs") + set.count_available("brawlersKegs")) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inducement_use_tracks_remaining() {
        let mut ind = Inducement::new("bribes", 3);
        assert!(!ind.is_used_up());
        ind.use_one();
        assert_eq!(ind.uses_remaining, 2);
        ind.use_one();
        ind.use_one();
        assert!(ind.is_used_up());
        ind.use_one(); // no underflow
        assert_eq!(ind.uses_remaining, 0);
    }

    #[test]
    fn inducement_set_has_available() {
        let mut set = InducementSet::default();
        assert!(!set.has_available("bribes"));
        set.add("bribes", 2);
        assert!(set.has_available("bribes"));
    }

    #[test]
    fn bloodweiser_bonus_stacks() {
        let mut set = InducementSet::default();
        set.add("bloodweiserKegs", 3);
        assert_eq!(bloodweiser_keg_bonus(&set), 3);
    }

    #[test]
    fn card_type_round_trip_name() {
        for ct in [CardType::MagicItem, CardType::DirtyTrick] {
            assert_eq!(CardType::from_name(ct.name()), Some(ct));
        }
        assert!(CardType::from_name("unknown").is_none());
    }

    #[test]
    fn card_type_magic_item_name() {
        assert_eq!(CardType::MagicItem.name(), "magicItem");
        assert_eq!(CardType::DirtyTrick.name(), "dirtyTrick");
    }

    #[test]
    fn card_struct_fields_set_correctly() {
        let card = Card {
            name: "Cloud Burster".into(),
            short_name: "CB".into(),
            card_type: CardType::MagicItem,
            target: CardTarget::OwnPlayer,
            remains_in_play: true,
            phases: vec![InducementPhase::StartOfOwnTurn],
            duration: InducementDuration::UntilEndOfTurn,
            description: "A test card".into(),
            effect: Some(CardEffect::Distracted),
        };
        assert_eq!(card.name, "Cloud Burster");
        assert_eq!(card.card_type, CardType::MagicItem);
        assert!(card.remains_in_play);
        assert_eq!(card.duration, InducementDuration::UntilEndOfTurn);
        assert_eq!(card.effect, Some(CardEffect::Distracted));
    }

    #[test]
    fn card_serde_round_trip() {
        let card = Card {
            name: "Sedative Tackle".into(),
            short_name: "ST".into(),
            card_type: CardType::DirtyTrick,
            target: CardTarget::OpposingPlayer,
            remains_in_play: false,
            phases: vec![InducementPhase::EndOfOwnTurn],
            duration: InducementDuration::UntilUsed,
            description: "Sedate a player".into(),
            effect: Some(CardEffect::Sedative),
        };
        let json = serde_json::to_string(&card).unwrap();
        let back: Card = serde_json::from_str(&json).unwrap();
        assert_eq!(card.name, back.name);
        assert_eq!(card.card_type, back.card_type);
        assert_eq!(card.duration, back.duration);
    }

    #[test]
    fn can_use_bribe_requires_remaining() {
        let mut set = InducementSet::default();
        assert!(!can_use_bribe(&set, Rules::Bb2020));
        set.add("bribes", 1);
        assert!(can_use_bribe(&set, Rules::Bb2020));
        set.find_mut("bribes").unwrap().use_one();
        assert!(!can_use_bribe(&set, Rules::Bb2020));
    }
}
