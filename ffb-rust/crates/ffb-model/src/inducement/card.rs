use serde::{Deserialize, Serialize};
use crate::enums::InducementDuration;
use crate::inducement::card_target::CardTarget;

/// 1:1 translation of `com.fumbbl.ffb.inducement.Card`.
/// Represents a single inducement card with its handler key and properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    /// Name of the CardHandlerKey variant that handles this card (matches Java handlerKey().name()).
    pub handler_key_name: Option<String>,
    /// Java: Card.getDuration() — how long the card effect lasts.
    pub duration: Option<InducementDuration>,
    /// Java: Card.isRemainsInPlay() — card stays deactivated on field rather than being removed.
    pub remains_in_play: bool,
    /// Java: `Card.getTarget()` — TURN (played on the turn, no player selection needed) or one of
    /// the player-targeted variants. Defaults to `TURN`, matching most cards.
    #[serde(default)]
    pub target: CardTarget,
    /// Java: `Card.requiresBlockablePlayerSelection()` — overridden per-card-subclass in Java
    /// (e.g. Custard Pie); default `false`. No card catalog with per-card overrides exists yet
    /// in this port (cards are constructed ad hoc, not data-driven) — this field lets a
    /// specific `Card` instance opt in once a caller knows which card it's building.
    #[serde(default)]
    pub requires_blockable_player_selection: bool,
}

impl Card {
    pub fn new(name: impl Into<String>, handler_key_name: Option<impl Into<String>>) -> Self {
        Card {
            name: name.into(),
            handler_key_name: handler_key_name.map(|s| s.into()),
            duration: None,
            remains_in_play: false,
            target: CardTarget::default(),
            requires_blockable_player_selection: false,
        }
    }

    /// Builder: set the duration.
    pub fn with_duration(mut self, duration: InducementDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Builder: set remains_in_play.
    pub fn with_remains_in_play(mut self, remains: bool) -> Self {
        self.remains_in_play = remains;
        self
    }

    /// Builder: set the card's target.
    pub fn with_target(mut self, target: CardTarget) -> Self {
        self.target = target;
        self
    }

    /// Builder: set requires_blockable_player_selection.
    pub fn with_requires_blockable_player_selection(mut self, requires: bool) -> Self {
        self.requires_blockable_player_selection = requires;
        self
    }

    /// Java: Card.getName()
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Java: Card.handlerKey() — returns the handler key name if one is set.
    pub fn handler_key_name(&self) -> Option<&str> {
        self.handler_key_name.as_deref()
    }

    /// Java: Card.getDuration()
    pub fn get_duration(&self) -> Option<InducementDuration> {
        self.duration
    }

    /// Java: Card.isRemainsInPlay()
    pub fn is_remains_in_play(&self) -> bool {
        self.remains_in_play
    }

    /// Java: Card.getTarget()
    pub fn get_target(&self) -> CardTarget {
        self.target
    }

    /// Java: Card.requiresBlockablePlayerSelection()
    pub fn requires_blockable_player_selection(&self) -> bool {
        self.requires_blockable_player_selection
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new("", None::<&str>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_with_handler_key_name() {
        let c = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert_eq!(c.get_name(), "Chop Block");
        assert_eq!(c.handler_key_name(), Some("CHOP_BLOCK"));
    }

    #[test]
    fn card_without_handler_key() {
        let c = Card::new("Bribe", None::<&str>);
        assert_eq!(c.get_name(), "Bribe");
        assert!(c.handler_key_name().is_none());
    }

    #[test]
    fn card_default_is_empty() {
        let c = Card::default();
        assert!(c.get_name().is_empty());
        assert!(c.handler_key_name().is_none());
        assert!(c.get_duration().is_none());
        assert!(!c.is_remains_in_play());
    }

    #[test]
    fn card_with_duration() {
        let c = Card::new("Witch Brew", Some("WITCH_BREW"))
            .with_duration(InducementDuration::UntilEndOfTurn);
        assert_eq!(c.get_duration(), Some(InducementDuration::UntilEndOfTurn));
    }

    #[test]
    fn card_with_remains_in_play() {
        let c = Card::new("Force Shield", Some("FORCE_SHIELD")).with_remains_in_play(true);
        assert!(c.is_remains_in_play());
    }

    #[test]
    fn card_new_has_no_duration_and_not_remains_in_play() {
        let c = Card::new("Test", Some("TEST_KEY"));
        assert!(c.get_duration().is_none());
        assert!(!c.is_remains_in_play());
    }

    #[test]
    fn card_with_until_end_of_opponents_turn_duration() {
        let c = Card::new("Distract", Some("DISTRACT"))
            .with_duration(InducementDuration::UntilEndOfOpponentsTurn);
        assert_eq!(c.get_duration(), Some(InducementDuration::UntilEndOfOpponentsTurn));
    }

    #[test]
    fn card_default_target_is_turn() {
        let c = Card::new("Pit Trap", Some("PIT_TRAP"));
        assert_eq!(c.get_target(), CardTarget::TURN);
        assert!(!c.get_target().is_played_on_player());
    }

    #[test]
    fn card_with_target_sets_player_targeting() {
        let c = Card::new("Custard Pie", Some("CUSTARD_PIE")).with_target(CardTarget::OPPOSING_PLAYER);
        assert_eq!(c.get_target(), CardTarget::OPPOSING_PLAYER);
        assert!(c.get_target().is_played_on_player());
    }

    #[test]
    fn card_default_does_not_require_blockable_player_selection() {
        let c = Card::new("Test", Some("TEST_KEY"));
        assert!(!c.requires_blockable_player_selection());
    }

    #[test]
    fn card_with_requires_blockable_player_selection() {
        let c = Card::new("Custard Pie", Some("CUSTARD_PIE"))
            .with_requires_blockable_player_selection(true);
        assert!(c.requires_blockable_player_selection());
    }
}
